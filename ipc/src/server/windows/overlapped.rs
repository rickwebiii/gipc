use super::event::{Event};

use std::fmt;
use std::io;
use std::mem;

use winapi::{
  um::{
    minwinbase::{
        OVERLAPPED
    },
  }
};


pub struct Overlapped {
    ovl: Box<OVERLAPPED>,
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
        let mut ovl: Box<OVERLAPPED> = Box::new(unsafe { mem::zeroed() });
        ovl.hEvent = event.handle.value;
        Ok(Overlapped {
            ovl: ovl,
            event: event,
        })
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.event.reset()?;
        self.ovl = Box::new(unsafe { mem::zeroed() });
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
}
