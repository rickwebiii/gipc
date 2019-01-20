use futures::Future;
use futures::task::{LocalWaker, Poll, AtomicWaker};
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

use super::completion_port::CompletionPort;

use std::io;
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic;
use std::sync::atomic::Ordering;
use std::thread;

#[repr(C)]
#[derive(Debug)]
pub struct OverlappedCompletionInfo {
    pub error: i32,
    pub bytes_transferred: u32,
}

#[repr(C)]
pub struct Overlapped {
    overlapped: OVERLAPPED,
    waker: Arc<AtomicWaker>,
    completion_info: Option<OverlappedCompletionInfo>
}

unsafe impl Sync for Overlapped {}
unsafe impl Send for Overlapped {}

impl Overlapped {
    pub fn new() -> io::Result<(Arc<Overlapped>, OverlappedAwaiter)> {
        let mut overlapped: OVERLAPPED = unsafe { mem::zeroed() };
        
        let waker = Arc::new(AtomicWaker::new());

        let overlapped_wrapper = Arc::new(Overlapped {
            overlapped: overlapped,
            waker: waker,
            completion_info: None
        });

        let overlapped_awaiter = OverlappedAwaiter {
            waker: overlapped_wrapper.waker.clone(),
            overlapped: overlapped_wrapper.clone()
        };

        Ok((overlapped_wrapper, overlapped_awaiter))
    }

    pub fn set_completion_info(&mut self, info: OverlappedCompletionInfo) {
        self.completion_info = Some(info);
    }

    pub fn get_completion_info<'a>(&'a self) -> &'a Option<OverlappedCompletionInfo> {
        &self.completion_info
    }

    pub fn get_waker<'a>(&'a self) -> &'a Arc<AtomicWaker> {
        &self.waker
    }
}

pub struct OverlappedAwaiter {
    waker: Arc<AtomicWaker>,
    overlapped: Arc<Overlapped>,
}

impl OverlappedAwaiter {
    pub async fn await(self) -> io::Result<u32> {
        let bytes_transferred = await!(OverlappedFuture::new(self.overlapped))?;

        Ok(bytes_transferred)
    }
}


struct OverlappedFuture {
    overlapped: Arc<Overlapped>,
}

impl OverlappedFuture {
    pub fn new(overlapped: Arc<Overlapped>) -> OverlappedFuture {
        OverlappedFuture {
            overlapped: overlapped,
        }
    }
}

impl Future for OverlappedFuture {
    type Output = io::Result<u32>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        self.overlapped.waker.register(lw);

        // Guarantee we never perceive the iocp thread waking us before it's written its
        // results.
        atomic::fence(Ordering::Release);

        match &self.overlapped.completion_info {
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