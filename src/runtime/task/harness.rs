use crate::runtime::task::waker::waker_ref;
use crate::runtime::task::Header;

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Task harness
#[repr(C)]
pub(crate) struct Harness<T: Future> {
    header: Header,
    state: RefCell<State<T>>,
}

enum State<T: Future> {
    // The future is not yet complete
    InProgress(T),

    // The future is complete and we have the output
    Complete(T::Output),

    // The future output has been consumed.
    Joined,
}

impl<T: Future> Harness<T> {
    pub(crate) fn new(header: Header, future: T) -> Harness<T> {
        Harness {
            header,
            state: RefCell::new(State::InProgress(future)),
        }
    }

    pub(crate) unsafe fn from_header_ref(header: &Header) -> &Harness<T> {
        &*(header as *const _ as *const Harness<T>)
    }

    pub fn poll(&self) {
        use State::*;

        // Build the waker
        let waker = waker_ref::<T>(&self.header);
        let mut cx = Context::from_waker(&waker);

        let mut state = self.state.borrow_mut();

        let future = match &mut *state {
            InProgress(future) => future,
            _ => panic!("invalid task state"),
        };

        // Safety: we don't move the future until it is dropped.
        let future = unsafe { Pin::new_unchecked(future) };

        // Poll the future
        match future.poll(&mut cx) {
            Poll::Ready(_) => {
                println!("TODO: handle join output");
                *state = Joined;
            }
            Poll::Pending => println!("pending"),
        }
    }
}
