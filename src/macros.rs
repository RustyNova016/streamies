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

/// This macro extract the value from a `Poll::Ready(Some(Ok(_)))`
///
/// If the value doesn't correspond to it, early return the value. Aka:
/// - Poll::Pending
/// - Poll::Ready(None)
/// - Poll::Ready(Some(Err(_)))
#[macro_export]
macro_rules! ready_some_ok {
    ($e:expr $(,)?) => {
        match ready_some!($e) {
            Ok(t) => t,
            Err(err) => return futures::task::Poll::Ready(Some(Err(err))),
        }
    };
}
