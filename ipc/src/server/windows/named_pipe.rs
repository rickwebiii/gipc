
//use futures::{Future};
//use futures::task::{LocalWaker, Poll};
// use futures::io::{AsyncRead, AsyncWrite};
use winapi::{
  um::{
    //errhandlingapi::*, 
    //fileapi::*,
    handleapi::{
      INVALID_HANDLE_VALUE
    },
    //ioapiset::*,
    //minwinbase::*,
    namedpipeapi::{
      CreateNamedPipeW,
      ConnectNamedPipe,
    },
    //synchapi::*,
    winbase::{
      FILE_FLAG_FIRST_PIPE_INSTANCE,
      FILE_FLAG_OVERLAPPED,
      PIPE_ACCESS_DUPLEX,
      PIPE_READMODE_BYTE,
      PIPE_TYPE_BYTE,
      PIPE_UNLIMITED_INSTANCES,
      PIPE_WAIT
    },
    //winnt::*,
  }
};

use super::handle::{Handle};
use super::overlapped::{Overlapped};

use std::ffi::{OsStr};
use std::os::windows::ffi::{OsStrExt};
use std::ptr;

pub struct NamedPipe {
  handle: Handle
}

impl NamedPipe {
  pub fn new(name: &OsStr, first: bool) -> std::io::Result<NamedPipe> {
    let first_instance = if first { FILE_FLAG_FIRST_PIPE_INSTANCE } else { 0 };
    let mut pipe_name = name.to_owned();
    pipe_name.push("\x00");
    let pipe_name = pipe_name.encode_wide().collect::<Vec<u16>>();

    let handle = unsafe {
      CreateNamedPipeW(
        pipe_name.as_ptr(),
        PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | first_instance,
        PIPE_TYPE_BYTE | PIPE_READMODE_BYTE| PIPE_WAIT,
        PIPE_UNLIMITED_INSTANCES,
        1024*1024,
        1024*1024,
        0,
        ptr::null_mut()
      )
    };

    if handle == INVALID_HANDLE_VALUE {
      return Err(std::io::Error::last_os_error());
    }

    Ok(NamedPipe {
      handle: Handle {
        value: handle
      }
    })
  }

  pub async fn wait_for_connection(self) -> std::io::Result<()> {
    let mut overlapped = Overlapped::new()?;

    unsafe {
      ConnectNamedPipe(self.handle.value, overlapped.get_mut());
    };

    await!(overlapped.await());

    Ok(())
  }
}