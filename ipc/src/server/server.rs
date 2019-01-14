#[cfg(windows)]
use super::windows::{IpcConnectionWrapper, IpcClientWrapper, IpcServerWrapper};

pub struct Server {
  a: IpcConnectionWrapper,
  b: IpcClientWrapper,
  c: IpcServerWrapper
}

impl Server {
  
}