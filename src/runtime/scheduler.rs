use crate::runtime::task::{self, JoinHandle, Task};

use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;

pub(crate) struct Scheduler {
    queue: RefCell<VecDeque<Task>>,
}

const INITIAL_QUEUE_CAPACITY: usize = 256;

impl Scheduler {
    pub(crate) fn new() -> Scheduler {
        Scheduler {
            queue: RefCell::new(VecDeque::with_capacity(INITIAL_QUEUE_CAPACITY)),
        }
    }

    pub(crate) fn run(&self) {
        loop {
            let task = match self.next_scheduled_task() {
                Some(task) => task,
                None => return,
            };

            task.poll();
        }
    }

    pub(crate) fn spawn<T>(&self, task: T) -> JoinHandle<T::Output>
    where
        T: Future + 'static,
        T::Output: 'static,
    {
        // Create the task harness
        let (task, handle) = task::spawn(task);

        // Schedule the task for execution
        self.queue.borrow_mut().push_back(task);

        // Return the join handle
        handle
    }

    /// Return the next scheduled task
    fn next_scheduled_task(&self) -> Option<Task> {
        self.queue.borrow_mut().pop_front()
    }
}