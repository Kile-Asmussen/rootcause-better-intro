use rootcause::{
    Report, ReportMut, ReportRef,
    markers::{Mutable, ReportOwnershipMarker},
    report_attachment::{ReportAttachmentMut, ReportAttachmentRef},
};

pub trait ReportRefExt<'a> {
    fn find_attachment<A>(self) -> Option<ReportAttachmentRef<'a, A>>;
}

impl<'a, C: ?Sized, O, T> ReportRefExt<'a> for ReportRef<'a, C, O, T> {
    fn find_attachment<A>(self) -> Option<ReportAttachmentRef<'a, A>> {
        self.attachments()
            .iter()
            .find_map(|a| a.downcast_attachment())
    }
}

pub trait ReportExt {
    fn find_attachment<A>(&self) -> Option<ReportAttachmentRef<'_, A>>;
}

impl<C: ?Sized, O: ReportOwnershipMarker, T> ReportExt for Report<C, O, T> {
    fn find_attachment<A>(&self) -> Option<ReportAttachmentRef<'_, A>> {
        self.as_ref().find_attachment()
    }
}

impl<C: ?Sized, T> ReportExt for ReportMut<'_, C, T> {
    fn find_attachment<A>(&self) -> Option<ReportAttachmentRef<'_, A>> {
        self.as_ref().find_attachment()
    }
}

pub trait ReportMutExt {
    fn find_attachment_mut<A>(&mut self) -> Option<ReportAttachmentMut<'_, A>>;
}

impl<C: ?Sized, T> ReportMutExt for Report<C, Mutable, T> {
    fn find_attachment_mut<A>(&mut self) -> Option<ReportAttachmentMut<'_, A>> {
        self.attachments_mut()
            .iter_mut()
            .find_map(|a| a.downcast_attachment().ok())
    }
}

impl<C: ?Sized, T> ReportMutExt for ReportMut<'_, C, T> {
    fn find_attachment_mut<A>(&mut self) -> Option<ReportAttachmentMut<'_, A>> {
        self.attachments_mut()
            .iter_mut()
            .find_map(|a| a.downcast_attachment().ok())
    }
}
