use crate::runtime::VTable;

use std::future::Future;
use std::ptr::NonNull;

pub struct JoinHandle<T> {
    _p: std::marker::PhantomData<T>,
}

pub(crate) struct Task {
    inner: NonNull<Header>,
}

#[repr(C)]
struct Inner<T> {
    header: Header,
    future: T,
}

#[repr(C)]
struct Header {
    /// Dynamic dispatch to future-specific functions.
    vtable: &'static VTable,
}

pub(crate) fn spawn<T: Future>(future: T) -> (Task, JoinHandle<T::Output>) {
    let inner = Box::new(Inner {
        header: Header {
            vtable: VTable::for_future::<T>(),
        },
        future,
    });

    let inner = Box::into_raw(inner);
    let inner = unsafe { NonNull::new_unchecked(inner as *mut Header) };
    let task = Task { inner };

    let handle = JoinHandle { _p: std::marker::PhantomData, };

    (task, handle)
}

impl Task {
    pub(crate) fn poll(&mut self) {
        todo!();
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        println!("TODO: fix leak");
    }
}
