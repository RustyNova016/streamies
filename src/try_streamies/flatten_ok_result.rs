use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use futures::stream::FusedStream;
use futures::Stream;
use futures::TryStream;
use pin_project_lite::pin_project;

use crate::ready_some;

pin_project! {
    /// Stream for the [`try_flatten`](super::TryStreamExt::try_flatten) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct FlattenOkResult<St>
    where
        St: TryStream,
    {
        #[pin]
        stream: St,
        #[pin]
        inner_stream: Option<St::Ok>,
    }
}

impl<St> FlattenOkResult<St>
where
    St: TryStream,
{
    pub(super) fn new(stream: St) -> Self {
        Self {
            stream,
            inner_stream: None,
        }
    }
}

impl<St, T> FusedStream for FlattenOkResult<St>
where
    St: TryStream + Stream<Item = Result<Result<T, St::Error>, St::Error>> + FusedStream,
    St::Ok: Stream,
{
    fn is_terminated(&self) -> bool {
        self.inner_stream.is_none() && self.stream.is_terminated()
    }
}

impl<St, T> Stream for FlattenOkResult<St>
where
    St: TryStream + Stream<Item = Result<Result<T, St::Error>, St::Error>>,
{
    type Item = Result<T, St::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match ready_some!(this.stream.poll_next(cx)) {
            Ok(Ok(val)) => Poll::Ready(Some(Ok(val))),
            Err(err) | Ok(Err(err)) => Poll::Ready(Some(Err(err))),
        }
    }
}
