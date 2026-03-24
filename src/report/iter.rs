use alloc::collections::vec_deque::VecDeque;
use core::{iter::FusedIterator, marker::PhantomData};

use crate::{ReportRef, markers::Dynamic};

/// An iterator over a report and all its descendant reports, by default in
/// depth-first order.
///
/// This iterator yields `ReportRef` items, which are references to the reports
/// in the hierarchy. The iterator traverses the report tree in a depth-first
/// manner, starting from the root report and visiting each child report before
/// moving to the next sibling.
#[must_use]
pub struct ReportIter<
    'a,
    Ownership: 'static,
    ThreadSafety: 'static,
    Traversal: ReportIterTraversalStrategy<Ownership, ThreadSafety> = DFS,
> {
    buffer: Traversal::Buffer<'a>,
    _ownership: PhantomData<Ownership>,
    _thread_safety: PhantomData<ThreadSafety>,
}

impl<'a, O, T, S: ReportIterTraversalStrategy<O, T>> ReportIter<'a, O, T, S> {
    /// Creates a new [`ReportIter`] from a vector of raw report references
    pub(crate) fn from_buffer(buffer: S::Buffer<'a>) -> Self {
        Self {
            buffer,
            _ownership: PhantomData,
            _thread_safety: PhantomData,
        }
    }
}

impl<'a, O, T, S: ReportIterTraversalStrategy<O, T>> Iterator for ReportIter<'a, O, T, S> {
    type Item = ReportRef<'a, Dynamic, O, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur: ReportRef<'a, Dynamic, O, T> = S::pop(&mut self.buffer)?;

        let new_children = cur
            .children()
            .iter()
            .map(|child_report| {
                // SAFETY:
                // 1. At this point we have an instance of a `ReportRef<'a, Dynamic, O, T>` in
                //    scope.  This means we can invoke the safety invariants of that ReportRef.
                //    One of the safety invariants of that `ReportRef` is that `O` must either
                //    be `Cloneable` or `Uncloneable`. But this fulfills our requirements for
                //    calling `ReportRef::from_cloneable` using that same `O`.
                unsafe {
                    // @add-unsafe-context: Dynamic
                    ReportRef::<Dynamic, O, T>::from_cloneable(child_report)
                }
            })
            .rev();

        self.buffer.extend(new_children);
        Some(cur)
    }
}

impl<'a, O, T, S: ReportIterTraversalStrategy<O, T>> FusedIterator for ReportIter<'a, O, T, S> {}

impl<'a, O, T, S: ReportIterTraversalStrategy<O, T>> Unpin for ReportIter<'a, O, T, S> {}

/// Depth-first traversal strategy for [`ReportIter`], implemented
/// using `pop_back` on a [`VecDeque`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DFS;

/// Breadth-first traversal strategy for [`ReportIter`], implemented
/// using `pop_front` on a [`VecDeque`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BFS;

/// Traversal strategy implementation trait, for use with [`ReportIter`].
pub trait ReportIterTraversalStrategy<O: 'static, T: 'static> {
    /// The internal type storing future nodes to visit.
    type Buffer<'a>: Extend<ReportRef<'a, Dynamic, O, T>>
        + FromIterator<ReportRef<'a, Dynamic, O, T>>;

    /// Retrieve the next node for traversal
    fn pop<'a>(it: &mut Self::Buffer<'a>) -> Option<ReportRef<'a, Dynamic, O, T>>;
}

impl<O: 'static, T: 'static> ReportIterTraversalStrategy<O, T> for DFS {
    type Buffer<'a> = VecDeque<ReportRef<'a, Dynamic, O, T>>;

    fn pop<'a>(it: &mut Self::Buffer<'a>) -> Option<ReportRef<'a, Dynamic, O, T>> {
        it.pop_back()
    }
}

impl<O: 'static, T: 'static> ReportIterTraversalStrategy<O, T> for BFS {
    type Buffer<'a> = VecDeque<ReportRef<'a, Dynamic, O, T>>;
    fn pop<'a>(it: &mut Self::Buffer<'a>) -> Option<ReportRef<'a, Dynamic, O, T>> {
        it.pop_front()
    }
}
