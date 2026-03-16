//! Utility functions for formatting data in generic fashion.
//!
//! These are used to implement the various formatting functions
//! such as [`format_current_context`](crate::Report::format_current_context) and
//! [`format_inner`](crate::report_attachment::ReportAttachment::format_inner).
//!
//! Provided here as part of the API

use core::fmt::{self, Debug, Display};

use rootcause_internals::handlers::FormattingFunction;

pub(crate) trait DisplayDebug: Display + Debug {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>, function: FormattingFunction) -> fmt::Result {
        match function {
            FormattingFunction::Display => fmt::Display::fmt(&self, f),
            FormattingFunction::Debug => fmt::Debug::fmt(&self, f),
        }
    }
}

impl<DD: Display + Debug> DisplayDebug for DD {}

pub(crate) struct FormattingCallbacks<Data: Copy, Callback: Copy> {
    data: Data,
    callback: Callback,
}

pub(crate) type Format1With2Callbacks<D> = FormattingCallbacks<(D,), (FmtFn<D>, FmtFn<D>)>;

pub(crate) type Format1With1Callback<D> = FormattingCallbacks<(D,), FmtFnX<D>>;

pub(crate) type Format2With1Callback<D, E> = FormattingCallbacks<(D, E), Fmt2FnX<D, E>>;

type FmtFn<T> = fn(T, &mut fmt::Formatter<'_>) -> fmt::Result;
type FmtFnX<T> = fn(T, &mut fmt::Formatter<'_>, FormattingFunction) -> fmt::Result;
type Fmt2FnX<T, U> = fn(T, U, &mut fmt::Formatter<'_>, FormattingFunction) -> fmt::Result;

impl<D: Copy, C: Copy> FormattingCallbacks<D, C> {
    pub(crate) fn new(data: D, callback: C) -> Self {
        Self { data, callback }
    }
}

impl<D: Copy> fmt::Display for Format1With2Callbacks<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.callback.0)(self.data.0, f)
    }
}

impl<D: Copy> fmt::Debug for Format1With2Callbacks<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.callback.1)(self.data.0, f)
    }
}

impl<D: Copy> fmt::Debug for FormattingCallbacks<(D,), FmtFnX<D>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.callback)(self.data.0, f, FormattingFunction::Debug)
    }
}

impl<D: Copy> fmt::Display for FormattingCallbacks<(D,), FmtFnX<D>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.callback)(self.data.0, f, FormattingFunction::Display)
    }
}

impl<D: Copy, E: Copy> fmt::Debug for FormattingCallbacks<(D, E), Fmt2FnX<D, E>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.callback)(self.data.0, self.data.1, f, FormattingFunction::Debug)
    }
}

impl<D: Copy, E: Copy> fmt::Display for FormattingCallbacks<(D, E), Fmt2FnX<D, E>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.callback)(self.data.0, self.data.1, f, FormattingFunction::Display)
    }
}
