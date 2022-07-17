use crate::runtime::io;

pub(crate) struct Driver {
    io: io::Driver,
}

impl Driver {
    pub(crate) fn new(io: io::Driver) -> Driver {
        Driver { io }
    }
}
