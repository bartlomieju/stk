use crate::runtime::task::waker::waker_ref;
use crate::runtime::task::Header;

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Task harness
#[repr(C)]
pub(crate) struct Harness<T> {
    header: Header,
    future: RefCell<T>,
}

impl<T: Future> Harness<T> {
    pub(crate) fn new(header: Header, future: T) -> Harness<T> {
        Harness {
            header,
            future: RefCell::new(future),
        }
    }

    pub(crate) unsafe fn from_header_ref(header: &Header) -> &Harness<T> {
        &*(header as *const _ as *const Harness<T>)
    }

    pub fn poll(&self) {
        // Build the waker
        let waker = waker_ref::<T>(&self.header);
        let mut cx = Context::from_waker(&waker);

        let mut future = self.future.borrow_mut();
        let future = &mut *future;

        // Safety: we don't move the future until it is dropped.
        let future = unsafe { Pin::new_unchecked(future) };

        // Poll the future
        match future.poll(&mut cx) {
            Poll::Ready(_) => println!("DONE"),
            Poll::Pending => println!("pending"),
        }
    }
}
