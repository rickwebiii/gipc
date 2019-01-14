use log::{debug};
use futures::{Future};
use futures::task::{LocalWaker, Poll};
use winapi::{
    shared::{
        winerror::{
            WAIT_TIMEOUT,
        }
    },
    um::{
        synchapi::{
            CreateEventW,
            ResetEvent,
            SetEvent,
            WaitForSingleObject,
        },
        winbase::{
            WAIT_OBJECT_0,
        }
    }
};

use super::handle::{Handle};

use std::io;
use std::pin::{Pin};
use std::ptr;

#[derive(Debug)]
pub struct Event {
    pub handle: Handle,
}

impl Event {
    pub fn new() -> io::Result<Event> {
        let handle = unsafe { CreateEventW(ptr::null_mut(), 1, 0, ptr::null()) };
        if handle != ptr::null_mut() {
            debug!("Created event {:?}", handle);
            Ok(Event {
                handle: Handle::new(handle),
            })
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn reset(&self) -> io::Result<()> {
        let result = unsafe { ResetEvent(self.handle.value) };
        if result != 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn set(&self) -> io::Result<()> {
        let result = unsafe { SetEvent(self.handle.value) };
        if result != 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub async fn await(self) -> io::Result<()> {
        let handle = self.handle.value;

        await!(EventFuture::new(self.handle))?;

        debug!("Event {:?} completed", handle);

        Ok(())
    }
}

struct EventFuture {
    handle: Handle,
}

impl EventFuture {
    fn new(handle: Handle) -> EventFuture {
        EventFuture {
            handle: handle
        }
    }
}

impl Future for EventFuture {
    type Output = std::io::Result<()>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        let result = unsafe { WaitForSingleObject(self.handle.value, 0) };

        match result {
            WAIT_OBJECT_0 => { Poll::Ready(Ok(())) },
            WAIT_TIMEOUT => {
                lw.wake();
                Poll::Pending
            },
            _ => {
                Poll::Ready(Err(io::Error::last_os_error()))
            }
        }
    }
}