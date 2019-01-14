
//use futures::{Future};
//use futures::task::{LocalWaker, Poll};
// use futures::io::{AsyncRead, AsyncWrite};
use winapi::{
  um::{
    fileapi::{
      CreateFileW,
      OPEN_EXISTING,
      ReadFile,
      WriteFile,
    },
    handleapi::{
      INVALID_HANDLE_VALUE
    },
    namedpipeapi::{
      CreateNamedPipeW,
      ConnectNamedPipe,
    },
    winbase::{
      FILE_FLAG_FIRST_PIPE_INSTANCE,
      FILE_FLAG_OVERLAPPED,
      PIPE_ACCESS_DUPLEX,
      PIPE_READMODE_BYTE,
      PIPE_TYPE_BYTE,
      PIPE_UNLIMITED_INSTANCES,
      PIPE_WAIT
    },
    winnt::{
      FILE_SHARE_READ,
      FILE_SHARE_WRITE,
      GENERIC_READ,
      GENERIC_WRITE,
    }
  }
};

use super::handle::{Handle};
use super::overlapped::{Overlapped};

use std::ffi::{c_void, OsStr, OsString};
use std::os::windows::ffi::{OsStrExt};
use std::ptr;

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
  pub fn new(name: &str) -> std::io::Result<NamedPipeServer> {
    NamedPipeServer::create(&OsString::from(PIPE_PREFIX.to_owned() + name), true)
  }

  fn create(name: &OsStr, first: bool) -> std::io::Result<NamedPipeServer> {
    let first_instance = if first { FILE_FLAG_FIRST_PIPE_INSTANCE } else { 0 };
    let pipe_name = name.to_owned();
    let pipe_name_bytes = make_pipe_name(&pipe_name);

    let handle = unsafe {
      CreateNamedPipeW(
        pipe_name_bytes.as_ptr(),
        PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | first_instance,
        PIPE_TYPE_BYTE | PIPE_READMODE_BYTE| PIPE_WAIT,
        PIPE_UNLIMITED_INSTANCES,
        1024 * 1024,
        1024 * 1024,
        0,
        ptr::null_mut()
      )
    };

    if handle == INVALID_HANDLE_VALUE {
      return Err(std::io::Error::last_os_error());
    }

    Ok(NamedPipeServer {
      handle: Handle::new(handle),
      name: pipe_name
    })
  }

  pub async fn wait_for_connection(self) -> std::io::Result<(PipeConnection, NamedPipeServer)> {
    let mut overlapped = Overlapped::new()?;

    unsafe {
      ConnectNamedPipe(self.handle.value, overlapped.get_mut());
    };

    await!(overlapped.await())?;

    let connection = PipeConnection::new(self.handle);
    let new_pipe = NamedPipeServer::create(&self.name, false)?;

    Ok((connection, new_pipe))
  }
}

pub struct PipeConnection {
  handle: Handle,
}

impl PipeConnection {
  pub fn new(handle: Handle) -> PipeConnection {
    PipeConnection {
      handle: handle
    }
  }

  pub async fn read<'a>(&'a self, data: &'a mut [u8]) -> std::io::Result<u32> {
    let mut overlapped = Overlapped::new()?;
    let mut bytes_read: u32 = 0;

    unsafe {
      ReadFile(
        self.handle.value,
        data.as_mut_ptr() as *mut c_void,
        data.len() as u32,
        &mut bytes_read,
        overlapped.get_mut()
      )
    };

    await!(overlapped.await())?;

    Ok(bytes_read)
  }

  pub async fn write<'a>(&'a self, data: &'a mut [u8]) -> std::io::Result<u32> {
    let mut overlapped = Overlapped::new()?;
    let mut bytes_written: u32 = 0;

    unsafe {
      WriteFile(
        self.handle.value,
        data.as_mut_ptr() as *mut c_void,
        data.len() as u32,
        &mut bytes_written,
        overlapped.get_mut()
      )
    };

    await!(overlapped.await())?;

    Ok(bytes_written)
  }
}

pub struct NamedPipeClient {

}

impl NamedPipeClient {
  pub fn new(pipe_name: &str) -> std::io::Result<PipeConnection> {
    let pipe_name_bytes = make_pipe_name(&OsString::from(PIPE_PREFIX.to_owned() + pipe_name));

    let handle = unsafe {
      CreateFileW(
        pipe_name_bytes.as_ptr(),
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        ptr::null_mut(),
        OPEN_EXISTING,
        FILE_FLAG_OVERLAPPED,
        ptr::null_mut()
      )
    };

    if handle == INVALID_HANDLE_VALUE {
      return Err(std::io::Error::last_os_error());
    }

    Ok(PipeConnection::new(Handle::new(handle)))
  }
}

#[test]
fn can_connect_to_named_pipe() {
  use std::thread;
  use std::sync::mpsc::{channel, Sender};
  use futures::executor::ThreadPoolBuilder;


  let (server_started_tx, server_started_rx) = channel();
  let (client_connected_tx, client_connected_rx) = channel();

  thread::spawn(move || {
    let mut pool = ThreadPoolBuilder::new()
      .pool_size(1)
      .create()
      .unwrap();

    async fn run_server(start_tx: Sender<()>) -> std::io::Result<()> {
      let server = NamedPipeServer::new("horse")?;
      start_tx.send(()).unwrap();

      let (_conection, _server) = await!(server.wait_for_connection())?;

      Ok(())
    }

    pool.run(async {
      match await!(run_server(server_started_tx)) {
        Ok(_) => { client_connected_tx.send(()).unwrap(); },
        Err(err) => { panic!(format!("Test failed {}", err)); }
      };
    });
  });

  thread::spawn(move || {
    let mut pool = ThreadPoolBuilder::new()
      .pool_size(1)
      .create()
      .unwrap();

    async fn run_client() -> std::io::Result<()> {
      let _client = NamedPipeClient::new("horse")?;

      Ok(())
    }
    server_started_rx.recv().unwrap();

    pool.run(async {
      match await!(run_client()) {
        Ok(_) => { },
        Err(err) => { panic!(format!("Test failed {}", err)); }
      };
    });
  });

  client_connected_rx.recv().unwrap();
}