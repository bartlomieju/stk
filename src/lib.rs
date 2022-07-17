pub mod io;
pub mod net;
pub mod runtime;
pub mod task;

use task::JoinHandle;

use std::future::Future;

pub fn spawn<T>(task: T) -> JoinHandle<T::Output>
where
    T: Future + 'static,
    T::Output: 'static,
{
    todo!()
}
