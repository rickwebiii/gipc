//use futures::{Future};
//use futures::task::{LocalWaker, Poll};
// use futures::io::{AsyncRead, AsyncWrite};

use super::named_pipe::{NamedPipeClient, NamedPipeConnection, NamedPipeServer};

use std::ffi::OsString;

pub struct IpcServerWrapper {
    pipe: NamedPipeServer,
}

impl IpcServerWrapper {
    pub fn new(name: &str) -> std::io::Result<IpcServerWrapper> {
        Ok(IpcServerWrapper {
            pipe: NamedPipeServer::new(name)?,
        })
    }

    fn from(server: NamedPipeServer) -> IpcServerWrapper {
        IpcServerWrapper { pipe: server }
    }

    pub async fn wait_for_connection(
        self,
    ) -> std::io::Result<(IpcConnectionWrapper, IpcServerWrapper)> {
        let (pipe_connection, server) = await!(self.pipe.wait_for_connection())?;

        Ok((
            IpcConnectionWrapper::new(pipe_connection),
            IpcServerWrapper::from(server),
        ))
    }
}

pub struct IpcConnectionWrapper {
    pipe_connection: NamedPipeConnection,
}

impl IpcConnectionWrapper {
    pub fn new(pipe_connection: NamedPipeConnection) -> IpcConnectionWrapper {
        IpcConnectionWrapper {
            pipe_connection: pipe_connection,
        }
    }

    pub async fn read<'a>(&'a self, data: &'a mut [u8]) -> std::io::Result<u32> {
        await!(self.pipe_connection.read(data))
    }

    pub async fn write<'a>(&'a self, data: &'a [u8]) -> std::io::Result<u32> {
        await!(self.pipe_connection.write(data))
    }
}

pub struct IpcClientWrapper {}

impl IpcClientWrapper {
    pub fn new(pipe_name: &str) -> std::io::Result<IpcConnectionWrapper> {
        let pipe_connection = NamedPipeClient::new(pipe_name)?;

        Ok(IpcConnectionWrapper {
            pipe_connection: pipe_connection,
        })
    }
}
