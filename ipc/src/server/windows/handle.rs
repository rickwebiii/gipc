use log::debug;
use winapi::um::{handleapi::CloseHandle, winnt::HANDLE};

use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

static HANDLE_ID: AtomicUsize = ATOMIC_USIZE_INIT;

#[cfg(debug_assertions)]
static NUM_HANDLES: AtomicUsize = ATOMIC_USIZE_INIT;

#[derive(Debug)]
pub struct Handle {
    pub value: HANDLE,

    #[cfg(debug_assertions)]
    pub id: usize,
}

impl Handle {
    #[cfg(debug_assertions)]
    pub fn new(handle: HANDLE) -> Handle {
        let id = HANDLE_ID.fetch_add(1, Ordering::SeqCst);
        NUM_HANDLES.fetch_add(1, Ordering::SeqCst);
        // debug!("Handle: created {}", id);

        Handle { value: handle, id: id }
    }

    #[cfg(debug_assertions)]
    pub fn id(&self) -> usize {
        self.id
    }

    #[cfg(not(debug_assertions))] 
    pub fn new(handle: HANDLE) -> Handle {
        Handle { value: handle }
    }

    #[cfg(debug_assertions)]
    pub fn num_open_handles() -> usize {
        NUM_HANDLES.load(Ordering::SeqCst)
    }
}

impl Drop for Handle {
    #[cfg(debug_assertions)]
    fn drop(&mut self) {
        // debug!("Handle: closed {}", self.id);

        NUM_HANDLES.fetch_sub(1, Ordering::SeqCst);

        let _ = unsafe { CloseHandle(self.value) };
    }

    #[cfg(not(debug_assertions))]
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.value) };
    }
}

unsafe impl Sync for Handle {}
unsafe impl Send for Handle {}
