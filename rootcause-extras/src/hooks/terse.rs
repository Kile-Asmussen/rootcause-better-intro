use core::fmt;

use rootcause::{
    ReportRef,
    handlers::FormattingFunction,
    hooks::{builtin_hooks::location::Location, report_formatter::ReportFormatter},
    markers::{Dynamic, Local, Uncloneable},
};
use rootcause_backtrace::Backtrace;

use crate::utils::ReportRefExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerseReportFormatting;

impl ReportFormatter for TerseReportFormatting {
    fn format_reports(
        &self,
        reports: &[ReportRef<'_, Dynamic, Uncloneable, Local>],
        formatter: &mut fmt::Formatter<'_>,
        report_formatting_function: FormattingFunction,
    ) -> fmt::Result {
        if let Some(report) = reports.first() {
            self.format_report(*report, formatter, report_formatting_function)?;
        }

        Ok(())
    }

    fn format_report(
        &self,
        report: ReportRef<'_, Dynamic, Uncloneable, Local>,
        formatter: &mut fmt::Formatter<'_>,
        report_formatting_function: FormattingFunction,
    ) -> fmt::Result {
        writeln!(formatter, "Report<{}>:", report.current_context_type_name())?;

        let context = report.format_current_context();

        match report_formatting_function {
            FormattingFunction::Display => fmt::Display::fmt(&context, formatter)?,
            FormattingFunction::Debug => fmt::Debug::fmt(&context, formatter)?,
        }

        if let Some(location) = report.find_attachment::<Location>() {
            write!(formatter, "\n@ {}", location.format_inner())?;
        }

        if let Some(backtrace) = report.find_attachment::<Backtrace>() {
            write!(formatter, "\n{}", backtrace.format_inner())?;
        }

        if report.attachments().len() > 0 {
            write!(formatter, "\n{} attachments", report.attachments().len())?;
        }

        if report.children().len() > 0 {
            write!(formatter, "\n{} children", report.children().len())?;
        }

        Ok(())
    }
}
