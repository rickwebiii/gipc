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

    pub async fn wait_for_connection(self) -> std::io::Result<(RawIpcConnection, MessageIpcServer)> {
        let (connection, server) = await!(self.server.wait_for_connection())?;

        let new_server = MessageIpcServer {
            server: server
        };

        let new_connection = RawIpcConnection {
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
            let (_, buffer) = size_bytes.split_at_mut(bytes_remaining as usize);

            bytes_remaining = bytes_remaining - await!(self.connection.read(buffer))?;
        }

        let size: u64 = u64::from_ne_bytes(size_bytes);

        let mut bytes_remaining: u64 = size;

        let mut data = vec![0; bytes_remaining as usize];

        while bytes_remaining > 0 {
            let (_, buffer) = data.split_at_mut(bytes_remaining as usize);

            // Perform our read in 16MB chunks.
            let (buffer, _) = buffer.split_at_mut(MessageIpcConnection::get_chunk_size(bytes_remaining as usize));

            bytes_remaining = bytes_remaining - await!(self.connection.read(buffer))? as u64;
        }

        Ok(data)
    }

    pub async fn write<'a>(&'a self, data: &'a [u8]) -> std::io::Result<()> {
        let size_bytes = (data.len() as u64).to_ne_bytes();

        let mut bytes_remaining: u32 = 8;

        while bytes_remaining > 0 {
            let (_, buffer) = size_bytes.split_at(bytes_remaining as usize);

            bytes_remaining = bytes_remaining - await!(self.connection.write(buffer))?;
        }

        let mut bytes_remaining: u64 = data.len() as u64;

        while bytes_remaining > 0 {
            let (_, buffer) = data.split_at(bytes_remaining as usize);

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