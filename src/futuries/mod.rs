use core::future::ready;
use core::future::Future;
use core::future::Ready;

use crate::futuries::future_result::FutureResult;

pub mod future_result;

pub trait ResultExt<T, E>: Sized {
    /// Extract the contained futures into a future that return a result.
    ///
    /// # Example
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use core::future::{Ready, ready};
    /// use streamies::futuries::ResultExt as _;
    ///
    /// let foo = Ok::<Ready<i64>, Ready<String>>(ready(5)).extract_future();
    /// assert_eq!(foo.await, Ok(5));
    /// # })
    /// ```
    fn extract_future(self) -> FutureResult<T, E>
    where
        T: Future,
        E: Future;

    /// Extract the future of the `Ok` value out of the result.
    ///
    /// # Example
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use core::future::{Ready, ready};
    /// use streamies::futuries::ResultExt as _;
    ///
    /// let foo = Ok::<Ready<i64>, String>(ready(5)).extract_future_ok();
    /// assert_eq!(foo.await, Ok(5));
    ///
    /// let bar = Err::<Ready<i64>, String>("Hello there!".to_string()).extract_future_ok();
    /// assert_eq!(bar.await, Err("Hello there!".to_string()));
    /// # })
    /// ```
    fn extract_future_ok(self) -> FutureResult<T, Ready<E>>
    where
        T: Future;

    /// Extract the future of the `Err` value out of the result.
    ///
    /// # Example
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use core::future::{Ready, ready};
    /// use streamies::futuries::ResultExt as _;
    ///
    /// let foo = Ok::<i64, Ready<String>>(5).extract_future_err();
    /// assert_eq!(foo.await, Ok(5));
    ///
    /// let bar = Err::<i64, Ready<String>>(ready("Hello there!".to_string())).extract_future_err();
    /// assert_eq!(bar.await, Err("Hello there!".to_string()));
    /// # })
    /// ```
    fn extract_future_err(self) -> FutureResult<Ready<T>, E>
    where
        E: Future;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn extract_future(self) -> FutureResult<T, E>
    where
        T: Future,
        E: Future,
    {
        match self {
            Ok(v) => FutureResult::ok(v),
            Err(v) => FutureResult::err(v),
        }
    }

    fn extract_future_ok(self) -> FutureResult<T, Ready<E>>
    where
        T: Future,
    {
        match self {
            Ok(v) => FutureResult::ok(v),
            Err(v) => FutureResult::err(ready(v)),
        }
    }

    fn extract_future_err(self) -> FutureResult<Ready<T>, E>
    where
        E: Future,
    {
        match self {
            Ok(v) => FutureResult::ok(ready(v)),
            Err(v) => FutureResult::err(v),
        }
    }
}
