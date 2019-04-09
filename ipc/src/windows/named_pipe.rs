use log::{error};
use winapi::{
    shared::{
        minwindef::TRUE,
        winerror::{ERROR_IO_PENDING, ERROR_NO_DATA, ERROR_PIPE_CONNECTED},
    },
    um::{
        fileapi::{CreateFileW, ReadFile, WriteFile, OPEN_EXISTING},
        handleapi::INVALID_HANDLE_VALUE,
        namedpipeapi::{ConnectNamedPipe, CreateNamedPipeW},
        winbase::{
            FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, PIPE_ACCESS_DUPLEX,
            PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE,
            PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
        },
        winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE},
    },
};

use super::completion_port::{CompletionPort};
use super::handle::Handle;
use super::overlapped::Overlapped;

use std::ffi::{c_void, OsStr, OsString};
use std::os::windows::ffi::OsStrExt;
use std::mem;
use std::ptr;
use std::sync::{Arc};

pub struct NamedPipeServer {
    handle: Handle,
    name: OsString,
}

const PIPE_PREFIX: &str = r"\\.\pipe\";

fn make_pipe_name(osstr: &OsStr) -> Vec<u16> {
    let mut name = osstr.to_owned();
    name.push("\x00");
    name.encode_wide().collect::<Vec<u16>>()
}

impl NamedPipeServer {
    /// Creates a new pipe server on \\.\pipe\<name>.
    pub fn new(name: &str) -> std::io::Result<NamedPipeServer> {
        let pipe = NamedPipeServer::create(&OsString::from(PIPE_PREFIX.to_owned() + name), true)?;

        Ok(pipe)
    }

    fn create(name: &OsStr, first: bool) -> std::io::Result<NamedPipeServer> {
        let first_instance = if first {
            FILE_FLAG_FIRST_PIPE_INSTANCE
        } else {
            0
        };
        let pipe_name = name.to_owned();
        let pipe_name_bytes = make_pipe_name(&pipe_name);

        let handle = unsafe {
            // SECURITY: Reject remote clients, as this presents potential security ramifications for consumers
            // and this library is intended for communication within a single machine.
            CreateNamedPipeW(
                pipe_name_bytes.as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | first_instance,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                PIPE_UNLIMITED_INSTANCES,
                0,
                0,
                0,
                ptr::null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error());
        }

        let handle = Handle::new(handle);

        CompletionPort::get()?.add_file_handle(&handle)?;

        Ok(NamedPipeServer {
            handle: handle,
            name: pipe_name,
        })
    }

    /// Blocks the current task until a client connects.
    pub async fn wait_for_connection(
        self,
    ) -> std::io::Result<(NamedPipeConnection, NamedPipeServer)> {
        let (connection, server) = await!(self.wait_for_connection_internal())?;

        Ok((connection, server))
    }

    async fn wait_for_connection_internal(
        self,
    ) -> std::io::Result<(NamedPipeConnection, NamedPipeServer)> {
        let (overlapped, overlapped_awaiter) = Overlapped::new()?;

        let new_pipe = NamedPipeServer::create(&self.name, false)?;

        let success = unsafe { ConnectNamedPipe(self.handle.value, mem::transmute(Arc::into_raw(overlapped))) };

        // If the client connected between us creating the pipe and calling ConnectNamedPipe,
        // the ConnectNamedPipe returns false and last_os_error() returns ERROR_PIPE_CONNECTED
        if success != TRUE {
            let err = std::io::Error::last_os_error();

            match err.raw_os_error().unwrap() as u32 {
                ERROR_IO_PENDING => { }
                ERROR_PIPE_CONNECTED => {
                    return Ok((NamedPipeConnection::new(self.handle), new_pipe));
                }
                _ => {
                    return Err(err);
                }
            }
        };

        await!(overlapped_awaiter.await())?;

        let connection = NamedPipeConnection::new(self.handle);

        Ok((connection, new_pipe))
    }
}

pub struct NamedPipeConnection {
    handle: Handle,
}

impl NamedPipeConnection {
    /// Creates a new named pipe connection.
    pub fn new(handle: Handle) -> NamedPipeConnection {
        NamedPipeConnection { handle: handle }
    }

