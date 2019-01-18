use winapi::{
    shared::{
        minwindef::{FALSE},
        winerror::{ERROR_ABANDONED_WAIT_0}
    },
    um::{
        ioapiset::{
            CreateIoCompletionPort,
            GetQueuedCompletionStatus,
        },
        minwinbase::{
            OVERLAPPED
        },
        winbase::{
            INFINITE
        }
    }
};

use super::handle::{Handle};
use super::overlapped::{Overlapped, OverlappedCompletionInfo};

use std::mem;
use std::ptr;
use std::sync::{Arc};
use std::sync::atomic;
use std::sync::atomic::{Ordering};

pub struct CompletionPort {
    handle: Handle
}

impl CompletionPort {
    pub fn new(file_handle: &Handle) -> std::io::Result<CompletionPort> {
        let iocp_handle = unsafe {
            CreateIoCompletionPort(
                file_handle.value,
                ptr::null_mut(),
                0,
                1
            )
        };

        if iocp_handle == ptr::null_mut() {
            return Err(std::io::Error::last_os_error());
        }

        Ok(CompletionPort {
            handle: Handle::new(iocp_handle)
        })
    }

    pub fn get_completion_status(&self) -> Arc<Overlapped> {
        let mut bytes_transferred: u32 = 0;
        let mut dummy: usize = 0;
        let mut overlapped: *mut OVERLAPPED = ptr::null_mut();

        let result = unsafe {
            GetQueuedCompletionStatus(
                self.handle.value,
                &mut bytes_transferred,
                &mut dummy,
                &mut overlapped as *mut *mut OVERLAPPED,
                INFINITE
            )
        };

        let overlapped_coerced: *mut Overlapped = unsafe { mem::transmute(overlapped) };
        let mut raw_err = 0 as i32;

        if result == FALSE {
            let err = std::io::Error::last_os_error();
            raw_err = err.raw_os_error().unwrap_or_default();

            // If we get this, then we have a serious problem. The IOCP has been freed while we
            // were waiting and we're now leaking overlappeds. Panic.
            if raw_err == ERROR_ABANDONED_WAIT_0 as i32 {
                panic!("I/O Completion Queue freed while waiting on event. This is a bug.");
            }
        }

        unsafe {
            (*overlapped_coerced).set_completion_info(OverlappedCompletionInfo {
                error: raw_err,
                bytes_transferred: bytes_transferred
            });

            atomic::fence(Ordering::SeqCst);

            Arc::from_raw(overlapped_coerced)
        }
    }
}