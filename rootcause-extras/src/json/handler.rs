use std::{
    error,
    fmt::{self, Debug},
    marker::PhantomData,
};

use rootcause::handlers::{
    self, AttachmentHandler, ContextFormattingStyle, ContextHandler, FormattingFunction,
};
use serde::Serialize;

use crate::utils::{FormatAttachment, FormatterWriter};

struct JsonHandler<T: 'static, H = handlers::Debug>(PhantomData<(T, H)>);

impl<C: Serialize + 'static, H: ContextHandler<C>> ContextHandler<C> for JsonHandler<C, H> {
    fn source(_value: &C) -> Option<&(dyn error::Error + 'static)> {
        None
    }

    fn display(value: &C, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = if formatter.alternate() {
            serde_json::to_string_pretty(value).map_err(|_| fmt::Error)?
        } else {
            serde_json::to_string(value).map_err(|_| fmt::Error)?
        };
        formatter.write_str(&val)
    }

    fn debug(value: &C, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        H::debug(value, formatter)
    }

    fn preferred_formatting_style(
        value: &C,
        function: FormattingFunction,
    ) -> ContextFormattingStyle {
        H::preferred_formatting_style(value, function)
    }
}

impl<A: Serialize + 'static, H: AttachmentHandler<A>> AttachmentHandler<A> for JsonHandler<A, H> {
    fn display(value: &A, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = if formatter.alternate() {
            serde_json::to_writer_pretty(FormatterWriter::new(formatter), value).map_err(|_| fmt::Error)?
        } else {
            serde_json::to_writer(FormatterWriter::new(formatter), value).map_err(|_| fmt::Error)?
        };
        Ok(())
    }

    fn debug(value: &A, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if formatter.alternate() {
            &FormatAttachment::<_, H>::new(value),
        } else {
            fmt::Debug::fmt(&FormatAttachment::<_, H>::new(value), formatter)
        }
    }

    fn preferred_formatting_style(
        value: &A,
        function: FormattingFunction,
    ) -> AttachmentFormattingStyle {
        H::preferred_formatting_style(value, function)
    }
}