    /// Reads data on named pipe connection, blocking the current task until data exists.
    pub async fn read<'a>(&'a self, data: &'a mut [u8]) -> std::io::Result<u32> {
        let mut bytes_read: u32 = 0;

        while bytes_read == 0 {
            bytes_read = await!(self.read_internal(data))?;
        }

        Ok(bytes_read)
    }

    async fn read_internal<'a>(&'a self, data: &'a mut [u8]) -> std::io::Result<u32> {
        let (overlapped, overlapped_awaiter) = Overlapped::new()?;
        let mut bytes_read: u32 = 0;

        let result = unsafe {
            ReadFile(
                self.handle.value,
                data.as_mut_ptr() as *mut c_void,
                data.len() as u32,
                &mut bytes_read,
                mem::transmute(Arc::into_raw(overlapped)),
            )
        };

        if result != TRUE {
            let err = std::io::Error::last_os_error();

            match err.raw_os_error().unwrap() as u32 {
                ERROR_IO_PENDING => {}, // Expected, as we're not blocking on I/O
                ERROR_NO_DATA => { return Ok(0); }, // If we have no data
                _ => {
                    error!("Read error: {:?}", err);
                    return Err(err);
                }
            }
        } else {
            return Ok(bytes_read);
        }

        let bytes_read: u32 = await!(overlapped_awaiter.await())?;

        Ok(bytes_read)
    }

    /// Writes the specified data to the named pipe connection. The resulting task blocks
    /// until this completes.
    pub async fn write<'a>(&'a self, data: &'a [u8]) -> std::io::Result<u32> {
        let (overlapped, overlapped_awaiter) = Overlapped::new()?;
        let mut bytes_written: u32 = 0;

        let result = unsafe {
            WriteFile(
                self.handle.value,
                data.as_ptr() as *const c_void,
                data.len() as u32,
                &mut bytes_written,
                mem::transmute(Arc::into_raw(overlapped)),
            )
        };

        if result != TRUE {
            let err = std::io::Error::last_os_error();

            match err.raw_os_error().unwrap() as u32 {
                ERROR_IO_PENDING => { }, // Expected, as we're not blocking on I/O
                _ => {
                    error!("Write error: {:?}", err);
                    return Err(err);
                }
            }
        } else {
            return Ok(bytes_written);
        }

        let bytes_written: u32 = await!(overlapped_awaiter.await())?;

        Ok(bytes_written)
    }
}

pub struct NamedPipeClient {
}

impl NamedPipeClient {
    /// Creates a named pipe connection to \\.\pipe\<pipe_name>
    pub fn new(pipe_name: &str) -> std::io::Result<NamedPipeConnection> {
        let pipe_name_bytes = make_pipe_name(&OsString::from(PIPE_PREFIX.to_owned() + pipe_name));

        let handle = unsafe {
            CreateFileW(
                pipe_name_bytes.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                ptr::null_mut(),
                OPEN_EXISTING,
                FILE_FLAG_OVERLAPPED,
                ptr::null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error());
        }

        let handle = Handle::new(handle);

        CompletionPort::get()?.add_file_handle(&handle)?;

        Ok(NamedPipeConnection::new(handle))
    }
}

#[cfg(test)]
mod tests {
    //use super::{Handle}; // Uncomment when asserting handles get freed
    use super::{NamedPipeClient, NamedPipeServer};
    use crate::test_utils::{install_logger};

    use futures::executor::ThreadPoolBuilder;
    use log::{info};

    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::thread;


    /// Asserts there are no open Win32 HANDLEs. This assertion relies on a global atomic,
    /// which probably won't be zero if tests are running in parallel. To actually test for,
    /// leaks, uncomment the contained assertion and run "cargo test -- --test-threads=1"
    pub fn assert_no_handles() {
        // After running the tests, we expect the handle to the io completion queue singleton to remain.
        // assert_eq!(Handle::num_open_handles(), 1);
    }

