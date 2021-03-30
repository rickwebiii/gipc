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

use std::future::Future;
use std::io;
use std::mem;
use std::cell::RefCell;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic;
use std::sync::atomic::Ordering;
use std::task::{ Context, Poll, Waker };

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
    waker: RefCell<Option<Waker>>,
    completion_info: RefCell<Option<OverlappedCompletionInfo>>
}

unsafe impl Sync for Overlapped {}
unsafe impl Send for Overlapped {}

impl Overlapped {
    /// Creates a new Overlapped structure for use with Win32 async I/O operations.
    pub fn new() -> io::Result<(Arc<Overlapped>, OverlappedAwaiter)> {
        let overlapped: OVERLAPPED = unsafe { mem::zeroed() };
        
        let overlapped_wrapper = Arc::new(Overlapped {
            overlapped: overlapped,
            waker: RefCell::new(None),
            completion_info: RefCell::new(None)
        });

        let overlapped_awaiter = OverlappedAwaiter {
            overlapped: overlapped_wrapper.clone()
        };

        Ok((overlapped_wrapper, overlapped_awaiter))
    }

    /// Sets the completion info for the finished I/O operation.
    pub fn set_completion_info(&self, info: OverlappedCompletionInfo) {
        self.completion_info.borrow_mut().replace(info);
    }

    /// Gets the completion info associated with the completed overlapped operation, or None if
    /// the operation hansn't completed.
    pub fn get_completion_info(&self) -> Option<OverlappedCompletionInfo> {
        self.completion_info.borrow().clone()
    }

    /// Gets the waker that alerts the task this overlapped operation needs polling (i.e. has completed).
    pub fn get_waker(&self) -> Option<Waker> {
        self.waker.borrow().clone()
    }

    fn set_waker(&self, waker: &Waker) {
        self.waker.borrow_mut().replace(waker.clone());
    }
}

/// A item returned with an overlapped that you can await for the associated I/O operation
/// to complete.
pub struct OverlappedAwaiter {
    overlapped: Arc<Overlapped>,
}

impl OverlappedAwaiter {
    /// Blocks the current task until the associated overlapped completes.
    pub async fn await_overlapped(self) -> io::Result<u32> {
        let bytes_transferred = OverlappedFuture::new(self.overlapped.clone()).await?;

        Ok(bytes_transferred)
    }
}

struct OverlappedFuture {
    overlapped: Arc<Overlapped>,
}

impl OverlappedFuture {
    /// Creates a new Overlapped task that can poll Win32 I/O operations.
    pub fn new(overlapped: Arc<Overlapped>) -> OverlappedFuture {
        OverlappedFuture {
            overlapped: overlapped,
        }
    }
}

impl Future for OverlappedFuture {
    type Output = io::Result<u32>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
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
        }
    }
}