use crate::runtime::task::{self, JoinHandle, Task};
use crate::runtime::{Driver, Handle};

use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::task::Waker;

pub(crate) struct Scheduler {
    /// Queue of tasks scheduled to run
    queue: RefCell<VecDeque<Task>>,

    /// Current task
    current: RefCell<Option<Task>>,
}

const INITIAL_QUEUE_CAPACITY: usize = 256;

impl Scheduler {
    pub(crate) fn new() -> Scheduler {
        Scheduler {
            queue: RefCell::new(VecDeque::with_capacity(INITIAL_QUEUE_CAPACITY)),
            current: RefCell::new(None),
        }
    }

    pub(crate) fn run(&self, handle: &Handle, driver: &mut Driver) {
        loop {
            loop {
                let task = match self.next_scheduled_task() {
                    Some(task) => task,
                    None => break,
                };

                self.run_task(task);
            }

            driver.park(handle, self).unwrap();
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

    /// Schedule a task for execution
    pub(crate) fn schedule(&self, task: Task) {
        self.queue.borrow_mut().push_back(task);
    }

    /// Returns the `Task` representing the waker
    pub(crate) fn waker_to_task(&self, waker: &Waker) -> Option<Task> {
        use std::mem::ManuallyDrop;
        // use std::task::Waker;

        let current = self.current.borrow();

        if let Some(ref task) = *current {
            let current_task_raw_waker = task.raw_waker();
            let current_task_waker = unsafe { Waker::from_raw(current_task_raw_waker) };
            let current_task_waker = ManuallyDrop::new(current_task_waker);

            if current_task_waker.will_wake(waker) {
                Some(task.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn run_task(&self, task: Task) {
        self.set_current(&task);

        task.poll(self);

        self.unset_current();
    }

    /// Return the next scheduled task
    fn next_scheduled_task(&self) -> Option<Task> {
        self.queue.borrow_mut().pop_front()
    }

    /// Set the currently running task
    fn set_current(&self, task: &Task) {
        *self.current.borrow_mut() = Some(task.clone());
    }

    fn unset_current(&self) {
        *self.current.borrow_mut() = None;
    }
}
