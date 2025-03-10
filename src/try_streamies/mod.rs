pub mod try_and;
use futures::TryStream;
use futures::TryStreamExt;

pub use try_collect_vec::TryCollectVec;

pub mod try_collect_vec;

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
}

impl<St: TryStream> TryStreamies for St {}
