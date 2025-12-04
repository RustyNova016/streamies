use futures_lite::Stream;

use crate::smol_streamies::ready_chunks::ReadyChunks;

pub mod ready_chunks;

/// Streamies trait using smol's crate ecosystem.
///
/// This trait also port some of futures's methods that are lacking from [futures_lite],
/// but still following the crate's phylosophy
pub trait SmolStreamies: Stream {
    /// An adaptor for chunking up ready items of the stream inside a vector.
    ///
    /// This combinator will attempt to pull ready items from this stream and
    /// buffer them into a local vector. At most `capacity` items will get
    /// buffered before they're yielded from the returned stream. If underlying
    /// stream returns `Poll::Pending`, and collected chunk is not empty, it will
    /// be immediately returned.
    ///
    /// If the underlying stream ended and only a partial vector was created,
    /// it will be returned.
    ///
    /// # Panics
    ///
    /// This method will panic if `capacity` is zero.
    ///
    /// Ported from futures
    fn ready_chunks(self, capacity: usize) -> ReadyChunks<Self>
    where
        Self: Sized,
    {
        ReadyChunks::new(self, capacity)
    }
}
