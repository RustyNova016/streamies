use core::pin::Pin;
use core::task::ready;
use core::task::Context;
use core::task::Poll;

use futures::stream::FusedStream;
use futures::Stream;
use futures::TryStream;
use pin_project_lite::pin_project;

pin_project! {
    /// Stream for the [`try_flatten`](super::TryStreamExt::try_flatten) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct FlattenOk<St>
    where
        St: TryStream,
    {
        #[pin]
        stream: St,
        #[pin]
        inner_stream: Option<St::Ok>,
    }
}

impl<St> FlattenOk<St>
where
    St: TryStream,
    St::Ok: Stream,
{
    pub(super) fn new(stream: St) -> Self {
        Self {
            stream,
            inner_stream: None,
        }
    }
}

impl<St> FusedStream for FlattenOk<St>
where
    St: TryStream + FusedStream,
    St::Ok: Stream,
{
    fn is_terminated(&self) -> bool {
        self.inner_stream.is_none() && self.stream.is_terminated()
    }
}

impl<St> Stream for FlattenOk<St>
where
    St: TryStream,
    St::Ok: Stream,
{
    type Item = Result<<St::Ok as Stream>::Item, St::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            if let Some(inner_stream) = this.inner_stream.as_mut().as_pin_mut() {
                match ready!(inner_stream.poll_next(cx)) {
                    Some(item) => break Some(Ok(item)),
                    None => this.inner_stream.set(None),
                }
            } else if let Some(s) = ready!(this.stream.as_mut().try_poll_next(cx)?) {
                this.inner_stream.set(Some(s));
            } else {
                break None;
            }
        })
    }
}
