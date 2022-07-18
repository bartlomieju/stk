use crate::runtime::task::Header;

use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops;
use std::task::Waker;

pub(crate) struct WakerRef<'a> {
    waker: ManuallyDrop<Waker>,
    _p: PhantomData<&'a Header>,
}

pub(super) fn waker_ref(header: &Header) -> WakerRef<'_> {
    let raw = header.raw_waker();
    let waker = unsafe { Waker::from_raw(raw) };

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
