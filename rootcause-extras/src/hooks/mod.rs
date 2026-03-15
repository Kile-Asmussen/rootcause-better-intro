mod terse;
mod timestamp;
pub use terse::TerseReportFormatting;
pub use timestamp::*;

use core::fmt;
use std::marker::PhantomData;

use rootcause::{
    ReportRef,
    handlers::{
        AttachmentFormattingStyle, AttachmentHandler, ContextFormattingStyle, ContextHandler,
        FormattingFunction,
    },
    hooks::{
        attachment_formatter::{AttachmentFormatterHook, AttachmentParent},
        context_formatter::ContextFormatterHook,
    },
    markers::{Dynamic, Local, Uncloneable},
    report_attachment::ReportAttachmentRef,
};

use crate::handlers::{Opaque, Redacted};

/// Redacts contexts of this type
pub type RedactContexts<Contexts> = AlternateHandlerHook<Contexts, Redacted<Contexts>, true, false>;

/// Redacts attachments of this type
pub type RedactAttachments<Contexts> =
    AlternateHandlerHook<Contexts, Redacted<Contexts>, false, true>;

/// Renders attachments of this type as opaque
pub type ObscureAttachments<Attachment> =
    AlternateHandlerHook<Attachment, Opaque<Attachment>, false, true>;

/// Renders attachments of this type as hidden
pub type HideAttachments<Attachment> =
    AlternateHandlerHook<Attachment, Opaque<Attachment>, false, true>;

/// Context or attachment formatting hook that uses an alternate handler.
#[derive(Clone, Copy)]
pub struct AlternateHandlerHook<
    ContextOrAttachment,
    Hook,
    const CTX: bool = true,
    const ATT: bool = true,
>(PhantomData<fn(ContextOrAttachment) -> Hook>);

impl<CoA, H> AlternateHandlerHook<CoA, H, false, false> {
    pub fn new() -> ! {
        panic!("AlternateHandlerHook must apply to contexts or attachments")
    }
}

impl<C, H: ContextHandler<C>> AlternateHandlerHook<C, H, true, false> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<A, H: AttachmentHandler<A>> AlternateHandlerHook<A, H, false, true> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<CoA, H: ContextHandler<CoA> + AttachmentHandler<CoA>>
    AlternateHandlerHook<CoA, H, true, true>
{
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<A: 'static, H: AttachmentHandler<A>, const CTX: bool> AttachmentFormatterHook<A>
    for AlternateHandlerHook<A, H, CTX, true>
{
    fn display(
        &self,
        attachment: ReportAttachmentRef<'_, A>,
        _attachment_parent: Option<AttachmentParent<'_>>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        H::display(attachment.inner(), formatter)
    }

    fn debug(
        &self,
        attachment: ReportAttachmentRef<'_, A>,
        _attachment_parent: Option<AttachmentParent<'_>>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        H::debug(attachment.inner(), formatter)
    }

    fn preferred_formatting_style(
        &self,
        attachment: ReportAttachmentRef<'_, Dynamic>,
        report_formatting_function: FormattingFunction,
    ) -> AttachmentFormattingStyle {
        attachment
            .downcast_inner::<A>()
            .map(|a| H::preferred_formatting_style(a, report_formatting_function))
            .unwrap_or_else(|| {
                attachment.preferred_formatting_style_unhooked(report_formatting_function)
            })
    }
}

impl<C: 'static, H: ContextHandler<C>, const ATT: bool> ContextFormatterHook<C>
    for AlternateHandlerHook<C, H, true, ATT>
{
    fn display(
        &self,
        report: ReportRef<'_, C, Uncloneable, Local>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        H::display(report.current_context(), formatter)
    }

    fn debug(
        &self,
        report: ReportRef<'_, C, Uncloneable, Local>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        H::debug(report.current_context(), formatter)
    }

    fn preferred_context_formatting_style(
        &self,
        report: ReportRef<'_, Dynamic, Uncloneable, Local>,
        report_formatting_function: FormattingFunction,
    ) -> ContextFormattingStyle {
        report
            .downcast_current_context::<C>()
            .map(|a| H::preferred_formatting_style(a, report_formatting_function))
            .unwrap_or_else(|| {
                report.preferred_context_formatting_style_unhooked(report_formatting_function)
            })
    }
}
