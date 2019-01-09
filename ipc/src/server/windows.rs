use named_pipe::{
  ConnectingServer,
  PipeClient,
  PipeOptions,
  PipeServer,
};

// use futures::io::{AsyncRead, AsyncWrite};

use std::ffi::{OsString};
use std::future::{Future};
use std::pin::{Pin};
use std::task::{LocalWaker, Poll};
use std::time::{Duration};

pub struct IpcServerWrapper {
  server: ConnectingServer,
  pipe_name: OsString,
}

impl IpcServerWrapper {
  pub fn new(name: &str) -> std::io::Result<IpcServerWrapper> {
    let pipe_name = OsString::from(r"\\.\pipe\".to_owned() + name); 
    let connecting_server = PipeOptions::new(&pipe_name)
      .single()?;

    Ok(IpcServerWrapper {
      server: connecting_server,
      pipe_name: pipe_name,
    })
  }


  pub async fn wait_for_client(self) -> std::io::Result<(PipeServer, IpcServerWrapper)> {
    let pipe_server = await!(PipeConnectionFuture::new(self.server))?;

    let connecting_server = PipeOptions::new(&self.pipe_name)
      .first(false)
      .single()?;

    let new_wrapper = IpcServerWrapper {
      server: connecting_server,
      pipe_name: self.pipe_name
    };

    Ok((pipe_server, new_wrapper))
  }
}

struct PipeConnectionFuture {
  connecting_server: Option<ConnectingServer>,
}

impl PipeConnectionFuture {
  pub fn new(server: ConnectingServer) -> PipeConnectionFuture {
    PipeConnectionFuture {
      connecting_server: Some(server)
    }
  }
}

impl Future for PipeConnectionFuture {
  type Output = std::io::Result<PipeServer>;

  fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
    let server = self.get_mut();

    let io_result = server.connecting_server
      .take()
      .expect("Got a None server")
      .wait_ms(50);

    match io_result {
      Ok(connection_result) => {
        match connection_result {
          Ok(pipe_server) => Poll::Ready(Ok(pipe_server)),
          Err(connection_server) => {
            server.connecting_server = Some(connection_server);
            lw.wake();
            Poll::Pending
          }
        }
      },
      Err(err) => {      
        Poll::Ready(Err(err))
      }
    }
  }
}

pub struct IpcClientWrapper {
  client: PipeClient,
  pipe_name: OsString,
}

impl IpcClientWrapper {
  pub fn new(name: &str) -> std::io::Result<IpcClientWrapper> {
    let pipe_name = OsString::from(r"\\.\pipe\".to_owned() + name);
    let mut client = PipeClient::connect(&pipe_name)?;
    client.set_read_timeout(Some(Duration::from_millis(20)));
    client.set_write_timeout(Some(Duration::from_millis(20)));

    Ok(IpcClientWrapper {
      client: client,
      pipe_name: pipe_name
    })
  }

/*
  pub async fn write(&mut self, data: &[u8]) -> std::io::Result<()> {
    let handle = unsafe { self.client.write_async()? }

    
  }
*/
}

// TODO: Remove
struct MyWaker { }

static MY_WAKER: MyWaker = MyWaker {};

impl std::task::Wake for MyWaker {

}

/*

impl AsyncRead for IpcServerWrapper {

}*/

#[test]
fn can_connect() {
  use std::sync::mpsc::channel;
  use std::thread;

  let (tx, rx) = channel();

  thread::spawn(move || {
    let server = IpcServerWrapper::new("horse").unwrap();

    loop {
      server.wait_for_client().wait();
    }
    tx.send(()).unwrap();
  });

  let client = IpcClientWrapper::new("horse");
  rx.recv().unwrap();
}