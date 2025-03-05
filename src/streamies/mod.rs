use core::future::Future;

use futures::Stream;
use futures::StreamExt;

pub use await_value::AwaitValue;
pub use collect_vec::CollectVec;
pub use merge_round_robin::MergeRoundRobin;

pub mod await_value;
pub mod collect_vec;
pub mod merge_round_robin;

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

    /// Collect the stream into a vec.
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::stream::{self, StreamExt};
    /// use streamies::Streamies as _;
    ///
    /// let stream = stream::iter(vec![1, 2, 3]);
    ///
    /// let result = stream.collect_vec().await; // No need for ...: Vec<_>!
    /// assert_eq!(result, vec![1, 2, 3]);
    /// # });
    /// ```
    fn collect_vec(self) -> CollectVec<Self>
    where
        Self: Sized,
    {
        CollectVec::new(self.collect())
    }

    /// Await the item, turning it from a future to the output value
    ///
    /// > ⚠️ This prevents other items to get polled until the current future is resolved,
    /// > essentially resolving futures 1 by 1 **without** concurency. 
    /// >
    /// > If you need **concurency**, use [`StreamExt::buffered`]
    /// >
    /// > If you need **parallelism**, map the future to a task, and use [`StreamExt::buffered`]
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use core::future::ready;
    /// use futures::stream::{self, StreamExt};
    /// use streamies::Streamies as _;
    ///
    /// let stream = stream::iter(vec![1, 2, 3]);
    ///
    /// // Execute an async function on the items,
    /// // turning it in a stream of futures
    /// let stream = stream.map(|n| ready(n * 2));
    ///
    /// // Then turn it back to a stream of number
    /// let stream = stream.await_value();
    ///
    /// let result = stream.collect_vec().await;
    /// assert_eq!(result, vec![2, 4, 6]);
    /// # });
    /// ```
    fn await_value(self) -> AwaitValue<Self>
    where
        Self: Stream + Sized,
        Self::Item: Future + Unpin,
    {
        AwaitValue::new(self)
    }
}

impl<St: Stream> Streamies for St {}