    #[test]
    fn can_connect_to_named_pipe() {
        install_logger();

        info!("Starting test can_connect_to_named_pipe");

        let (server_started_tx, server_started_rx) = channel();
        let (client_connected_tx, client_connected_rx) = channel();
        let (server_got_connection_tx, server_got_connection_rx) = channel();

        let server_thread = thread::spawn(move || {
            let mut pool = ThreadPoolBuilder::new().pool_size(1).create().unwrap();

            async fn run_server(
                start_tx: Sender<()>,
                connect_tx: Sender<()>,
            ) -> std::io::Result<()> {
                let server = NamedPipeServer::new("horse")?;
                start_tx.send(()).unwrap();

                let (_conection, _server) = await!(server.wait_for_connection())?;

                connect_tx.send(()).unwrap();

                Ok(())
            }

            pool.run(
                async {
                    match await!(run_server(server_started_tx, server_got_connection_tx)) {
                        Ok(_) => {
                            client_connected_tx.send(()).unwrap();
                        }
                        Err(err) => {
                            panic!(format!("Test failed {}", err));
                        }
                    };
                },
            );
        });

        let client_thread = thread::spawn(move || {
            let mut pool = ThreadPoolBuilder::new().pool_size(1).create().unwrap();

            async fn run_client(connect_rx: Receiver<()>) -> std::io::Result<()> {
                let _client = NamedPipeClient::new("horse")?;

                connect_rx.recv().unwrap();

                Ok(())
            }

            server_started_rx.recv().unwrap();

            pool.run(
                async {
                    match await!(run_client(server_got_connection_rx)) {
                        Ok(_) => {}
                        Err(err) => {
                            panic!(format!("Test failed {}", err));
                        }
                    };
                },
            );
        });

        server_thread.join().unwrap();
        client_thread.join().unwrap();

        client_connected_rx.recv().unwrap();

        assert_no_handles();
    }

    #[test]
    fn can_send_data_over_named_pipe() {
        install_logger();

        info!("Starting test can_send_data_over_named_pipe");

        let (server_started_tx, server_started_rx) = channel();
        let (client_connected_tx, client_connected_rx) = channel();
        // A side-channel so the server knows the client has received its data
        // and the thread can die.
        let (pong_tx, pong_rx) = channel();

        let server_thread = thread::Builder::new()
            .name("server".to_owned())
            .spawn(move || 
        {
            let mut pool = ThreadPoolBuilder::new().pool_size(1).create().unwrap();

            async fn run_server(
                start_tx: Sender<()>,
                pong_rx: Receiver<()>,
            ) -> std::io::Result<()> {
                let server = NamedPipeServer::new("cow")?;
                start_tx.send(()).unwrap();

                let (connection, _server) = await!(server.wait_for_connection())?;

                let mut data: Vec<u8> = vec![0; 16];

                info!("Server receiving");

                let bytes_read = await!(connection.read(data.as_mut_slice()))?;

                assert_eq!(bytes_read, 16);

                for i in 0..16 {
                    assert_eq!(i as u8, data[i]);
                    data[i] *= 2;
                }

                info!("Server sending");

                let _bytes_written = await!(connection.write(data.as_slice()))?;

                pong_rx.recv().unwrap();

                Ok(())
            }

            pool.run(
                async {
                    match await!(run_server(server_started_tx, pong_rx)) {
                        Ok(_) => {
                            client_connected_tx.send(()).unwrap();
                        }
                        Err(err) => {
                            panic!(format!("Test failed {}", err));
                        }
                    };
                },
            );
        }).unwrap();

        let client_thread = thread::Builder::new()
            .name("client".to_owned())
            .spawn(move || 
        {
            let mut pool = ThreadPoolBuilder::new().pool_size(1).create().unwrap();

            async fn run_client(pong_tx: Sender<()>) -> std::io::Result<()> {
                let client = NamedPipeClient::new("cow")?;

                let mut data: Vec<u8> = vec![];

                for i in 0..16 {
                    data.push(i as u8);
                }

                info!("Client sending");
                await!(client.write(data.as_slice()))?;

                info!("Client receiving");
                await!(client.read(data.as_mut_slice()))?;

                for i in 0..16 {
                    assert_eq!(data[i], 2 * i as u8);
                }

                pong_tx.send(()).unwrap();

                Ok(())
            }

            // Wait for the server to start.
            server_started_rx.recv().unwrap();

            pool.run(
                async {
                    match await!(run_client(pong_tx)) {
                        Ok(_) => {}
                        Err(err) => {
                            panic!(format!("Test failed {}", err));
                        }
                    };
                },
            );
        }).unwrap();

        server_thread.join().unwrap();
        client_thread.join().unwrap();

        client_connected_rx.recv().unwrap();

        assert_no_handles();
    }
}
