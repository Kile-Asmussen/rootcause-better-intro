use std::{collections::VecDeque, iter::FusedIterator, marker::PhantomData};

use rootcause::{
    Report, ReportMut, ReportRef,
    markers::{Cloneable, Dynamic, ReportOwnershipMarker, Uncloneable},
};

/// An iterator over a report and all its descendant reports in breadth-first
/// order.
///
/// This iterator yields `ReportRef` items, which are references to the reports
/// in the hierarchy. The iterator traverses the report tree in a breadth-first
/// manner, starting from the root report and visiting each child report before
/// moving to the next sibling.
#[must_use]
pub struct ReportIterBfs<'a, Ownership: 'static, ThreadSafety: 'static> {
    stack: VecDeque<ReportRef<'a, Dynamic, Ownership, ThreadSafety>>,
    // This needs to be the standard
    _markers: PhantomData<(Ownership, ThreadSafety)>,
}

impl<'a, O, T> ReportIterBfs<'a, O, T> {
    pub(crate) fn from_raw(stack: VecDeque<ReportRef<'a, Dynamic, O, T>>) -> Self {
        Self {
            stack,
            _markers: PhantomData,
        }
    }
}

impl<'a, T> Iterator for ReportIterBfs<'a, Cloneable, T> {
    type Item = ReportRef<'a, Dynamic, Cloneable, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.stack.pop_front()?;

        let new_children = cur.children().iter().rev();
        self.stack.extend(new_children);
        Some(cur)
    }
}

impl<'a, T> Iterator for ReportIterBfs<'a, Uncloneable, T> {
    type Item = ReportRef<'a, Dynamic, Uncloneable, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.stack.pop_front()?;

        let new_children = cur.children().iter().map(ReportRef::into_uncloneable).rev();
        self.stack.extend(new_children);
        Some(cur)
    }
}

impl<'a, T> FusedIterator for ReportIterBfs<'a, Cloneable, T> {}
impl<'a, T> Unpin for ReportIterBfs<'a, Cloneable, T> {}

impl<'a, T> FusedIterator for ReportIterBfs<'a, Uncloneable, T> {}
impl<'a, T> Unpin for ReportIterBfs<'a, Uncloneable, T> {}

pub trait ReportIterExt<Ownership, ThreadSafety> {
    fn iter_reports_bfs(&self) -> ReportIterBfs<'_, Ownership, ThreadSafety>;
    fn iter_sub_reports_bfs(&self) -> ReportIterBfs<'_, Cloneable, ThreadSafety>;
}

impl<C: ?Sized, O: ReportOwnershipMarker, T> ReportIterExt<O::RefMarker, T> for &Report<C, O, T> {
    fn iter_reports_bfs(&self) -> ReportIterBfs<'_, O::RefMarker, T> {
        self.as_ref().iter_reports_bfs()
    }

    fn iter_sub_reports_bfs(&self) -> ReportIterBfs<'_, Cloneable, T> {
        self.as_ref().iter_sub_reports_bfs()
    }
}

impl<'a, C: ?Sized, T> ReportIterExt<Uncloneable, T> for ReportMut<'a, C, T> {
    fn iter_reports_bfs(&self) -> ReportIterBfs<'_, Uncloneable, T> {
        self.as_ref().iter_reports_bfs()
    }

    fn iter_sub_reports_bfs(&self) -> ReportIterBfs<'_, Cloneable, T> {
        self.as_ref().iter_sub_reports_bfs()
    }
}

pub trait ReportRefIterExt<'a, Ownership, ThreadSafety> {
    fn iter_reports_bfs(self) -> ReportIterBfs<'a, Ownership, ThreadSafety>;
    fn iter_sub_reports_bfs(self) -> ReportIterBfs<'a, Cloneable, ThreadSafety>;
}

impl<'a, C: ?Sized, O, T> ReportRefIterExt<'a, O, T> for ReportRef<'a, C, O, T> {
    fn iter_reports_bfs(self) -> ReportIterBfs<'a, O, T> {
        let stack = VecDeque::from([self.into_dynamic()]);
        ReportIterBfs::from_raw(stack)
    }

    fn iter_sub_reports_bfs(self) -> ReportIterBfs<'a, Cloneable, T> {
        let stack = self.children().iter().collect();
        ReportIterBfs::from_raw(stack)
    }
}
