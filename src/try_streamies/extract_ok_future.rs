use core::future::Future;
use core::future::Ready;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use futures::stream::FusedStream;
use futures::Stream;
use futures::TryStream;
use pin_project_lite::pin_project;

use crate::futuries::future_result::FutureResult;
use crate::futuries::ResultExt as _;
use crate::ready_some;

pin_project! {
    /// Stream for the [`merge_round_robin`](crate::Streamies::merge_round_robin) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct ExtractFutureOk<St> where St: TryStream{
        #[pin]
        stream: St,
    }
}

impl<St> ExtractFutureOk<St>
where
    St: TryStream,
{
    pub(super) fn new(stream: St) -> Self {
        Self { stream }
    }
}

impl<St> FusedStream for ExtractFutureOk<St>
where
    St: FusedStream + TryStream + Stream<Item = Result<St::Ok, St::Error>>,
    St::Ok: Future,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St> Stream for ExtractFutureOk<St>
where
    St: TryStream + Stream<Item = Result<St::Ok, St::Error>>, // Stream bound for the TryStream to have a Result item
    St::Ok: Future,
{
    type Item = FutureResult<St::Ok, Ready<St::Error>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let item = ready_some!(this.stream.as_mut().poll_next(cx));

        Poll::Ready(Some(item.extract_future_ok()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
