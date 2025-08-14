use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use futures::stream::FusedStream;
use futures::Stream;
use futures::TryStream;
use pin_project_lite::pin_project;

use crate::ready_some;
use crate::ready_some_ok;

pin_project! {
    /// Stream for the [`flatten_ok_iter`](super::TryStreamExt::flatten_ok_iter) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct FlattenOkIter<St, It>
    where
        St: TryStream,
        It: Iterator
    {
        #[pin]
        stream: St,
        iter: Option<It>
    }
}

impl<St, It> FlattenOkIter<St, It>
where
    St: TryStream,
    St::Ok: IntoIterator<IntoIter = It>,
    It: Iterator,
{
    pub(super) fn new(stream: St) -> Self {
        Self { stream, iter: None }
    }
}

impl<St, It> FusedStream for FlattenOkIter<St, It>
where
    St: TryStream + FusedStream,
    St::Ok: IntoIterator<IntoIter = It>,
    It: Iterator,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St, It> Stream for FlattenOkIter<St, It>
where
    St: TryStream,
    St::Ok: IntoIterator<IntoIter = It>,
    It: Iterator,
{
    type Item = Result<It::Item, St::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            // Are we already polling an iter?
            if let Some(iter) = &mut this.iter {
                // Try yielding the next item
                if let Some(item) = iter.next() {
                    return Poll::Ready(Some(Ok(item)));
                } else {
                    // The iterator is finished. We remove the current one
                    this.iter.take();
                }
            }

            // Let's get a new iterator
            this.iter
                .replace(ready_some_ok!(this.stream.as_mut().try_poll_next(cx)).into_iter());
        }
    }
}
