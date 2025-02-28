use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use futures::future::FusedFuture;
use futures::stream::FusedStream;
use futures::stream::TryCollect;
use futures::TryStream;
use pin_project_lite::pin_project;

pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct TryCollectVec<St>
    where
        St: TryStream {
        #[pin]
        stream: TryCollect<St, Vec<St::Ok>>,
    }
}

impl<St: TryStream> TryCollectVec<St> {
    pub(crate) fn new(stream: TryCollect<St, Vec<St::Ok>>) -> Self {
        Self { stream }
    }
}

impl<St> FusedFuture for TryCollectVec<St>
where
    St: TryStream + FusedStream,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St> Future for TryCollectVec<St>
where
    St: TryStream,
{
    type Output = Result<Vec<St::Ok>, St::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        this.stream.poll(cx)
    }
}
