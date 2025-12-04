use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use futures_lite::stream::Fuse;
use futures_lite::Stream;
use futures_lite::StreamExt as _;
use pin_project_lite::pin_project;

pin_project! {
    /// Stream for the [`ready_chunks`](super::StreamExt::ready_chunks) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct ReadyChunks<St: Stream> {
        #[pin]
        stream: Fuse<St>,
        cap: usize, // https://github.com/rust-lang/futures-rs/issues/1475
    }
}

impl<St: Stream> ReadyChunks<St> {
    pub(super) fn new(stream: St, capacity: usize) -> Self {
        assert!(capacity > 0);

        Self {
            stream: stream.fuse(),
            cap: capacity,
        }
    }
}

impl<St: Stream> Stream for ReadyChunks<St> {
    type Item = Vec<St::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let mut items: Vec<St::Item> = Vec::new();

        loop {
            match this.stream.as_mut().poll_next(cx) {
                // Flush all collected data if the underlying stream doesn't contain
                // more ready values
                Poll::Pending => {
                    return if items.is_empty() {
                        Poll::Pending
                    } else {
                        Poll::Ready(Some(items))
                    }
                }

                // Push the ready item into the buffer and check whether it is full.
                // If so, replace our buffer with a new and empty one and return
                // the full one.
                Poll::Ready(Some(item)) => {
                    if items.is_empty() {
                        items.reserve(*this.cap);
                    }
                    items.push(item);
                    if items.len() >= *this.cap {
                        return Poll::Ready(Some(items));
                    }
                }

                // Since the underlying stream ran out of values, return what we
                // have buffered, if we have anything.
                Poll::Ready(None) => {
                    let last = if items.is_empty() { None } else { Some(items) };

                    return Poll::Ready(last);
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.stream.size_hint();
        let lower = lower / self.cap;
        (lower, upper)
    }
}
