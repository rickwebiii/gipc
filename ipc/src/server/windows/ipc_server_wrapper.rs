//use futures::{Future};
//use futures::task::{LocalWaker, Poll};
// use futures::io::{AsyncRead, AsyncWrite};

use super::named_pipe::{NamedPipeServer};

use std::ffi::{OsString};

pub struct IpcServerWrapper {
  pipe: NamedPipeServer,
}

impl IpcServerWrapper {
  pub fn new(name: &str) -> std::io::Result<IpcServerWrapper> {
    Ok(IpcServerWrapper {
      pipe: NamedPipeServer::new(name)?,
    })
  }
  
  /*
  pub async fn wait_for_client(self) -> std::io::Result<(IpcConnection, IpcServerWrapper)> {
    let overlapped = 

    unsafe { namedpipeapi::ConnectNamedPipe(self.server, ) }

    //let pipe_server = await!(PipeConnectionFuture::new(self.server))?;

    

    let new_wrapper = IpcServerWrapper {
      
      pipe_name: self.pipe_name
    };

    Ok((pipe_server, new_wrapper))
  }*/
}

#[test]
fn can_connect() {
  use futures::executor::{ThreadPoolBuilder};

  use std::sync::mpsc::channel;
  use std::sync::{Arc};
  use std::thread;

  let (client_done_tx, client_done_rx) = channel();
  let (server_started_tx, server_started_rx) = channel();
  let (server_done_tx, server_done_rx) = channel();

  // Spawn the server
  thread::spawn(move || {
    let mut thread_pool = ThreadPoolBuilder::new()
      .pool_size(2)
      .create()
      .unwrap();

    thread_pool.run(async {
      let server = IpcServerWrapper::new("horse").unwrap();
      server_started_tx.send(()).unwrap();
      await!(server.wait_for_client()).unwrap();
    });

    server_done_tx.send(()).unwrap();
  });

  thread::spawn(move || {
    let mut thread_pool = ThreadPoolBuilder::new()
      .pool_size(2)
      .create()
      .unwrap();

    server_started_rx.recv().unwrap();

    thread_pool.run(async {
      let client = IpcClientWrapper::new("horse").unwrap();
    });

    client_done_tx.send(()).unwrap();
  });

  server_done_rx.recv().unwrap();
  client_done_rx.recv().unwrap();
}