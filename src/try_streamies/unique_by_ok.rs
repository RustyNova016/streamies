use core::hash::Hash;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use futures::stream::FusedStream;
use futures::Stream;
use futures::TryStream;
use pin_project_lite::pin_project;

use crate::ready_some;
use crate::ready_some_ok;

pin_project! {
    /// Stream for the [`merge_round_robin`](crate::Streamies::merge_round_robin) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct UniqueByOk<St: Stream, V, F> {
        #[pin]
        stream: St,
        used: HashMap<V, ()>,
        f: F,
    }
}

impl<St: TryStream, V, F> UniqueByOk<St, V, F> {
    pub(super) fn new(stream: St, f: F) -> Self
    where
        V: Eq + Hash,

        F: FnMut(&St::Ok) -> V,
    {
        Self {
            stream,
            used: Default::default(),
            f,
        }
    }
}

impl<St: Stream, V, F> FusedStream for UniqueByOk<St, V, F>
where
    St: FusedStream + TryStream,
    V: Eq + Hash,
    F: FnMut(&St::Ok) -> V,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St, V, F> Stream for UniqueByOk<St, V, F>
where
    St: TryStream,
    V: Eq + Hash,
    F: FnMut(&St::Ok) -> V,
{
    type Item = Result<St::Ok, St::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let f = this.f;

        loop {
            let item = ready_some_ok!(this.stream.as_mut().try_poll_next(cx));

            let key = f(&item);

            if let Entry::Vacant(e) = this.used.entry(key) {
                e.insert(());
                return Poll::Ready(Some(Ok(item)));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
