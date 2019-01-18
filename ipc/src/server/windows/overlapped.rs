use futures::Future;
use futures::task::{LocalWaker, Poll, AtomicWaker};
use log::{debug};
use winapi::{
    shared::{
        minwindef::{FALSE, TRUE},
        winerror::{
            ERROR_SUCCESS
        }
    },
    um::{
        ioapiset::GetOverlappedResult, 
        minwinbase::OVERLAPPED,
        synchapi::{WaitForSingleObject},
    },
};

use super::completion_port::CompletionPort;
use super::event::Event;
use super::handle::Handle;

use std::fmt;
use std::io;
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering;
use std::thread;

#[repr(C)]
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
    pub fn new(completion_port: Arc<CompletionPort>) -> io::Result<(Arc<Overlapped>, OverlappedAwaiter)> {
        let event = Event::new()?;
        let mut overlapped: OVERLAPPED = unsafe { mem::zeroed() };
        overlapped.hEvent = event.handle.value;
        
        let waker = Arc::new(AtomicWaker::new());

        let overlapped_wrapper = Arc::new(Overlapped {
            overlapped: overlapped,
            waker: waker,
            completion_info: None
        });

        let overlapped_awaiter = OverlappedAwaiter {
            event: event,
            waker: waker.clone(),
            completion_port: completion_port,
            overlapped: overlapped_wrapper.clone()
        };

        Ok((overlapped_wrapper, overlapped_awaiter))
    }

    pub fn set_completion_info(&mut self, info: OverlappedCompletionInfo) {
        self.completion_info = Some(info);
    }

    pub fn get_completion_info(&self) -> Option<OverlappedCompletionInfo> {
        self.completion_info
    }
}

pub struct OverlappedAwaiter {
    waker: Arc<AtomicWaker>,
    completion_port: Arc<CompletionPort>,
    event: Event,
    overlapped: Arc<Overlapped>,
}

impl OverlappedAwaiter {
    pub async fn await(self) -> io::Result<u32> {
        let bytes_transferred = await!(OverlappedFuture::new(self.overlapped, self.completion_port))?;

        Ok(bytes_transferred)
    }
}


struct OverlappedFuture {
    overlapped: Arc<Overlapped>,
    completion_port: Arc<CompletionPort>,
}

impl OverlappedFuture {
    pub fn new(overlapped: Arc<Overlapped>, completion_port: Arc<CompletionPort>) -> OverlappedFuture {
        OverlappedFuture {
            overlapped: overlapped,
            completion_port: completion_port
        }
    }
}

impl Future for OverlappedFuture {
    type Output = io::Result<u32>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        self.overlapped.waker.register(lw);

        match self.overlapped.completion_info {
            Some(info) => {
                if info.error != ERROR_SUCCESS as i32 {
                    Poll::Ready(Err(std::io::Error::from_raw_os_error(info.error)))
                } else {
                    Poll::Ready(Ok(info.bytes_transferred))
                }                
            },
            None => {
                let completion_port = self.completion_port.clone();

                thread::spawn(move || {
                    let completion = completion_port.get_completion_status();

                    completion.waker.wake();
                });

                Poll::Pending
            }
        }
    }
}