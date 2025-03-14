use core::future::Future;
use core::pin::Pin;
use core::task::ready;
use core::task::Context;
use core::task::Poll;

use futures::stream::FuturesUnordered;
use futures::StreamExt as _;
use pin_project_lite::pin_project;

// It would be better to use a enum, but that would require manual pin implementation
pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct FutureResult<FutOk, FutErr>{
        #[pin]
        ok: FuturesUnordered<FutOk>,
        #[pin]
        err: FuturesUnordered<FutErr>,
    }
}

impl<FutOk, FutErr> FutureResult<FutOk, FutErr> {
    pub fn ok(ok: FutOk) -> Self {
        let value = FuturesUnordered::new();
        value.push(ok);
        Self {
            ok: value,
            err: FuturesUnordered::new(),
        }
    }

    pub fn err(err: FutErr) -> Self {
        let value = FuturesUnordered::new();
        value.push(err);
        Self {
            ok: FuturesUnordered::new(),
            err: value,
        }
    }
}

impl<FutOk, FutErr> Future for FutureResult<FutOk, FutErr>
where
    FutOk: Future,
    FutErr: Future,
{
    type Output = Result<FutOk::Output, FutErr::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        match ready!(this.ok.poll_next_unpin(cx)) {
            None => {}
            Some(val) => return Poll::Ready(Ok(val)),
        }

        match ready!(this.err.poll_next_unpin(cx)) {
            None => {}
            Some(val) => return Poll::Ready(Err(val)),
        }

        panic!("FutureResult has no future saved.")
    }
}
