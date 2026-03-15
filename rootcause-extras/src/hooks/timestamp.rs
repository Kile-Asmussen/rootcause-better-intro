use std::time::SystemTime;

#[cfg(feature = "chrono")]
use rootcause::handlers::{
    AttachmentFormattingPlacement, AttachmentFormattingStyle, FormattingFunction,
};
use rootcause::{handlers::AttachmentHandler, hooks::report_creation::AttachmentCollector};

use crate::handlers::Invisible;

#[cfg(not(feature = "chrono"))]
pub struct TimestampCollector;

#[cfg(not(feature = "chrono"))]
impl AttachmentCollector<SystemTime> for TimestampCollector {
    type Handler = Invisible;

    fn collect(&self) -> SystemTime {
        SystemTime::now()
    }
}

#[cfg(feature = "chrono")]
pub struct TimestampCollector<const VISIBLE: bool = false>;

#[cfg(feature = "chrono")]
impl AttachmentCollector<SystemTime> for TimestampCollector<false> {
    type Handler = Invisible;

    fn collect(&self) -> SystemTime {
        SystemTime::now()
    }
}

#[cfg(feature = "chrono")]
impl AttachmentCollector<SystemTime> for TimestampCollector<true> {
    type Handler = ChronoHandler;

    fn collect(&self) -> SystemTime {
        SystemTime::now()
    }
}

#[cfg(feature = "chrono")]
pub struct ChronoHandler;

#[cfg(feature = "chrono")]
impl AttachmentHandler<SystemTime> for ChronoHandler {
    fn display(value: &SystemTime, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "{}",
            chrono::DateTime::<chrono::Utc>::from(*value).to_rfc3339()
        )
    }

    fn debug(value: &SystemTime, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "{:?}",
            chrono::DateTime::<chrono::Utc>::from(*value).to_rfc3339()
        )
    }

    fn preferred_formatting_style(
        _value: &SystemTime,
        report_formatting_function: FormattingFunction,
    ) -> AttachmentFormattingStyle {
        AttachmentFormattingStyle {
            function: report_formatting_function,
            placement: AttachmentFormattingPlacement::Inline,
            priority: 15,
        }
    }
}
