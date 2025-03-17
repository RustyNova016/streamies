use core::hash::Hash;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use futures::stream::FusedStream;
use futures::Stream;
use futures::StreamExt;
use pin_project_lite::pin_project;

use crate::ready_some;

pin_project! {
    /// Stream for the [`merge_round_robin`](crate::Streamies::merge_round_robin) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct UniqueBy<St: Stream, V, F> {
        #[pin]
        stream: St,
        used: HashMap<V, ()>,
        f: F,
    }
}

impl<St: Stream, V, F> UniqueBy<St, V, F> {
    pub(super) fn new(stream: St, f: F) -> Self
    where
        V: Eq + Hash,

        F: FnMut(&St::Item) -> V,
    {
        Self {
            stream,
            used: Default::default(),
            f,
        }
    }
}

impl<St: Stream, V, F> FusedStream for UniqueBy<St, V, F>
where
    St: FusedStream + Stream + StreamExt,
    V: Eq + Hash,
    F: FnMut(&St::Item) -> V,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St, V, F> Stream for UniqueBy<St, V, F>
where
    St: Stream + StreamExt,
    V: Eq + Hash,
    F: FnMut(&St::Item) -> V,
{
    type Item = St::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let f = this.f;

        loop {
            let item = ready_some!(this.stream.as_mut().poll_next(cx));
            let key = f(&item);

            if let Entry::Vacant(e) = this.used.entry(key) {
                e.insert(());
                return Poll::Ready(Some(item));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
