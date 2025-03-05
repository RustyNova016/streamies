use core::future::Future;
use core::pin::pin;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use futures::ready;
use futures::stream::FusedStream;
use futures::Stream;
use pin_project_lite::pin_project;

pin_project! {
    /// Stream for the [`await_value`](crate::Streamies::await_value) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct AwaitValue<St>
    where
        St: Stream,
        St::Item: Future,
    {
        #[pin]
        stream: St,
        item: Option<St::Item>
    }
}

impl<St> AwaitValue<St>
where
    St: Stream,
    St::Item: Future + Unpin,
{
    pub(super) fn new(stream: St) -> Self {
        Self { stream, item: None }
    }
}

impl<St> FusedStream for AwaitValue<St>
where
    St: Stream + FusedStream,
    St::Item: Future + Unpin,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St> Stream for AwaitValue<St>
where
    St: Stream,
    St::Item: Future + Unpin,
{
    type Item = <St::Item as Future>::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if this.item.is_none() {
            match ready!(this.stream.poll_next(cx)) {
                Some(fut) => *this.item = Some(fut),
                None => return Poll::Ready(None),
            }
        }

        match this.item {
            Some(fut) => {
                let mut pinned_fut = pin!(fut);
                let poll = pinned_fut.as_mut().poll(cx);
                match poll {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(ite) => {
                        // Reset the held future
                        *this.item = None;

                        Poll::Ready(Some(ite))
                    }
                }
            }
            None => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let stream_size = self.stream.size_hint();

        match self.item {
            None => stream_size,
            Some(_) => (stream_size.0 + 1, stream_size.1.map(|n| n + 1)),
        }
    }
}
