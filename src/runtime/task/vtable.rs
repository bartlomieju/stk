use crate::runtime::task;

use std::future::Future;

pub(crate) struct VTable {
    pub(super) poll: fn(&task::Header),
}

impl VTable {
    pub(crate) fn for_future<T: Future>() -> &'static VTable {
        &VTable { poll: poll::<T> }
    }
}

fn poll<T: Future>(task: &task::Header) {
    unsafe { task::Harness::<T>::from_header_ref(task) }.poll();
}
