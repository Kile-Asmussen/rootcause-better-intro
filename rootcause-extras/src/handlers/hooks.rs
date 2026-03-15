use std::{fmt, marker::PhantomData};

use rootcause::{
    handlers::{AttachmentFormattingStyle, AttachmentHandler, FormattingFunction},
    hooks::attachment_formatter::{AttachmentFormatterHook, AttachmentParent},
    markers::Dynamic,
    preformatted::PreformattedAttachment,
    report_attachment::ReportAttachmentRef,
};

#[derive(Clone, Copy, Default)]
struct AlternateHandlerHook<ContextOrAttachment, Hook>(
    PhantomData<ContextOrAttachment>,
    PhantomData<Hook>,
);

impl<A, H: AttachmentHandler<A>> AttachmentFormatterHook<A> for AlternateHandlerHook<A, H> {
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
