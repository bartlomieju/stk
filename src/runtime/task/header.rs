use crate::runtime::task::VTable;
use crate::runtime::Scheduler;

use std::future::Future;
use std::task::RawWaker;

#[repr(C)]
pub(crate) struct Header {
    /// Dynamic dispatch to future-specific functions.
    vtable: &'static VTable,
}

impl Header {
    pub(crate) fn new<T: Future>() -> Header {
        Header {
            vtable: VTable::for_future::<T>(),
        }
    }

    pub(crate) fn poll(&self, scheduler: &Scheduler) {
        (self.vtable.poll)(scheduler, self)
    }

    pub(crate) fn raw_waker(&self) -> RawWaker {
        let ptr = self as *const _ as *const ();
        RawWaker::new(ptr, self.vtable.waker_ref)
    }
}
