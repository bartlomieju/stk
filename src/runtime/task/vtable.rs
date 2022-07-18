use crate::runtime::{task, Scheduler};

use std::future::Future;
use std::task::{RawWaker, RawWakerVTable};

pub(crate) struct VTable {
    /// Poll the future
    pub(super) poll: fn(scheduler: &Scheduler, &task::Header),

    /// Waker ref VTable
    pub(super) waker_ref: &'static RawWakerVTable,
}

impl VTable {
    pub(crate) fn for_future<T: Future>() -> &'static VTable {
        &VTable {
            poll: poll::<T>,
            waker_ref: &RawWakerVTable::new(
                clone_waker::<T>,
                wake_by_val::<T>,
                wake_by_ref::<T>,
                drop_waker::<T>,
            ),
        }
    }
}

fn poll<T: Future>(scheduler: &Scheduler, task: &task::Header) {
    unsafe { task::Harness::<T>::from_header_ref(task) }.poll(scheduler);
}

unsafe fn clone_waker<T>(ptr: *const ()) -> RawWaker
where
    T: Future,
{
    todo!()
}

unsafe fn drop_waker<T>(ptr: *const ())
where
    T: Future,
{
    todo!()
}

unsafe fn wake_by_val<T>(ptr: *const ())
where
    T: Future,
{
    todo!()
}

// Wake without consuming the waker
unsafe fn wake_by_ref<T>(ptr: *const ())
where
    T: Future,
{
    todo!()
}
