pub mod chunks_ok;
pub mod flatten_ok;
pub mod flatten_result;
use futures::Stream;
use futures::TryStream;
use futures::TryStreamExt;

pub use crate::chunks_ok::ChunksOk;
pub use crate::extract_ok_future::ExtractFutureOk;
pub use crate::flatten_ok::FlattenOk;
use crate::flatten_result::FlattenResult;
pub use crate::try_collect_vec::TryCollectVec;
pub use crate::try_ready_result::ReadyChunksOk;

pub mod extract_ok_future;
pub mod try_collect_vec;
pub mod try_ready_result;

pub trait TryStreamies: TryStream {
    /// Collect the stream into a vec.
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::stream::{self, StreamExt};
    /// use streamies::TryStreamies as _;
    ///
    /// let stream = stream::iter(vec![Ok::<i32, String>(1), Ok(2), Ok(3)]);
    ///
    /// let result = stream.try_collect_vec().await; // No need for ...: Result<Vec<_>, _>!
    /// assert_eq!(result, Ok(vec![1, 2, 3]));
    ///
    /// // However, this will result in an error value
    /// let stream = stream::iter(vec![Ok(1), Err("uh oh"), Ok(3)]);
    ///
    /// let result = stream.try_collect_vec().await;
    /// assert_eq!(result, Err("uh oh"));
    /// # });
    /// ```
    fn try_collect_vec(self) -> TryCollectVec<Self>
    where
        Self: Sized + TryStreamExt,
    {
        TryCollectVec::new(self.try_collect())
    }

    /// An adaptor for chunking up successful, ready items of the stream inside a vector.
    ///
    /// This combinator will attempt to pull successful items from this stream and buffer
    /// them into a local vector. At most `capacity` items will get buffered
    /// before they're yielded from the returned stream. If the underlying stream
    /// returns `Poll::Pending`, and the collected chunk is not empty, it will
    /// be immediately returned.
    ///
    /// Note that the vectors returned from this iterator may not always have
    /// `capacity` elements. If the underlying stream ended and only a partial
    /// vector was created, it'll be returned. Additionally if an error happens
    /// from the underlying stream then the currently buffered items will be
    /// yielded.
    ///
    /// This function is similar to
    /// [`TryReadyChunksError::try_ready_chunks`](futures::stream::TryStreamExt::try_ready_chunks) but
    /// with a key distinction. The stream doesn't return any "error" value containing the
    /// current chunk and the error. Instead it yield the current chunk, and the error will get
    /// yielded on the next poll
    /// This allows easy chaining without having to worry some `Ok` values are put in the `Err`
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::stream::{self, TryStreamExt};
    /// use streamies::TryStreamies as _;
    ///
    /// let stream = stream::iter(vec![Ok::<i32, i32>(1), Ok(2), Ok(3), Err(4), Err(5), Ok(6), Ok(7)]);
    /// let mut stream = stream.ready_chunks_ok(2);
    ///
    /// assert_eq!(stream.try_next().await, Ok(Some(vec![1, 2])));    
    /// assert_eq!(stream.try_next().await, Ok(Some(vec![3])));    // The next value is an error, so couldn't fill the vec
    /// assert_eq!(stream.try_next().await, Err(4));
    /// assert_eq!(stream.try_next().await, Err(5));               // Consecutive errors are yielded 1 by 1
    /// assert_eq!(stream.try_next().await, Ok(Some(vec![6, 7])));
    /// assert_eq!(stream.try_next().await, Ok(None));
    /// # })
    /// ```
    ///
    /// # Panics
    ///
    /// This method will panic if `capacity` is zero.
    fn ready_chunks_ok(self, cap: usize) -> ReadyChunksOk<Self>
    where
        Self: Sized + TryStreamExt,
    {
        ReadyChunksOk::new(self, cap)
    }

    /// Extract the future of the `Ok` value out of the result.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use core::future::{Ready, ready};
    /// use futures::stream::{self, StreamExt, TryStreamExt};
    /// use streamies::TryStreamies as _;
    ///
    /// let stream = stream::iter(vec![Ok::<Ready<i32>, i32>(ready(1)), Ok(ready(2)), Err(3)]);
    /// let mut stream = stream.extract_future_ok().buffer_unordered(1);
    ///
    /// assert_eq!(stream.next().await.unwrap(), Ok(1));    
    /// assert_eq!(stream.next().await.unwrap(), Ok(2));  
    /// assert_eq!(stream.next().await.unwrap(), Err(3));  
    /// # })
    /// ```
    fn extract_future_ok(self) -> ExtractFutureOk<Self>
    where
        Self: Sized + TryStreamExt,
    {
        ExtractFutureOk::new(self)
    }

