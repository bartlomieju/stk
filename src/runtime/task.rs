mod harness;
use harness::Harness;

mod header;
use header::Header;

mod vtable;
use vtable::VTable;

mod waker;

use std::future::Future;
use std::ptr::NonNull;

pub struct JoinHandle<T> {
    _p: std::marker::PhantomData<T>,
}

pub(crate) struct Task {
    header: NonNull<Header>,
}

pub(crate) fn spawn<T: Future>(future: T) -> (Task, JoinHandle<T::Output>) {
    let header = Header::new::<T>();

    let harness = Box::new(Harness::new(header, future));
    let harness = Box::into_raw(harness);
    let header = unsafe { NonNull::new_unchecked(harness as *mut Header) };

    let task = Task { header };
    let handle = JoinHandle {
        _p: std::marker::PhantomData,
    };

    (task, handle)
}

impl Task {
    pub(crate) fn poll(&self) {
        self.header().poll();
    }

    fn header(&self) -> &Header {
        unsafe { self.header.as_ref() }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        // TODO: don't leak
    }
}
