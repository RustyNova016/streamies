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
    pub struct TransposeFutureErr<T, FutErr, E> where FutErr: Future<Output = E> {
        #[pin]
        item: Result<T, FutErr>
    }
}

impl<T, FutErr, E> TransposeFutureErr<T, FutErr, E>
where
    FutErr: Future<Output = E>,
{
    pub(crate) fn new(item: Result<T, FutErr>) -> Self {
        Self { item }
    }
}

impl<T, FutErr, E> FusedFuture for TransposeFutureErr<T, FutErr, E>
where
    FutErr: Future<Output = E> + FusedFuture,
{
    fn is_terminated(&self) -> bool {
        match &self.item {
            Ok(_) => false,
            Err(err) => err.is_terminated()
        }
    }
}

impl<T, FutErr, E> Future for TransposeFutureErr<T, FutErr, E>
where
    FutErr: Future<Output = E>,
{
    type Output = Result<T, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        match this.item.as {
            Ok(v) => Poll::Ready(v),
            Err(fut_err) => {

            }
        }
    }
}
