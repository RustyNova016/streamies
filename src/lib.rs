pub use futures::{Stream, StreamExt, TryStream, TryStreamExt};

pub mod merge_round_robin;
pub use self::merge_round_robin::MergeRoundRobin;

pub trait Streamies: Stream {
    /// Merge two streams into one, allowing a custom round robin policy
    ///
    /// The resulting stream emits `nb_self` elements from the first stream,
    /// then `nb_other` from the other. When one of the stream finishes, the
    /// second is then used.
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::stream::{self, StreamExt};
    /// use streamies::Streamies as _;
    ///
    /// let stream1 = stream::iter(vec!["a", "a"]);
    /// let stream2 = stream::iter(vec!["b", "b", "b", "b", "c"]);
    ///
    /// let stream = stream1.merge_round_robin(stream2, 1, 2);
    ///
    /// let result: Vec<_> = stream.collect().await;
    /// assert_eq!(result, vec![
    ///     "a",
    ///     "b",
    ///     "b",
    ///     "a",
    ///     "b",
    ///     "b",
    ///     "c"
    /// ]);
    /// # });
    /// ```
    fn merge_round_robin<St>(
        self,
        other: St,
        nb_self: usize,
        nb_other: usize,
    ) -> MergeRoundRobin<Self, St>
    where
        St: Stream<Item = Self::Item>,
        Self: Sized,
    {
        MergeRoundRobin::new(self, other, nb_self, nb_other)
    }
}

impl<St: Stream> Streamies for St {}
