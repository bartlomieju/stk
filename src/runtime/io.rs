mod interest;
pub use interest::Interest;

mod ready;
pub use ready::Ready;

use crate::runtime::{self, Scheduler, Task};

use mio::event::Source;
use mio::Token;
use slab::Slab;
use std::cell::{Cell, RefCell};
use std::io;
use std::rc::Rc;

pub(crate) use std::io::Result;

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
    /// Handle to the runtime
    ///
    /// TODO: break cycle
    rt: runtime::Handle,

    /// Which slot in the slab this resource is stored in.
    key: usize,

    /// Current resource readiness
    readiness: Cell<Ready>,

    /// Task to schedule on readable
    read_task: RefCell<Option<Task>>,

    /// Task to schedule on writable
    write_task: RefCell<Option<Task>>,
}

pub(crate) struct Registration {
    resource: Rc<Resource>,
}

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
        rt: &runtime::Handle,
        io: &mut impl Source,
        interest: Interest,
    ) -> io::Result<Registration> {
        // Reserve a new slot for the new resource. We will be using the key as
        // the mio token.
        let mut resources = self.resources.borrow_mut();
        let entry = resources.vacant_entry();

        // Register the socket with mio
        self.mio
            .register(io, Token(entry.key()), interest.to_mio())?;

        let resource = Rc::new(Resource {
            rt: rt.clone(),
            key: entry.key(),
            readiness: Cell::new(Ready::EMPTY),
            read_task: RefCell::new(None),
            write_task: RefCell::new(None),
        });

        entry.insert(resource.clone());

        Ok(Registration { resource })
    }
}

impl Driver {
    pub(crate) fn park(&mut self, handle: &Handle, scheduler: &Scheduler) -> io::Result<()> {
        match self.mio.poll(&mut &mut self.events, None) {
            Ok(_) => {}
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }

        let resources = handle.resources.borrow();

        for event in self.events.iter() {
            let resource = match resources.get(event.token().0) {
                Some(resource) => resource,
                None => continue,
            };

            resource.add_readiness(scheduler, Ready::from_mio(event));
        }

        Ok(())
    }
}

impl Registration {
    /// Wait for an I/O readiness event
    pub(crate) async fn readiness(&self, interest: Interest) -> io::Result<Ready> {
        use std::task::Poll;

        crate::future::poll_fn(|cx| {
            let ready = self.resource.readiness.get();
            let ready = ready.intersection(interest);

            if ready.is_empty() {
                if interest.is_readable() {
                    // Get the runtime task associated with the current waker
                    let task = self
                        .resource
                        .rt
                        .scheduler()
                        .waker_to_task(cx.waker())
                        .expect("TODO");

                    *self.resource.read_task.borrow_mut() = Some(task);
                }

                if interest.is_writable() {
                    let task = self
                        .resource
                        .rt
                        .scheduler()
                        .waker_to_task(cx.waker())
                        .expect("TODO");
                    *self.resource.write_task.borrow_mut() = Some(task);
                }

                return Poll::Pending;
            }

            Poll::Ready(Ok(ready))
        })
        .await
    }

    pub(crate) fn clear_readiness(&self, ready: Ready) {
        self.resource
            .readiness
            .set(self.resource.readiness.get() - ready);
    }

    pub(crate) async fn async_io<R>(
        &self,
        interest: Interest,
        mut f: impl FnMut() -> io::Result<R>,
    ) -> io::Result<R> {
        loop {
            let ready = self.readiness(interest).await?;

            match f() {
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    self.clear_readiness(ready);
                }
                x => return x,
            }
        }
    }
}

impl Resource {
    // Called by the I/O driver
    pub(crate) fn add_readiness(&self, scheduler: &Scheduler, ready: Ready) {
        let old = self.readiness.get();
        let add = ready - old;

        self.readiness.set(old | ready);

        if add.is_readable() {
            if let Some(task) = self.read_task.borrow_mut().take() {
                scheduler.schedule(task);
            }
        }

        if add.is_writable() {
            if let Some(task) = self.write_task.borrow_mut().take() {
                scheduler.schedule(task);
            }
        }
    }
}
