use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use futures::future::FusedFuture;
use futures::ready;
use futures::stream::FusedStream;
use futures::stream::TryCollect;
use futures::Stream;
use futures::TryStream;
use pin_project_lite::pin_project;

pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct NullifyErr<St, F: FnOnce(St::Error)>
    where
        St: TryStream {
        #[pin]
        stream: St,
        func: F
    }
}

impl<St: TryStream, F: FnOnce(St::Error)> NullifyErr<St, F> {
    pub(crate) fn new(stream: St) -> Self {
        Self { stream }
    }
}

impl<St> FusedStream for NullifyErr<St>
where
    St: TryStream + FusedStream,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St> Stream for NullifyErr<St>
where
    St: TryStream,
{
    type Item = St::Ok;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();

        loop {
            match ready!(this.stream.poll_next(cx)) {
                Some(item) => match item {
                    Ok(item) => return Poll::Ready(Some(item)),
                    Err(err) => return Poll::Ready(Some(item)),
                },
                None => return Poll::Ready(None),
            }
        }
    }
}
