#[cfg(windows)]
use super::windows::{IpcClientWrapper, IpcConnectionWrapper, IpcServerWrapper};

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