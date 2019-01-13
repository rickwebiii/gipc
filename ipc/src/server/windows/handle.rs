use winapi::{
  um::{
    handleapi::{
      CloseHandle,
    },
    winnt::{
      HANDLE
    }
  }
};

#[derive(Debug)]
pub struct Handle {
    pub value: HANDLE,
}

impl Drop for Handle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.value) };
    }
}

unsafe impl Sync for Handle {}
unsafe impl Send for Handle {}