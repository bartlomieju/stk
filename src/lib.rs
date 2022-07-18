mod future;
pub mod net;
pub mod runtime;
pub mod task;

use runtime::Handle;
use task::JoinHandle;

use std::future::Future;

pub fn spawn<T>(task: T) -> JoinHandle<T::Output>
where
    T: Future + 'static,
    T::Output: 'static,
{
    Handle::with_current(|handle| handle.scheduler().spawn(task))
}
