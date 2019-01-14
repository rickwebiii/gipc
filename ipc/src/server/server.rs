#[cfg(windows)]
use super::windows::{IpcClientWrapper, IpcConnectionWrapper, IpcServerWrapper};

pub struct Server {
    a: IpcConnectionWrapper,
    b: IpcClientWrapper,
    c: IpcServerWrapper,
}

impl Server {}
