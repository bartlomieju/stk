use crate::runtime::Task;

use mio::event::Source;
use mio::{Interest, Token};
use slab::Slab;
use std::cell::RefCell;
use std::io;
use std::rc::Rc;

pub(crate) struct Handle {
    /// Used to register new sockets w/ epoll
    mio: mio::Registry,

    /// Tracks state for open sockets and other resources
    resources: RefCell<Slab<Rc<Resource>>>,
}

/// Used by the runtime to process I/O events
pub(crate) struct Driver {
    /// The system event queue
    mio: mio::Poll,

    /// Used to receive events from `Poll`
    events: mio::Events,
}

pub(crate) struct Resource {
    /// Which slot in the slab this resource is stored in.
    key: usize,

    /// Task to schedule on readable
    read_task: RefCell<Option<Task>>,
}

pub(crate) struct Registration {
    resource: Rc<Resource>,
}

pub(crate) struct Ready(usize);

const INITIAL_RESOURCES_CAPACITY: usize = 256;
const INITIAL_EVENTS_CAPACITY: usize = 1024;

pub(crate) fn driver() -> io::Result<(Driver, Handle)> {
    let mio = mio::Poll::new()?;

    let handle = Handle {
        mio: mio.registry().try_clone()?,
        resources: RefCell::new(Slab::with_capacity(INITIAL_RESOURCES_CAPACITY)),
    };

    let driver = Driver {
        mio,
        events: mio::Events::with_capacity(INITIAL_EVENTS_CAPACITY),
    };

    Ok((driver, handle))
}

impl Handle {
    pub(crate) fn register(
        &self,
        io: &mut impl Source,
        interest: Interest,
    ) -> io::Result<Registration> {
        // Reserve a new slot for the new resource. We will be using the key as
        // the mio token.
        let mut resources = self.resources.borrow_mut();
        let entry = resources.vacant_entry();

        // Register the socket with mio
        self.mio.register(io, Token(entry.key()), interest)?;

        let resource = Rc::new(Resource {
            key: entry.key(),
            read_task: RefCell::new(None),
        });

        Ok(Registration { resource })
    }
}

impl Driver {}

impl Registration {
    /// Wait for an I/O readiness event
    pub(crate) async fn readiness(&self) -> io::Result<Ready> {
        todo!();
    }

    pub(crate) fn clear_readiness(&self, ready: Ready) {
        todo!();
    }

    pub(crate) async fn async_io<R>(
        &self,
        interest: Interest,
        mut f: impl FnMut() -> io::Result<R>,
    ) -> io::Result<R> {
        loop {
            let ready = self.readiness().await?;

            match f() {
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    self.clear_readiness(ready);
                }
                x => return x,
            }
        }
    }
}
