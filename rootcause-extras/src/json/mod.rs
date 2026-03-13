use core::{error, fmt};
use std::{
    fmt::{self, Debug, format},
    marker::PhantomData,
};

use rootcause::{
    handlers::{
        self, AttachmentFormattingStyle, AttachmentHandler, ContextFormattingStyle, ContextHandler,
        FormattingFunction,
    },
    hooks::{
        attachment_formatter::{AttachmentFormatterHook, AttachmentParent},
        report_formatter::ReportFormatter,
    },
    markers::{Dynamic, Local, Uncloneable},
    report_attachment::ReportAttachmentRef,
};
use serde::Serialize;
use serde_json::Value;

use crate::utils::FormatAttachment;

mod formatter;
mod handler;

pub use handler::*;

pub use formatter::*;

struct JsonFormattingHook;

impl<A: Serialize + 'static> AttachmentFormatterHook<A> for JsonFormattingHook {
    fn display(
        &self,
        attachment: ReportAttachmentRef<'_, A>,
        attachment_parent: Option<AttachmentParent<'_>>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let _ = attachment_parent;
        fmt::Display::fmt(&attachment.format_inner_unhooked(), formatter)
    }

    fn display_preformatted(
        &self,
        attachment: ReportAttachmentRef<'_, rootcause::preformatted::PreformattedAttachment>,
        attachment_parent: Option<AttachmentParent<'_>>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let _ = attachment_parent;
        fmt::Display::fmt(&attachment.format_inner_unhooked(), formatter)
    }

    fn debug(
        &self,
        attachment: ReportAttachmentRef<'_, A>,
        attachment_parent: Option<AttachmentParent<'_>>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let _ = attachment_parent;
        fmt::Debug::fmt(&attachment.format_inner_unhooked(), formatter)
    }

    fn debug_preformatted(
        &self,
        attachment: ReportAttachmentRef<'_, rootcause::preformatted::PreformattedAttachment>,
        attachment_parent: Option<AttachmentParent<'_>>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let _ = attachment_parent;
        fmt::Debug::fmt(&attachment.format_inner_unhooked(), formatter)
    }

    fn preferred_formatting_style(
        &self,
        attachment: ReportAttachmentRef<'_, Dynamic>,
        report_formatting_function: FormattingFunction,
    ) -> AttachmentFormattingStyle {
        attachment.preferred_formatting_style_unhooked(report_formatting_function)
    }
}

#[derive(Debug)]
struct JsonReportFormatter;

impl ReportFormatter for JsonReportFormatter {
    fn format_reports(
        &self,
        reports: &[rootcause::ReportRef<'_, Dynamic, Uncloneable, Local>],
        formatter: &mut fmt::Formatter<'_>,
        report_formatting_function: FormattingFunction,
    ) -> fmt::Result {
    }

    fn format_report(
        &self,
        report: rootcause::ReportRef<'_, Dynamic, Uncloneable, Local>,
        formatter: &mut fmt::Formatter<'_>,
        report_formatting_function: FormattingFunction,
    ) -> fmt::Result {
    }
}

#[derive(Serialize)]
struct JsonFormattedReport {
    context: String,
    location: Option<String>,
    backtrace: Option<Vec<String>>,
    attachments: Vec<Value>,
    source: String,
    child_reports: Vec<JsonFormattedReport>,
}
