use futures::TryStream;
use futures::TryStreamExt;

pub use try_collect_vec::TryCollectVec;
pub use try_ready_result::TryReadyChunksResult;

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
    /// // However, this will resul in an error value
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
    /// use futures::stream::{self, TryReadyChunksError, TryStreamExt};
    /// use streamies::TryStreamies as _;
    ///
    /// let stream = stream::iter(vec![Ok::<i32, i32>(1), Ok(2), Ok(3), Err(4), Err(5), Ok(6), Ok(7)]);
    /// let mut stream = stream.try_ready_chunks_result(2);
    ///
    /// assert_eq!(stream.try_next().await, Ok(Some(vec![1, 2])));    
    /// assert_eq!(stream.try_next().await, Ok(Some(vec![3])));    // The next value is an error, so couldn't fill the vec
    /// assert_eq!(stream.try_next().await, Err(4));
    /// assert_eq!(stream.try_next().await, Err(5));               // Consecutive errors are yielded 1 by 1
    /// assert_eq!(stream.try_next().await, Ok(Some(vec![6, 7])));
    /// # })
    /// ```
    ///
    /// # Panics
    ///
    /// This method will panic if `capacity` is zero.
    fn try_ready_chunks_result(self, cap: usize) -> TryReadyChunksResult<Self>
    where
        Self: Sized + TryStreamExt,
    {
        TryReadyChunksResult::new(self, cap)
    }
}

impl<St: TryStream> TryStreamies for St {}
