use winapi::{
    shared::minwindef::FALSE,
    um::{ioapiset::GetOverlappedResult, minwinbase::OVERLAPPED},
};

use super::event::Event;
use super::handle::Handle;

use std::fmt;
use std::io;
use std::mem;

pub struct Overlapped {
    ovl: OVERLAPPED,
    event: Event,
}

impl fmt::Debug for Overlapped {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Overlapped")
            .field("ovl", &"OVERLAPPED")
            .field("event", &self.event)
            .finish()
    }
}

unsafe impl Send for Overlapped {}
unsafe impl Sync for Overlapped {}

impl Overlapped {
    pub fn new() -> io::Result<Overlapped> {
        let event = Event::new()?;
        let mut ovl: OVERLAPPED = unsafe { mem::zeroed() };
        ovl.hEvent = event.handle.value;
        Ok(Overlapped {
            ovl: ovl,
            event: event,
        })
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.event.reset()?;
        self.ovl = unsafe { mem::zeroed() };
        self.ovl.hEvent = self.event.handle.value;
        Ok(())
    }

    pub fn get_mut(&mut self) -> &mut OVERLAPPED {
        &mut self.ovl
    }

    pub async fn await(self) -> io::Result<()> {
        await!(self.event.await())?;

        Ok(())
    }

    pub async fn await_bytes_transferred(mut self, handle: &Handle) -> io::Result<u32> {
        await!(self.event.await())?;

        let mut bytes_transferred: u32 = 0;

        unsafe { GetOverlappedResult(handle.value, &mut self.ovl, &mut bytes_transferred, FALSE) };

        Ok(bytes_transferred)
    }
}
