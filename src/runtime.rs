mod driver;
use driver::Driver;

pub(crate) mod io;

mod scheduler;
use scheduler::Scheduler;

pub(crate) mod task;
use task::{JoinHandle, Task};

use std::cell::RefCell;

use std::future::Future;
use std::rc::Rc;

pub struct Runtime {
    handle: Handle,
}

/// Runtime handle
#[derive(Clone)]
pub struct Handle {
    inner: Rc<Inner>,
}

struct Inner {
    /// Executes tasks
    scheduler: Scheduler,

    /// Receives events from the OS and dispatches them.
    io: io::Handle,

    /// Holds scheduler & io state used to drive the runtime forward
    driver: RefCell<Driver>,
}

thread_local!(static CURRENT: RefCell<Option<Handle>> = RefCell::new(None));

impl Runtime {
    /// Create a new runtime
    pub fn new() -> std::io::Result<Runtime> {
        let (io_driver, io_handle) = io::driver()?;
        let driver = Driver::new(io_driver);

        Ok(Runtime {
            handle: Handle {
                inner: Rc::new(Inner {
                    scheduler: Scheduler::new(),
                    io: io_handle,
                    driver: RefCell::new(driver),
                }),
            },
        })
    }

    /// Spawn a task on the runtime
    pub fn spawn<T>(&self, task: T) -> JoinHandle<T::Output>
    where
        T: Future + 'static,
        T::Output: 'static,
    {
        self.handle.spawn(task)
    }

    /// Block on a future
    pub fn block_on<T: Future>(&self, task: T) -> T::Output {
        todo!();
    }

    /// Execute the runtime until all tasks have completed
    pub fn run(&self) {
        let mut driver = self.handle.inner.driver.borrow_mut();

        CURRENT.with(|c| {
            *c.borrow_mut() = Some(self.handle.clone());
        });

        self.handle.inner.scheduler.run(&self.handle, &mut driver);

        CURRENT.with(|c| {
            *c.borrow_mut() = None;
        });
    }
}

impl Handle {
    /// Spawn a task on the runtime
    pub fn spawn<T>(&self, task: T) -> JoinHandle<T::Output>
    where
        T: Future + 'static,
        T::Output: 'static,
    {
        self.inner.scheduler.spawn(task)
    }

    pub(crate) fn scheduler(&self) -> &Scheduler {
        &self.inner.scheduler
    }

    /// Returns a reference to the I/O handle
    pub(crate) fn io(&self) -> &io::Handle {
        &self.inner.io
    }

    #[track_caller]
    pub(crate) fn with_current<R>(f: impl FnOnce(&Handle) -> R) -> R {
        CURRENT.with(|current| {
            let current = current.borrow();
            let current = current.as_ref().expect("called from outside of runtime");
            f(current)
        })
    }
}
