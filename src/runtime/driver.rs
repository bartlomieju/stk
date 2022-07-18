use crate::runtime::{io, Handle, Scheduler};

pub(crate) struct Driver {
    io: io::Driver,
}

impl Driver {
    pub(crate) fn new(io: io::Driver) -> Driver {
        Driver { io }
    }

    pub(crate) fn park(&mut self, handle: &Handle, scheduler: &Scheduler) -> io::Result<()> {
        self.io.park(handle.io(), scheduler)
    }
}
