pub(crate) mod task;
use task::{JoinHandle, Task};

mod vtable;
use vtable::VTable;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::rc::Rc;

pub struct Runtime {
    handle: Handle,
}

/// Runtime handle
pub struct Handle {
    inner: Rc<Inner>,
}

struct Inner {
    queue: RefCell<VecDeque<Task>>,
}

thread_local!(static CURRENT: RefCell<Option<Handle>> = RefCell::new(None));

const INITIAL_QUEUE_CAPACITY: usize = 256;

impl Runtime {
    /// Create a new runtime
    pub fn new() -> Runtime {
        Runtime {
            handle: Handle {
                inner: Rc::new(Inner {
                    queue: RefCell::new(VecDeque::with_capacity(INITIAL_QUEUE_CAPACITY)),
                }),
            },
        }
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
        loop {
            let mut task = match self.next_scheduled_task() {
                Some(task) => task,
                None => return,
            };

            task.poll();
        }
    }

    /// Return the next scheduled task
    fn next_scheduled_task(&self) -> Option<Task> {
        self.handle.inner.queue.borrow_mut().pop_front()
    }
}

impl Handle {
    /// Spawn a task on the runtime
    pub fn spawn<T>(&self, task: T) -> JoinHandle<T::Output>
    where
        T: Future + 'static,
        T::Output: 'static,
    {
        // Create the task harness
        let (task, handle) = task::spawn(task);

        // Schedule the task for execution
        self.inner.queue.borrow_mut().push_back(task);

        // Return the join handle
        handle
    }
}