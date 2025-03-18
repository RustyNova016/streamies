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
    pub struct Unique<St: Stream> {
        #[pin]
        stream: St,
        used: HashMap<St::Item, ()>,
    }
}

impl<St: Stream> Unique<St> {
    pub(super) fn new(stream: St) -> Self
    where
        St::Item: Eq + Hash,
    {
        Self {
            stream,
            used: Default::default(),
        }
    }
}

impl<St: Stream> FusedStream for Unique<St>
where
    St: FusedStream + Stream + StreamExt,
    St::Item: Eq + Hash + Clone,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St> Stream for Unique<St>
where
    St: Stream + StreamExt,
    St::Item: Eq + Hash + Clone,
{
    type Item = St::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            let item = ready_some!(this.stream.as_mut().poll_next(cx));

            if let Entry::Vacant(e) = this.used.entry(item) {
                let elt = e.key().clone();
                e.insert(());
                return Poll::Ready(Some(elt));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
