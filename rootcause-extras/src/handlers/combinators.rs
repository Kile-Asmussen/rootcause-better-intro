use core::fmt;
use std::{any::type_name, marker::PhantomData};

use rootcause::handlers::{
    AttachmentFormattingPlacement, AttachmentFormattingStyle, AttachmentHandler,
    ContextFormattingStyle, ContextHandler, Display, FormattingFunction,
};

/// Attachement and Context handler combinator that redacts sensitive information when
/// rendering a full report. The innder handler is by default [`Debug`].
///
/// - **Debug output:** delegated to the inner handler.
/// - **Display output:** `Redacted attachment/context of type <typename>`
/// - **Preferred formatting:** overrides
pub struct Redacted<T: 'static, H = rootcause::handlers::Debug>(PhantomData<(T, H)>);

impl<T: 'static, H: AttachmentHandler<T>> AttachmentHandler<T> for Redacted<T, H> {
    fn display(_value: &T, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Redacted attachment of type {}",
            type_name::<T>()
        )
    }

    fn debug(value: &T, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        H::debug(value, formatter)
    }

    fn preferred_formatting_style(
        value: &T,
        function: FormattingFunction,
    ) -> AttachmentFormattingStyle {
        AttachmentFormattingStyle {
            function: FormattingFunction::Display,
            ..H::preferred_formatting_style(value, function)
        }
    }
}

impl<T: 'static, H: ContextHandler<T>> ContextHandler<T> for Redacted<T, H> {
    fn display(_value: &T, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Redacted context of type {}", type_name::<T>())
    }

    fn debug(value: &T, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        H::debug(value, formatter)
    }

    fn source(value: &T) -> Option<&(dyn core::error::Error + 'static)> {
        H::source(value)
    }

    fn preferred_formatting_style(
        value: &T,
        function: FormattingFunction,
    ) -> ContextFormattingStyle {
        let _ = value;
        ContextFormattingStyle {
            function: FormattingFunction::Display,
            ..H::preferred_formatting_style(value, function)
        }
    }
}

/// Attachment handler combinator that overrides the display type to be [`Hidden`].
///
///
///
/// [`Hidden`]: AttachmentFormattingPlacement::Hidden
pub struct Hidden<T: 'static, H: AttachmentHandler<T> = Display>(PhantomData<(T, H)>);

impl<T: 'static, H: AttachmentHandler<T>> AttachmentHandler<T> for Hidden<T, H> {
    fn display(value: &T, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        H::display(value, formatter)
    }

    fn debug(value: &T, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        H::debug(value, formatter)
    }

    fn preferred_formatting_style(
        value: &T,
        function: FormattingFunction,
    ) -> AttachmentFormattingStyle {
        AttachmentFormattingStyle {
            placement: AttachmentFormattingPlacement::Hidden,
            ..H::preferred_formatting_style(value, function)
        }
    }
}

pub struct Priority<T: 'static, H: AttachmentHandler<T> = Display, const PRIORITY: i32 = 0>(
    PhantomData<(T, H)>,
);

pub type LowPriority<T, H = Display> = Priority<T, H, -10>;
pub type LowestPriority<T, H = Display> = Priority<T, H, -100>;
pub type HighPriority<T, H = Display> = Priority<T, H, 10>;
pub type HighestPriority<T, H = Display> = Priority<T, H, 100>;

impl<T: 'static, H: AttachmentHandler<T>, const P: i32> AttachmentHandler<T> for Priority<T, H, P> {
    fn display(value: &T, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        H::display(value, formatter)
    }

    fn debug(value: &T, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        H::debug(value, formatter)
    }

    fn preferred_formatting_style(
        value: &T,
        function: FormattingFunction,
    ) -> AttachmentFormattingStyle {
        AttachmentFormattingStyle {
            priority: P,
            ..H::preferred_formatting_style(value, function)
        }
    }
}
