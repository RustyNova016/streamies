use core::num::NonZeroUsize;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use futures::ready;
use futures::stream::FusedStream;
use futures::Stream;
use pin_project_lite::pin_project;

pin_project! {
    /// Stream for the [`merge_round_robin`](crate::Streamies::merge_round_robin) method.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct TryBooleanAnd<St1, St2> 
    where 
        St2: Stream 
    {
        #[pin]
        first: Option<St1>,
        #[pin]
        second: Option<St2>,

        comparison_pool: Vec<St2::Item>
    }
}


impl<St1, St2> TryBooleanAnd<St1, St2>
where
    St1: Stream,
    St2: Stream<Item = St1::Item>,
{
    pub(super) fn new(
        stream1: St1,
        stream2: St2,
        first_nb_ele: usize,
        second_nb_ele: usize,
    ) -> Self {
        Self {
            first: Some(stream1),
            second: Some(stream2),
            comparison_pool: Default::default(),
        }
    }
}

impl<St1, St2> FusedStream for TryBooleanAnd<St1, St2>
where
    St1: FusedStream,
    St2: FusedStream<Item = St1::Item>,
{
    fn is_terminated(&self) -> bool {
        self.first.as_ref().is_none_or(|s| s.is_terminated())
            && self.second.as_ref().is_none_or(|s| s.is_terminated())
    }
}

impl<St1, St2> Stream for TryBooleanAnd<St1, St2>
where
    St1: Stream,
    St1::Item: Eq,
    St2: Stream<Item = St1::Item>,
{
    type Item = St1::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        

    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.first {
            Some(first) => match &self.second {
                Some(second) => {
                    let first_size = first.size_hint();
                    let second_size = second.size_hint();

                    (
                        first_size.0.saturating_add(second_size.0),
                        match (first_size.1, second_size.1) {
                            (Some(x), Some(y)) => x.checked_add(y),
                            _ => None,
                        },
                    )
                }
                None => first.size_hint(),
            },
            None => match &self.second {
                Some(second) => second.size_hint(),
                None => (0, Some(0)),
            },
        }
    }
}
