use crate::runtime::Task;

use std::future::Future;

pub(crate) struct VTable {
    poll: fn (Task),
}

impl VTable {
    pub(crate) fn for_future<T: Future>() -> &'static VTable {
        &VTable {
            poll: poll::<T>,
        }
    }
}

fn poll<T: Future>(task: Task) {
    todo!()
}