    /// Flattens a stream of streams into just one continuous stream.
    ///
    /// Values yielded by the inner streams will get assigned to `Ok` values,
    /// while `Err` values will pass through
    ///
    /// The difference between this combinator and [`try_flatten`](futures::stream::TryStreamExt::try_flatten)
    /// is that it doesn't flatten `Err` values if the inner stream return
    /// `Result`s
    ///
    /// # Examples
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use streamies::TryStreamies as _;
    /// use futures::stream::{self, StreamExt, TryStreamExt};
    /// use std::thread;
    ///
    /// let foo = stream::iter(vec![1, 2 ,3]);
    /// let bar = stream::iter(vec![5, 6]);
    /// let mut baz = stream::iter(vec![Ok(foo), Err(4), Ok(bar)])
    ///     .flatten_ok();
    ///
    /// assert_eq!(baz.next().await, Some(Ok(1)));
    /// assert_eq!(baz.next().await, Some(Ok(2)));
    /// assert_eq!(baz.next().await, Some(Ok(3)));
    /// assert_eq!(baz.next().await, Some(Err(4)));
    /// assert_eq!(baz.next().await, Some(Ok(5)));
    /// assert_eq!(baz.next().await, Some(Ok(6)));
    /// assert_eq!(baz.next().await, None);
    /// # });
    /// ```
    fn flatten_ok(self) -> FlattenOk<Self>
    where
        Self::Ok: Stream,
        Self: Sized,
    {
        FlattenOk::new(self)
    }

    /// An adaptor for chunking up items of the stream inside a vector.
    ///
    /// This combinator will attempt to pull `Ok` items from this stream and buffer
    /// them into a local vector. At most `capacity` items will get buffered
    /// before they're yielded from the returned stream.
    ///
    /// Encountering an `Err` value result in a early return of the storead chunk,
    /// then the error in the next poll
    ///
    /// Note that the vectors returned from this iterator may not always have
    /// `capacity` elements. If the underlying stream ended and only a partial
    /// vector was created, it'll be returned. Additionally if an error happens
    /// from the underlying stream then the currently buffered items will be
    /// yielded.
    ///
    /// # Panics
    ///
    /// This method will panic if `capacity` is zero.
    ///
    /// # Example
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::stream::{self, TryStreamExt};
    /// use streamies::TryStreamies as _;
    ///
    /// let stream = stream::iter(vec![Ok::<i32, i32>(1), Ok(2), Ok(3), Err(4), Err(5), Ok(6)]);
    /// let mut stream = stream.chunks_ok(2);
    ///
    /// assert_eq!(stream.try_next().await, Ok(Some(vec![1, 2])));    
    /// assert_eq!(stream.try_next().await, Ok(Some(vec![3])));    // The next value is an error, so couldn't fill the vec
    /// assert_eq!(stream.try_next().await, Err(4));
    /// assert_eq!(stream.try_next().await, Err(5));               // Consecutive errors are yielded 1 by 1
    /// assert_eq!(stream.try_next().await, Ok(Some(vec![6])));
    /// assert_eq!(stream.try_next().await, Ok(None));
    /// # })
    /// ```
    ///
    /// # Panics
    ///
    /// This method will panic if `capacity` is zero.
    fn chunks_ok(self, cap: usize) -> ChunksOk<Self>
    where
        Self: Sized + TryStreamExt,
    {
        ChunksOk::new(self, cap)
    }

    /// Flatten the result from the `Ok` value into the stream
    ///
    /// # Example
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::stream::{self, TryStreamExt, StreamExt};
    /// use streamies::TryStreamies as _;
    ///
    /// let stream = stream::iter(vec![Ok::<Result<i32, i32>, i32>(Ok::<i32, i32>(1)), Ok(Err(2)), Err(3)]);
    /// let mut stream = stream.flatten_result_ok();
    ///
    /// assert_eq!(stream.next().await, Some(Ok(1)));
    /// assert_eq!(stream.next().await, Some(Err(2)));
    /// assert_eq!(stream.next().await, Some(Err(3)));
    /// assert_eq!(stream.next().await, None);
    /// # })
    /// ```
    fn flatten_result_ok<T>(self) -> FlattenResult<Self>
    where
        Self: TryStream + Stream<Item = Result<Result<T, Self::Error>, Self::Error>> + Sized,
    {
        FlattenResult::new(self)
    }
}

impl<St: TryStream> TryStreamies for St {}
