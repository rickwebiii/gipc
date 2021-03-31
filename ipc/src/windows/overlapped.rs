use winapi::{
    shared::{
        winerror::{
            ERROR_SUCCESS
        }
    },
    um::{
        minwinbase::OVERLAPPED,
    },
};

use futures::channel::oneshot::{self, Sender, Receiver};

use std::future::Future;
use std::io;
use std::mem;
use std::pin::Pin;
use std::task::{ Context, Poll };

#[repr(C)]
#[derive(Clone, Debug)]
/// A PODS containing the error code and number of bytes transferred for an I/O operation.
pub struct OverlappedCompletionInfo {
    pub error: i32,
    pub bytes_transferred: u32,
}

#[repr(C)]
/// The data associated with a Win32 I/O operation.
pub struct Overlapped {
    overlapped: OVERLAPPED,
    tx_info: Sender<OverlappedCompletionInfo>,
}

unsafe impl Sync for Overlapped {}
unsafe impl Send for Overlapped {}

impl Overlapped {
    /// Creates a new Overlapped structure for use with Win32 async I/O operations.
    pub fn new() -> io::Result<(Overlapped, OverlappedFuture)> {
        let overlapped: OVERLAPPED = unsafe { mem::zeroed() };
        
        let (tx, rx) = oneshot::channel();

        let overlapped_wrapper = Overlapped {
            overlapped: overlapped,
            tx_info: tx,
        };

        let future = OverlappedFuture {
            rx_info: rx,
        };

        Ok((overlapped_wrapper, future))
    }

    pub fn resolve(self, info: OverlappedCompletionInfo) -> io::Result<()> {
        self.tx_info.send(info)
            .map_err(|_| { io::Error::new(io::ErrorKind::Interrupted, "Oneshot cancelled") })?;

        Ok(())
    }
}

pub struct OverlappedFuture {
    rx_info: Receiver<OverlappedCompletionInfo>,
}

impl Future for OverlappedFuture {
    type Output = io::Result<u32>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        futures::future::Future::poll(Pin::new(&mut self.rx_info), ctx)
            .map(|info_result| {
                match info_result {
                    Ok(info) => {
                        if info.error != ERROR_SUCCESS as i32 {
                            Err(std::io::Error::from_raw_os_error(info.error))
                        } else {
                            Ok(info.bytes_transferred)
                        }
                        
                    },
                    Err(_) => {
                        Err(std::io::Error::new(io::ErrorKind::Interrupted, "Oneshot cancelled"))
                    }
                }
            })
        
        /*
        self.overlapped.set_waker(ctx.waker());

        // Guarantee we never perceive the iocp thread waking us before it's written its
        // results.
        atomic::fence(Ordering::Release);

        match self.overlapped.get_completion_info() {
            Some(info) => {
                if info.error != ERROR_SUCCESS as i32 {
                    Poll::Ready(Err(std::io::Error::from_raw_os_error(info.error)))
                } else {
                    Poll::Ready(Ok(info.bytes_transferred))
                }                
            },
            None => {
                Poll::Pending
            }
        }*/
    }
}