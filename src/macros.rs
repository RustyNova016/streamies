/// Extracts the successful type of a `Poll<T>`.
///
/// This macro bakes in propagation of `Pending` signals by returning early.
///
/// It also early return `None` value
#[macro_export]
macro_rules! ready_some {
    ($e:expr $(,)?) => {
        match futures::ready!($e) {
            Some(t) => t,
            None => return futures::task::Poll::Ready(None),
        }
    };
}
