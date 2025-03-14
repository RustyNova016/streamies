use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use futures::stream::FusedStream;
use futures::Stream;
use futures::TryStream;
use pin_project_lite::pin_project;

pin_project! {
    /// Stream for the [`merge_round_robin`](crate::Streamies::merge_round_robin) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct TryReadyChunksResult<St> where St: TryStream{
        #[pin]
        stream: St,
        cap: usize,
        error: Option<St::Error>
    }
}

impl<St> TryReadyChunksResult<St>
where
    St: TryStream,
{
    pub(super) fn new(stream: St, cap: usize) -> Self {
        Self {
            stream,
            cap,
            error: None,
        }
    }
}

impl<St> FusedStream for TryReadyChunksResult<St>
where
    St: FusedStream + TryStream + Stream<Item = Result<St::Ok, St::Error>>,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St> Stream for TryReadyChunksResult<St>
where
    St: TryStream + Stream<Item = Result<St::Ok, St::Error>>, // Stream bound for the TryStream to have a Result item
{
    type Item = Result<Vec<St::Ok>, St::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // Return the error of the previous poll
        if let Some(err) = this.error.take() {
            return Poll::Ready(Some(Err(err)));
        }

        let mut items = Vec::new();

        loop {
            match this.stream.as_mut().poll_next(cx) {
                // Flush all collected data if underlying stream doesn't contain
                // more ready values
                Poll::Pending => {
                    return if items.is_empty() {
                        Poll::Pending
                    } else {
                        Poll::Ready(Some(Ok(items)))
                    }
                }

                // Push the ready item into the buffer and check whether it is full.
                // If so, replace our buffer with a new and empty one and return
                // the full one.
                Poll::Ready(Some(Ok(item))) => {
                    if items.is_empty() {
                        items.reserve(*this.cap);
                    }
                    items.push(item);
                    if items.len() >= *this.cap {
                        return Poll::Ready(Some(Ok(items)));
                    }
                }

                // Found an error! If we got items, we store it for next poll, and return our items
                // Or else we return the error directly
                Poll::Ready(Some(Err(item))) => {
                    if items.is_empty() {
                        return Poll::Ready(Some(Err(item)));
                    }
                    let _ = this.error.insert(item); // The previous error should be yielded earlier

                    return Poll::Ready(Some(Ok(items)));
                }

                // Got a None. The stream is finished, so if we got values, we return them
                Poll::Ready(None) => {
                    if items.is_empty() {
                        return Poll::Ready(None);
                    }

                    return Poll::Ready(Some(Ok(items)));
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.error.is_some() {
            let (lower, upper) = self.stream.size_hint();
            (lower + 1, upper.map(|v| v + 1))
        } else {
            self.stream.size_hint()
        }
    }
}
