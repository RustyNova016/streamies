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
    pub struct UniqueOk<St: TryStream> {
        #[pin]
        stream: St,
        used: HashMap<St::Ok, ()>,
    }
}

impl<St: TryStream> UniqueOk<St> {
    pub(super) fn new(stream: St) -> Self
    where
        St::Ok: Eq + Hash,
    {
        Self {
            stream,
            used: Default::default(),
        }
    }
}

impl<St> FusedStream for UniqueOk<St>
where
    St: FusedStream + TryStream,
    St::Ok: Eq + Hash + Clone,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated()
    }
}

impl<St> Stream for UniqueOk<St>
where
    St: TryStream,
    St::Ok: Eq + Hash + Clone,
{
    type Item = Result<St::Ok, St::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            let item = ready_some_ok!(this.stream.as_mut().try_poll_next(cx));

            if let Entry::Vacant(e) = this.used.entry(item) {
                let elt = e.key().clone();
                e.insert(());
                return Poll::Ready(Some(Ok(elt)));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
