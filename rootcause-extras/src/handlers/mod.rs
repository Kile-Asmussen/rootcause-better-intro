use core::{error::Error, fmt};

use rootcause::{
    Report,
    handlers::{
        AttachmentFormattingPlacement, AttachmentFormattingStyle, AttachmentHandler,
        ContextFormattingStyle, ContextHandler, FormattingFunction,
    },
};

pub mod combinators;
pub use combinators::*;

/// The 'null' context and attachment handler.
///
/// - **Display output**: empty string
/// - **Debug output**: empty string
/// - **Source**: Always returns `None`
/// - **Placement**: always hidden
pub struct Invisible;

impl<T> ContextHandler<T> for Invisible {
    fn source(_value: &T) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn display(_value: &T, _formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }

    fn debug(_value: &T, _formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }

    fn preferred_formatting_style(
        _value: &T,
        function: FormattingFunction,
    ) -> ContextFormattingStyle {
        ContextFormattingStyle {
            function,
            follow_source: false,
            follow_source_depth: Some(0),
        }
    }
}

impl<T> AttachmentHandler<T> for Invisible {
    fn display(_value: &T, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }

    fn debug(_value: &T, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }

    fn preferred_formatting_style(
        _value: &T,
        function: FormattingFunction,
    ) -> AttachmentFormattingStyle {
        AttachmentFormattingStyle {
            function,
            placement: AttachmentFormattingPlacement::Hidden,
            priority: i32::MIN,
        }
    }
}

/// A very silly context handler, for if you ever for some reason
/// want to have a full report as your context.
///
/// - **Display output**: the full report formatted with display formatting
/// - **Debug output**: the full report formatted with debug formatting
/// - **Source**: the source of the contained report
/// - **Formatting**: the contained report's formatting is passed on
///
/// You shouldn't use this, but... here it is.
pub struct ReportAsContextHandler;

impl<C, O, T> ContextHandler<Report<C, O, T>> for ReportAsContextHandler {
    fn source(value: &Report<C, O, T>) -> Option<&(dyn core::error::Error + 'static)> {
        value.current_context_error_source()
    }

    fn display(
        value: &Report<C, O, T>,
        formatter: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        fmt::Display::fmt(value, formatter)
    }

    fn debug(
        value: &Report<C, O, T>,
        formatter: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        fmt::Debug::fmt(value, formatter)
    }

    fn preferred_formatting_style(
        value: &Report<C, O, T>,
        report_formatting_function: FormattingFunction,
    ) -> ContextFormattingStyle {
        value.preferred_context_formatting_style(report_formatting_function)
    }
}
