use crate::runtime::task::VTable;

use std::future::Future;

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

    pub(crate) fn poll(&self) {
        (self.vtable.poll)(self)
    }
}
