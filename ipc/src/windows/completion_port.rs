use winapi::{
    shared::{
        minwindef::{FALSE},
        winerror::{ERROR_ABANDONED_WAIT_0}
    },
    um::{
        handleapi::{
            CloseHandle,
            INVALID_HANDLE_VALUE
        },
        ioapiset::{
            CreateIoCompletionPort,
            GetQueuedCompletionStatus,
        },
        minwinbase::{
            OVERLAPPED
        },
        winbase::{
            INFINITE,
            SetFileCompletionNotificationModes,
        },
        winnt::{
            HANDLE
        }
    }
};

use super::handle::{Handle};
use super::overlapped::{Overlapped, OverlappedCompletionInfo};

use std::io::ErrorKind;
use std::mem;
use std::ptr;
use std::sync::{Arc, Once};
use std::sync::atomic;
use std::sync::atomic::{Ordering};
use std::thread;

pub struct CompletionPort {
    handle: Handle
}

// Not sure why this isn't defined in the winapi crate. Its value comes from
// https://docs.microsoft.com/en-us/windows/desktop/api/winbase/nf-winbase-setfilecompletionnotificationmodes
const FILE_SKIP_COMPLETION_PORT_ON_SUCCESS: u8 = 0x1;

static mut COMPLETION_PORT_SINGLETON: Option<CompletionPort> = None;
static COMPLETION_THREAD_INIT: Once = Once::new();

impl CompletionPort {
    /// Gets the singleton IO completion port. If it hasn't been created yet, this function will
    /// create it and the thread that watches it for events.
    pub fn get() -> std::io::Result<&'static CompletionPort> {
        let mut error: Option<std::io::Error> = None;

        unsafe {
            COMPLETION_THREAD_INIT.call_once(|| {
                let result = CompletionPort::new();

                match result {
                    Ok(port) => { COMPLETION_PORT_SINGLETON = Some(port) },
                    Err(err) => { error = Some(err); return (); }
                };

                let result = thread::Builder::new().name("IO Completion Thread".to_owned()).spawn(move || {
                    loop {
                        let overlapped = CompletionPort::get()
                            .expect("Couldn't get I/O completion port. Named pipes won't work.")
                            .get_completion_status();

                        overlapped.get_waker().wake();
                    }
                });

                if let Err(err) = result {
                    error = Some(err);
                }
            });
        
            match error {
                Some(err) => Err(err),
                None => {
                    match &COMPLETION_PORT_SINGLETON {
                        Some(iocp) => Ok(iocp),
                        None => Err(std::io::Error::from(ErrorKind::Other))
                    }
                }
            }
        }
    }

    /// Creates a Win32 I/O completion port not associated with any particular file HANDLE. Use
    /// add_file_handle to do so.
    fn new() -> std::io::Result<CompletionPort> {
        let iocp_handle = CompletionPort::associate_completion_port(None, INVALID_HANDLE_VALUE)?;

        Ok(CompletionPort {
            handle: Handle::new(iocp_handle)
        })
    }

    /// Associates a new file handle with this I/O completion port.
    pub fn add_file_handle(&self, file_handle: &Handle) -> std::io::Result<()> {
        let handle = CompletionPort::associate_completion_port(Some(self.handle.value), file_handle.value)?;

        assert_eq!(handle, self.handle.value);

        Ok(())
    }

    /// Associates an iocp with the passed file handle. If existing_iocp is None, then the returned iocp HANDLE
    /// will be to a new completion port. If file_handle is INVALID_HANDLE_VALUE, then this creates an iocp
    /// not associated with any file (which can be later associated with files). In this case, existing_iocp
    /// must be None or you'll get an error.
    fn associate_completion_port(existing_iocp: Option<HANDLE>, file_handle: HANDLE) -> std::io::Result<HANDLE> {
        let existing_iocp = existing_iocp.unwrap_or(ptr::null_mut());

        let iocp_handle = unsafe {
            // https://docs.microsoft.com/en-us/windows/desktop/FileIO/createiocompletionport
            CreateIoCompletionPort(
                file_handle,
                existing_iocp,
                0,
                1
            )
        };

        if iocp_handle == ptr::null_mut() {
            return Err(std::io::Error::last_os_error());
        }

        // By default, files will notify the completion port even when the function generating the operation
        // completes synchronously. Disable that mechanism, as read and write already handle that case
        // explicitly and won't await the event we try to send.
        if file_handle != INVALID_HANDLE_VALUE  {
            let result = unsafe {
                // https://docs.microsoft.com/en-us/windows/desktop/api/winbase/nf-winbase-setfilecompletionnotificationmodes
                SetFileCompletionNotificationModes(
                    file_handle,
                    FILE_SKIP_COMPLETION_PORT_ON_SUCCESS
                )
            };

            if result == FALSE {
                // Ideally we'd wrap the HANDLE in a Handle that would manage its lifetime. However,
                // if we're attaching to an existing iocp, the returned handle is the existing iocp's
                // HANDLE, which we don't own. If we wrapped in a Handle, there are now 2 Boxed references
                // to the same HANDLE and either one dropping will free the other, which is wrong. So,
                // we don't wrap in a Handle and manage cleanup manually.
                let _ = unsafe { CloseHandle(iocp_handle) };
                return Err(std::io::Error::last_os_error());
            }
        }      

        Ok(iocp_handle)
    }

    /// Blocks the current thread on the completion port until an I/O operation completes, and then populates the
    /// results into 
    fn get_completion_status(&self) -> Arc<Overlapped> {
        let mut bytes_transferred: u32 = 0;
        let mut dummy: usize = 0;
        let mut overlapped: *mut OVERLAPPED = ptr::null_mut();

        let result = unsafe {
            // https://msdn.microsoft.com/en-us/library/Aa364986(v=VS.85).aspx
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

            // Nobody should be closing the completion port singleton, which results in this error.
            if raw_err == ERROR_ABANDONED_WAIT_0 as i32 {
                panic!("Somebody closed the completion port singleton. This is a bug.");
            }
        }

        unsafe {
            (*overlapped_coerced).set_completion_info(OverlappedCompletionInfo {
                error: raw_err,
                bytes_transferred: bytes_transferred
            });

            // The intent is for get_completion_status function to be called from a different thread than the one 
            // awaiting the I/O event. As such, we need to guarantee the awaiting threads see the set_completion_info
            // before this thread calls the waker.
            atomic::fence(Ordering::Acquire);

            Arc::from_raw(overlapped_coerced)
        }
    }
}