//! Utility functions for formatting data in generic fashion.
//!
//! These are used to implement the various formatting functions
//! such as [`format_current_context`](crate::Report::format_current_context) and
//! [`format_inner`](crate::report_attachment::ReportAttachment::format_inner).
//!
//! Provided here as part of the API

use core::fmt;

use rootcause_internals::handlers::FormattingFunction;

/// Helper struct that implements [`Display`] + [`Debug`] using various
/// callbacks.
///
/// See:
/// - [`Format1With2Callbacks`]
/// - [`Format2With2Callbacks`]
/// - [`Format1With1Callback`]
/// - [`Format2With1Callback`]
///
/// [`Display`]: fmt::Display
/// [`Debug`]: fmt::Debug
pub struct FormattingCallbacks<Data: Copy, Callback: Copy> {
    data: Data,
    callback: Callback,
}

/// Format one item using two formatting functions. Essentially
/// it contains an item `D` and two functions fitting the type signature
/// of [`fmt::Display::fmt`].
pub type Format1With2Callbacks<D> = FormattingCallbacks<(D,), (FmtFn<D>, FmtFn<D>)>;

/// Format two item s using two formatting functions.
pub type Format2With2Callbacks<D, E> = FormattingCallbacks<(D, E), (Fmt2Fn<D, E>, Fmt2Fn<D, E>)>;

/// Format one item using a single formatting function. Essentially
/// it contains an item `D` and a function that chooses [`Debug`] or
/// [`Display`] behavior based on a [`FormattingFunction`].
///
/// [`Debug`]: fmt::Debug
/// [`Display`]: fmt::Display
pub type Format1With1Callback<D> = FormattingCallbacks<(D,), FmtFnX<D>>;

/// Format two item s using two formatting functions.
pub type Format2With1Callback<D, E> = FormattingCallbacks<(D, E), Fmt2FnX<D, E>>;

type FmtFn<T> = fn(T, &mut fmt::Formatter<'_>) -> fmt::Result;
type Fmt2Fn<T, U> = fn(T, U, &mut fmt::Formatter<'_>) -> fmt::Result;

type FmtFnX<T> = fn(T, &mut fmt::Formatter<'_>, FormattingFunction) -> fmt::Result;
type Fmt2FnX<T, U> = fn(T, U, &mut fmt::Formatter<'_>, FormattingFunction) -> fmt::Result;

impl<D: Copy> Format1With2Callbacks<D> {
    ///
    pub fn new(data: (D,), callback: (FmtFn<D>, FmtFn<D>)) -> Self {
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

impl<D: Copy, E: Copy> FormattingCallbacks<(D, E), (Fmt2Fn<D, E>, Fmt2Fn<D, E>)> {
    ///
    pub fn new(data: (D, E), callback: (Fmt2Fn<D, E>, Fmt2Fn<D, E>)) -> Self {
        Self { data, callback }
    }
}

impl<D: Copy, E: Copy> fmt::Display for FormattingCallbacks<(D, E), (Fmt2Fn<D, E>, Fmt2Fn<D, E>)> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.callback.0)(self.data.0, self.data.1, f)
    }
}

impl<D: Copy, E: Copy> fmt::Debug for FormattingCallbacks<(D, E), (Fmt2Fn<D, E>, Fmt2Fn<D, E>)> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.callback.1)(self.data.0, self.data.1, f)
    }
}

impl<D: Copy> FormattingCallbacks<(D,), FmtFnX<D>> {
    ///
    pub fn new(data: (D,), callback: FmtFnX<D>) -> Self {
        Self { data, callback }
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

impl<D: Copy, E: Copy> FormattingCallbacks<(D, E), Fmt2FnX<D, E>> {
    ///
    pub fn new(data: (D, E), callback: Fmt2FnX<D, E>) -> Self {
        Self { data, callback }
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
