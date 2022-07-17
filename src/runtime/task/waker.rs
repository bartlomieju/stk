use crate::runtime::task::Header;

use std::future::Future;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops;
use std::task::{RawWaker, RawWakerVTable, Waker};

pub(crate) struct WakerRef<'a> {
    waker: ManuallyDrop<Waker>,
    _p: PhantomData<&'a Header>,
}

pub(super) fn waker_ref<T>(header: &Header) -> WakerRef<'_>
where
    T: Future,
{
    let waker = raw_waker::<T>(header);
    let waker = unsafe { Waker::from_raw(waker) };

    WakerRef {
        waker: ManuallyDrop::new(waker),
        _p: PhantomData,
    }
}

impl ops::Deref for WakerRef<'_> {
    type Target = Waker;

    fn deref(&self) -> &Waker {
        &self.waker
    }
}

fn raw_waker<T>(header: &Header) -> RawWaker
where
    T: Future,
{
    let ptr = header as *const _ as *const ();
    let vtable = &RawWakerVTable::new(
        clone_waker::<T>,
        wake_by_val::<T>,
        wake_by_ref::<T>,
        drop_waker::<T>,
    );
    RawWaker::new(ptr, vtable)
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
