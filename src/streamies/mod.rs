use core::hash::Hash;

use futures::Stream;
use futures::StreamExt;

pub use crate::unique::Unique;
pub use crate::unique_by::UniqueBy;
pub use collect_vec::CollectVec;
pub use merge_round_robin::MergeRoundRobin;

pub mod collect_vec;
pub mod merge_round_robin;
pub mod unique;
pub mod unique_by;

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

    /// Return an stream adaptor that filters out elements that have
    /// already been produced once during the iteration.
    ///
    /// Duplicates are detected by comparing the key they map to
    /// with the keying function `f` by hash and equality.
    /// The keys are stored in a hash set in the stream.
    ///
    /// The stream is stable, returning the non-duplicate items in the order
    /// in which they occur in the adapted stream. In a set of duplicate
    /// items, the first item encountered is the item retained.
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::stream::{self, StreamExt};
    /// use streamies::Streamies as _;
    ///
    /// let data = vec!["a", "bb", "aa", "c", "ccc"];
    /// let stream = stream::iter(data).unique_by(|s| s.len());
    /// assert_eq!(stream.collect_vec().await, vec!["a", "bb", "ccc"]);
    /// # });
    /// ```
    fn unique_by<F, V>(self, f: F) -> UniqueBy<Self, V, F>
    where
        Self: Stream + StreamExt + Sized,
        V: Eq + Hash,
        F: FnMut(&Self::Item) -> V,
    {
        UniqueBy::new(self, f)
    }

    /// Return an stream adaptor that filters out elements that have
    /// already been produced once during the iteration. Duplicates
    /// are detected using hash and equality.
    ///
    /// Clones of visited elements are stored in a hash set in the
    /// stream.
    ///
    /// The stream is stable, returning the non-duplicate items in the order
    /// in which they occur in the adapted stream. In a set of duplicate
    /// items, the first item encountered is the item retained.
    ///
    /// ```
    /// # futures::executor::block_on(async {
    /// use futures::stream::{self, StreamExt};
    /// use streamies::Streamies as _;
    ///
    /// let data = vec![10, 20, 30, 20, 40, 10, 50];
    /// let stream = stream::iter(data).unique();
    /// assert_eq!(stream.collect_vec().await, vec![10, 20, 30, 40, 50]);
    /// # });
    /// ```
    fn unique(self) -> Unique<Self>
    where
        Self: Sized,
        Self::Item: Eq + Hash + Clone,
    {
        Unique::new(self)
    }
}

impl<St: Stream> Streamies for St {}
