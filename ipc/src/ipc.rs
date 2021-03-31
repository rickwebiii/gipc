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
        let (connection, server) = self.server.wait_for_connection().await?;

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
        self.connection.read(data).await
    }

    pub async fn write<'a>(&'a self, data: &'a [u8]) -> std::io::Result<u32> {
        self.connection.write(data).await
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
        let (connection, server) = self.server.wait_for_connection().await?;

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

            let bytes_read = self.connection.read(buffer).await?;

            bytes_remaining -= bytes_read;
        }

        let size: u64 = u64::from_ne_bytes(size_bytes);

        let mut bytes_remaining: u64 = size;

        let mut data = vec![0; bytes_remaining as usize];

        while bytes_remaining > 0 {
            let (_, buffer) = data.split_at_mut(size as usize - bytes_remaining as usize);

            // Perform our read in 16MB chunks.
            let (buffer, _) = buffer.split_at_mut(MessageIpcConnection::get_chunk_size(bytes_remaining as usize));

            bytes_remaining = bytes_remaining - self.connection.read(buffer).await? as u64;
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

            let bytes_written = self.connection.write(buffer).await?;

            bytes_remaining -= bytes_written;
        }

        let mut bytes_remaining: u64 = data.len() as u64;

        while bytes_remaining > 0 {
            let (_, buffer) = data.split_at(data.len() - bytes_remaining as usize);

            let (buffer, _) = buffer.split_at(MessageIpcConnection::get_chunk_size(bytes_remaining as usize));

            bytes_remaining = bytes_remaining - self.connection.write(buffer).await? as u64;
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

    use tokio::runtime;
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
            let pool = runtime::Runtime::new().unwrap();

            async fn run_server(
                start_tx: Sender<()>,
                pong_rx: Receiver<()>,
                server_name: &str
            ) -> std::io::Result<()> {
                let server = MessageIpcServer::new(server_name)?;
                start_tx.send(()).unwrap();

                let (connection, _server) = server.wait_for_connection().await?;

                info!("Server receiving");

                let message = connection.read().await?;
                let message = String::from_utf8_lossy(message.as_slice());

                assert_eq!(message, "hello world");

                info!("Server sending");

                let response = "Goodbye.".as_bytes();

                connection.write(response).await?;

                pong_rx.recv().unwrap();

                Ok(())
            }

            pool.block_on(
                async {
                    match run_server(server_started_tx, pong_rx, &server_server_name).await {
                        Ok(_) => {
                            client_connected_tx.send(()).unwrap();
                        }
                        Err(err) => {
                            panic!("Test failed {}", err);
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
            let pool = runtime::Runtime::new().unwrap();

            async fn run_client(pong_tx: Sender<()>, server_name: &str) -> std::io::Result<()> {
                let client = MessageIpcClient::new(server_name)?;

                let data = "hello world".as_bytes();

                info!("Client sending");
                client.write(data).await?;

                info!("Client receiving");
                let response = client.read().await?;

                let response = String::from_utf8_lossy(response.as_slice());

                assert_eq!(response, "Goodbye.");

                pong_tx.send(()).unwrap();

                Ok(())
            }

            // Wait for the server to start.
            server_started_rx.recv().unwrap();

            pool.block_on(
                async {
                    match run_client(pong_tx, &client_server_name).await {
                        Ok(_) => {}
                        Err(err) => {
                            panic!("Test failed {}", err);
                        }
                    };
                },
            );
        }).unwrap();

        server_thread.join().unwrap();
        client_thread.join().unwrap();

        client_connected_rx.recv().unwrap();
    }

    fn allocate_message(len: usize) -> Vec<u8> {
        let mut data: Vec<u8> = vec![0; len];

        for i in 0..data.len() {
            data[i] = i as u8;
        }

        data
    }

    fn validate_message(message: Vec<u8>) {
        for i in 0..message.len() {
            assert_eq!(i as u8, message[i]);
        }
    }

    #[test]
    fn can_write_large_messages() {
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
            let pool = runtime::Runtime::new().unwrap();

            async fn run_server(
                start_tx: Sender<()>,
                pong_rx: Receiver<()>,
                server_name: &str
            ) -> std::io::Result<()> {
                let server = MessageIpcServer::new(server_name)?;
                start_tx.send(()).unwrap();

                let (connection, _server) = server.wait_for_connection().await?;

                info!("Server receiving");

                let message = connection.read().await?;

                validate_message(message);

                info!("Server sending");

                let message = allocate_message(100 * 1024 * 1024);

                connection.write(&message).await?;

                pong_rx.recv().unwrap();

                Ok(())
            }

            pool.block_on(
                async {
                    match run_server(server_started_tx, pong_rx, &server_server_name).await {
                        Ok(_) => {
                            client_connected_tx.send(()).unwrap();
                        }
                        Err(err) => {
                            panic!("Test failed {}", err);
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
            let pool = runtime::Runtime::new().unwrap();

            async fn run_client(pong_tx: Sender<()>, server_name: &str) -> std::io::Result<()> {
                let client = MessageIpcClient::new(server_name)?;

                let message = allocate_message(100 * 1024 * 1024);

                info!("Client sending");
                client.write(&message).await?;

                info!("Client receiving");
                let message = client.read().await?;

                validate_message(message);

                pong_tx.send(()).unwrap();

                Ok(())
            }

            // Wait for the server to start.
            server_started_rx.recv().unwrap();

            pool.block_on(
                async {
                    match run_client(pong_tx, &client_server_name).await {
                        Ok(_) => {}
                        Err(err) => {
                            panic!("Test failed {}", err);
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