#[cfg(windows)]
use super::windows::{IpcClientWrapper, IpcConnectionWrapper, IpcServerWrapper};

use std::cmp::{min};
use std::vec::{Vec};

pub struct RawIpcServer {
    server: IpcServerWrapper,
}

impl RawIpcServer {
    pub fn new(name: &str) -> std::io::Result<RawIpcServer> {
        let server = IpcServerWrapper::new(name)?;

        Ok(RawIpcServer {
            server: server
        })
    }

    pub async fn wait_for_connection(self) -> std::io::Result<(RawIpcConnection, RawIpcServer)> {
        let (connection, server) = await!(self.server.wait_for_connection())?;

        let new_server = RawIpcServer {
            server: server
        };

        let new_connection = RawIpcConnection {
            connection: connection
        };

        Ok((new_connection, new_server))
    }
}


pub struct RawIpcConnection {
    connection: IpcConnectionWrapper,
}

impl RawIpcConnection {
    pub async fn read<'a>(&'a self, data: &'a mut [u8]) -> std::io::Result<u32> {
        await!(self.connection.read(data))
    }

    pub async fn write<'a>(&'a self, data: &'a [u8]) -> std::io::Result<u32> {
        await!(self.connection.write(data))
    }
}

pub struct RawIpcClient {
}

impl RawIpcClient {
    pub fn new(name: &str) -> std::io::Result<RawIpcConnection> {
        let connection = IpcClientWrapper::new(name)?;

        Ok(RawIpcConnection {
            connection: connection
        })
    }
}

pub struct MessageIpcServer {
    server: IpcServerWrapper,
}

impl MessageIpcServer {
    pub fn new(name: &str) -> std::io::Result<MessageIpcServer> {
        let server = IpcServerWrapper::new(name)?;

        Ok(MessageIpcServer {
            server: server
        })
    }

    pub async fn wait_for_connection(self) -> std::io::Result<(MessageIpcConnection, MessageIpcServer)> {
        let (connection, server) = await!(self.server.wait_for_connection())?;

        let new_server = MessageIpcServer {
            server: server
        };

        let new_connection = MessageIpcConnection {
            connection: connection
        };

        Ok((new_connection, new_server))
    }
}

pub struct MessageIpcConnection {
    connection: IpcConnectionWrapper,
}


impl MessageIpcConnection {
    pub async fn read<'a>(&'a self) -> std::io::Result<Vec<u8>> {
        let mut size_bytes: [u8; 8] = [0; 8];

        let mut bytes_remaining: u32 = 8;

        while bytes_remaining > 0 {
            let (_, buffer) = size_bytes.split_at_mut(8 - bytes_remaining as usize);

            let bytes_read = await!(self.connection.read(buffer))?;

            bytes_remaining -= bytes_read;
        }

        let size: u64 = u64::from_ne_bytes(size_bytes);

        let mut bytes_remaining: u64 = size;

        let mut data = vec![0; bytes_remaining as usize];

        while bytes_remaining > 0 {
            let (_, buffer) = data.split_at_mut(size as usize - bytes_remaining as usize);

            // Perform our read in 16MB chunks.
            let (buffer, _) = buffer.split_at_mut(MessageIpcConnection::get_chunk_size(bytes_remaining as usize));

            bytes_remaining = bytes_remaining - await!(self.connection.read(buffer))? as u64;
        }

        Ok(data)
    }

    pub async fn write<'a>(&'a self, data: &'a [u8]) -> std::io::Result<()> {
        if data.len() == 0 {
            return Ok(());
        }

        let size_bytes = (data.len() as u64).to_ne_bytes();

        let mut bytes_remaining: u32 = 8;

        while bytes_remaining > 0 {
            let (_, buffer) = size_bytes.split_at(8 - bytes_remaining as usize);

            let bytes_written = await!(self.connection.write(buffer))?;

            bytes_remaining -= bytes_written;
        }

        let mut bytes_remaining: u64 = data.len() as u64;

        while bytes_remaining > 0 {
            let (_, buffer) = data.split_at(data.len() - bytes_remaining as usize);

            let (buffer, _) = buffer.split_at(MessageIpcConnection::get_chunk_size(bytes_remaining as usize));

            bytes_remaining = bytes_remaining - await!(self.connection.write(buffer))? as u64;
        }

        Ok(())
    }

    fn get_chunk_size(bytes_remaining: usize) -> usize {
        min(bytes_remaining as usize, 16 * 1024 * 1024)
    }
}

pub struct MessageIpcClient {
}

impl MessageIpcClient {
    pub fn new(name: &str) -> std::io::Result<MessageIpcConnection> {
        let connection = IpcClientWrapper::new(name)?;

        Ok(MessageIpcConnection {
            connection: connection
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{MessageIpcClient, MessageIpcServer};
    use crate::test_utils::{get_server_name, install_logger};

    use futures::executor::ThreadPoolBuilder;
    use log::{info};

    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::thread;

    #[test]
    fn messaging_ipc_hello_world() {
        install_logger();

        let server_name = get_server_name();

        let (server_started_tx, server_started_rx) = channel();
        let (client_connected_tx, client_connected_rx) = channel();
        // A side-channel so the server knows the client has received its data
        // and the thread can die.
        let (pong_tx, pong_rx) = channel();

        let server_server_name = server_name.to_owned();

        let server_thread = thread::Builder::new()
            .name(server_name.to_owned() + "_server")
            .spawn(move || 
        {
            let mut pool = ThreadPoolBuilder::new().pool_size(1).create().unwrap();

            async fn run_server(
                start_tx: Sender<()>,
                pong_rx: Receiver<()>,
                server_name: &str
            ) -> std::io::Result<()> {
                let server = MessageIpcServer::new(server_name)?;
                start_tx.send(()).unwrap();

                let (connection, _server) = await!(server.wait_for_connection())?;

                info!("Server receiving");

                let message = await!(connection.read())?;
                let message = String::from_utf8_lossy(message.as_slice());

                assert_eq!(message, "hello world");

                info!("Server sending");

                let response = "Goodbye.".as_bytes();

                await!(connection.write(response))?;

                pong_rx.recv().unwrap();

                Ok(())
            }

            pool.run(
                async {
                    match await!(run_server(server_started_tx, pong_rx, &server_server_name)) {
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

        let client_server_name = server_name.to_owned();

        let client_thread = thread::Builder::new()
            .name(server_name.to_owned() + "client")
            .spawn(move || 
        {
            let mut pool = ThreadPoolBuilder::new().pool_size(1).create().unwrap();

            async fn run_client(pong_tx: Sender<()>, server_name: &str) -> std::io::Result<()> {
                let client = MessageIpcClient::new(server_name)?;

                let data = "hello world".as_bytes();

                info!("Client sending");
                await!(client.write(data))?;

                info!("Client receiving");
                let response = await!(client.read())?;

                let response = String::from_utf8_lossy(response.as_slice());

                assert_eq!(response, "Goodbye.");

                pong_tx.send(()).unwrap();

                Ok(())
            }

            // Wait for the server to start.
            server_started_rx.recv().unwrap();

            pool.run(
                async {
                    match await!(run_client(pong_tx, &client_server_name)) {
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
    }
}