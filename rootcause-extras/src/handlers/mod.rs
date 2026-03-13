use core::{error::Error, fmt};

use rootcause::handlers::{
    AttachmentFormattingPlacement, AttachmentFormattingStyle, AttachmentHandler,
    ContextFormattingStyle, ContextHandler, FormattingFunction,
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
