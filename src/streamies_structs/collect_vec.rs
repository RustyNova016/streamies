use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use futures::future::FusedFuture;
use futures::stream::Collect;
use futures::stream::FusedStream;
use futures::Stream;
use pin_project_lite::pin_project;

pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct CollectVec<St>
    where
        St: Stream
    {
        #[pin]
        collect: Collect<St, Vec<St::Item>>,
    }
}

impl<St> CollectVec<St>
where
    St: Stream,
{
    pub fn new(collect: Collect<St, Vec<St::Item>>) -> Self {
        Self { collect }
    }
}

impl<St> FusedFuture for CollectVec<St>
where
    St: FusedStream,
{
    fn is_terminated(&self) -> bool {
        self.collect.is_terminated()
    }
}

impl<St> Future for CollectVec<St>
where
    St: Stream,
{
    type Output = Vec<St::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        this.collect.poll(cx)
    }
}
