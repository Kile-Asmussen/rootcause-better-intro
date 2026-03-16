//! Report creation hooks for automatic report modification.
//!
//! This module provides hooks that run automatically when errors are created,
//! allowing you to attach metadata or modify reports without changing the code
//! that creates the errors.
//!
//! **Note:** Hooks affect ALL errors globally. If you only need to attach data
//! to a specific error, use `.attach()` directly instead of hooks.
//!
//! # Hook Types (use in order of complexity)
//!
//! 1. **Closures** - Simplest: Just return a value to attach
//!
//!    ```rust
//!    # use rootcause::hooks::Hooks;
//!    Hooks::new().attachment_collector(|| "some data")
//!    # ;
//!    ```
//!
//! 2. **[`AttachmentCollector`]** - Simple: Collect and attach specific data
//!    automatically to every error. Use when you always want to attach the same
//!    type of information.
//!
//! 3. **[`ReportCreationHook`]** - Advanced: Full access to the report for
//!    conditional logic. Use when you need to inspect the error type or context
//!    before deciding what to attach.
//!
//! # Examples
//!
//! ## Simple: Using a Closure
//!
//! The easiest way to attach data to all errors:
//!
//! ```
//! use rootcause::hooks::Hooks;
//!
//! // Attach a request ID to every error
//! Hooks::new()
//!     .attachment_collector(|| format!("Request ID: {}", get_request_id()))
//!     .install()
//!     .expect("failed to install hooks");
//!
//! fn get_request_id() -> u64 {
//!     42
//! }
//! ```
//!
//! ## Medium: Custom Attachment Collector
//!
//! When you need to attach structured data or use a custom handler, implement
//! [`AttachmentCollector`]:
//!
//! ```
//! use rootcause::{
//!     hooks::{Hooks, report_creation::AttachmentCollector},
//!     prelude::*,
//! };
//!
//! // Simulates data from an external system monitoring crate
//! #[derive(Debug)]
//! struct SystemLoad {
//!     cpu_percent: f32,
//!     memory_used_mb: u64,
//! }
//!
//! fn get_system_load() -> SystemLoad {
//!     // In real code, this would call an external crate like `sysinfo`
//!     SystemLoad {
//!         cpu_percent: 45.2,
//!         memory_used_mb: 2048,
//!     }
//! }
//!
//! struct SystemLoadCollector;
//!
//! impl AttachmentCollector<SystemLoad> for SystemLoadCollector {
//!     type Handler = handlers::Debug;
//!
//!     fn collect(&self) -> SystemLoad {
//!         get_system_load()
//!     }
//! }
//!
//! Hooks::new()
//!     .attachment_collector(SystemLoadCollector)
//!     .install()
//!     .expect("failed to install hooks");
//! ```
//!
//! ## Advanced: Custom Report Creation Hook
//!
//! When you need conditional logic based on the error type, implement
//! [`ReportCreationHook`]:
//!
//! ```
//! use rootcause::{
//!     ReportMut,
//!     hooks::{Hooks, report_creation::ReportCreationHook},
//!     markers::{Dynamic, Local, SendSync},
//!     prelude::*,
//! };
//!
//! // Hook that adds retry hints only for retryable I/O errors
//! struct RetryHintHook;
//!
//! impl ReportCreationHook for RetryHintHook {
//!     fn on_local_creation(&self, mut report: ReportMut<'_, Dynamic, Local>) {
//!         // Only attach hint for I/O errors where retry might help
//!         if let Some(io_err) = report.downcast_current_context::<std::io::Error>() {
//!             if matches!(
//!                 io_err.kind(),
//!                 std::io::ErrorKind::TimedOut | std::io::ErrorKind::ConnectionRefused
//!             ) {
//!                 report
//!                     .attachments_mut()
//!                     .push(report_attachment!("Retry may succeed").into());
//!             }
//!         }
//!     }
//!
//!     fn on_sendsync_creation(&self, mut report: ReportMut<'_, Dynamic, SendSync>) {
//!         // Same logic for Send+Sync errors
//!         if let Some(io_err) = report.downcast_current_context::<std::io::Error>() {
//!             if matches!(
//!                 io_err.kind(),
//!                 std::io::ErrorKind::TimedOut | std::io::ErrorKind::ConnectionRefused
//!             ) {
//!                 report
//!                     .attachments_mut()
//!                     .push(report_attachment!("Retry may succeed").into());
//!             }
//!         }
//!     }
//! }
//!
//! Hooks::new()
//!     .report_creation_hook(RetryHintHook)
//!     .install()
//!     .expect("failed to install hooks");
//! ```

use core::{any, fmt, marker::PhantomData};

use alloc::{boxed::Box, vec::Vec};
use rootcause_internals::handlers::AttachmentHandler;

use crate::{
    ReportMut, handlers,
    hooks::{
        HookData,
        builtin_hooks::location::{Location, LocationHook},
        use_hooks,
    },
    markers::{Dynamic, Local, SendSync},
    report_attachment::ReportAttachment,
};

/// A hook that is called whenever a report is created.
///
/// Report creation hooks provide a way to automatically modify or enhance
/// reports as they are being created, without requiring changes to the code
/// that creates the reports. This is useful for adding consistent metadata,
/// logging, or performing other side effects.
///
/// If you only need to add attachments, then consider using an
/// [`AttachmentCollector`] instead, as it gives you an easier to use API
/// for this use case.
///
/// # Examples
///
/// ```
/// use rootcause::{
///     ReportMut,
///     hooks::{Hooks, report_creation::ReportCreationHook},
///     markers::{Dynamic, Local, SendSync},
///     prelude::*,
/// };
///
/// struct LoggingHook;
///
/// impl ReportCreationHook for LoggingHook {
///     fn on_local_creation(&self, mut report: ReportMut<'_, Dynamic, Local>) {
///         println!("Local report created: {}", report);
///         let attachment = report_attachment!("Logged by LoggingHook");
///         report.attachments_mut().push(attachment.into());
///     }
///
///     fn on_sendsync_creation(&self, mut report: ReportMut<'_, Dynamic, SendSync>) {
///         println!("SendSync report created: {}", report);
///         let attachment = report_attachment!("Logged by LoggingHook");
///         report.attachments_mut().push(attachment.into());
///     }
/// }
///
/// // Install the hook globally
/// Hooks::new()
///     .report_creation_hook(LoggingHook)
///     .install()
///     .expect("failed to install hooks");
/// ```
pub trait ReportCreationHook: 'static + Send + Sync {
    /// Called when a [`Local`] report is created.
    ///
    /// # Examples
    ///
    /// ```
    /// use rootcause::{
    ///     ReportMut,
    ///     hooks::report_creation::ReportCreationHook,
    ///     markers::{Dynamic, Local, SendSync},
    ///     report_attachment,
    /// };
    ///
    /// struct ThreadInfoHook;
    /// impl ReportCreationHook for ThreadInfoHook {
    ///     fn on_local_creation(&self, mut report: ReportMut<'_, Dynamic, Local>) {
    ///         let thread_id = format!("Thread: {:?}", std::thread::current().id());
    ///         report
    ///             .attachments_mut()
    ///             .push(report_attachment!(thread_id).into());
    ///     }
    ///
    ///     fn on_sendsync_creation(&self, _report: ReportMut<'_, Dynamic, SendSync>) {}
    /// }
    /// ```
    #[track_caller]
    fn on_local_creation(&self, report: ReportMut<'_, Dynamic, Local>);

    /// Called when a [`SendSync`] report is created.
    ///
    /// # Examples
    ///
    /// ```
    /// use rootcause::{
    ///     ReportMut,
    ///     hooks::report_creation::ReportCreationHook,
    ///     markers::{Dynamic, Local, SendSync},
    ///     report_attachment,
    /// };
    ///
    /// struct ProcessInfoHook;
    /// impl ReportCreationHook for ProcessInfoHook {
    ///     fn on_local_creation(&self, _report: ReportMut<'_, Dynamic, Local>) {}
    ///
    ///     fn on_sendsync_creation(&self, mut report: ReportMut<'_, Dynamic, SendSync>) {
    ///         let process_id = format!("Process ID: {}", std::process::id());
    ///         report
    ///             .attachments_mut()
    ///             .push(report_attachment!(process_id).into());
    ///     }
    /// }
    /// ```
    #[track_caller]
    fn on_sendsync_creation(&self, report: ReportMut<'_, Dynamic, SendSync>);
}

/// A hook that collects data to be automatically attached to reports when they
/// are created.
///
/// Attachment collector hooks provide a specialized way to automatically
/// collect and attach specific types of data to all reports. Unlike
/// [`ReportCreationHook`], which provides full access to the report, attachment
/// collectors are focused solely on gathering data to be attached.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for any closure that returns a value
/// implementing [`Display`] and [`Debug`], using [`handlers::Display`] as the
/// handler:
///
/// [`Display`]: core::fmt::Display
/// [`Debug`]: core::fmt::Debug
///
/// ```
/// use rootcause::hooks::Hooks;
///
/// // This closure automatically implements AttachmentCollector<String>
/// Hooks::new()
///     .attachment_collector(|| "timestamp".to_string())
///     .install()
///     .expect("failed to install hooks");
/// ```
///
/// # Examples
///
/// ## Custom Collector Implementation
///
/// ```
/// use rootcause::{
///     hooks::{Hooks, report_creation::AttachmentCollector},
///     prelude::*,
/// };
///
/// struct SystemInfoCollector;
///
/// impl AttachmentCollector<String> for SystemInfoCollector {
///     type Handler = handlers::Display;
///
///     fn collect(&self) -> String {
///         format!(
///             "OS: {}, Arch: {}",
///             std::env::consts::OS,
///             std::env::consts::ARCH
///         )
///     }
/// }
///
/// // Install the collector globally
/// Hooks::new()
///     .attachment_collector(SystemInfoCollector)
///     .install()
///     .expect("failed to install hooks");
/// ```
///
/// ## Using a Closure
///
/// ```
/// use rootcause::hooks::Hooks;
///
/// // Install a closure that collects the current working directory
/// Hooks::new()
///     .attachment_collector(|| {
///         std::env::current_dir()
///             .map(|p| p.display().to_string())
///             .unwrap_or_else(|_| "unknown".to_string())
///     })
///     .install()
///     .expect("failed to install hooks");
/// ```
pub trait AttachmentCollector<A>: 'static + Send + Sync {
    /// The handler type used to format the collected data.
    type Handler: AttachmentHandler<A>;

    /// Collects the data to be attached to a report.
    ///
    /// This method is called once for each report creation and should return
    /// the data that will be attached to the report. The data will be formatted
    /// using the associated [`Handler`](Self::Handler) type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::SystemTime;
    ///
    /// use rootcause::hooks::report_creation::AttachmentCollector;
    ///
    /// struct TimestampCollector;
    /// impl AttachmentCollector<String> for TimestampCollector {
    ///     type Handler = rootcause::handlers::Display;
    ///
    ///     fn collect(&self) -> String {
    ///         // Collect current timestamp to attach to reports
    ///         match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
    ///             Ok(duration) => format!("Timestamp: {}s", duration.as_secs()),
    ///             Err(_) => "Timestamp: unknown".to_string(),
    ///         }
    ///     }
    /// }
    /// ```
    #[track_caller]
    fn collect(&self) -> A;
}

impl<A, F> AttachmentCollector<A> for F
where
    A: 'static + core::fmt::Display + core::fmt::Debug,
    F: 'static + Send + Sync + Fn() -> A,
{
    type Handler = handlers::Display;

    #[track_caller]
    fn collect(&self) -> A {
        (self)()
    }
}

#[derive(Debug, Default)]
pub(crate) struct HookList {
    list: Vec<Box<dyn StoredHook>>,
}

impl HookList {
    pub(crate) fn new_with_locations() -> Self {
        let mut res = Self { list: Vec::new() };
        res.push_collector(LocationHook);
        res
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = &dyn StoredHook> {
        self.list.iter().map(|b| &**b)
    }

    pub(crate) fn push_collector<A, C>(&mut self, collector: C)
    where
        A: 'static + Send + Sync,
        C: AttachmentCollector<A>,
    {
        let hook = Hook::<C, (A,)> {
            hook: collector,
            _hooked_type: PhantomData,
        };

        self.list.push(Box::new(hook))
    }

    pub(crate) fn push_hook<H: ReportCreationHook>(&mut self, hook: H) {
        let hook = Hook::<H, ()> {
            hook,
            _hooked_type: PhantomData,
        };

        self.list.push(Box::new(hook))
    }

    #[track_caller]
    fn on_local_creation(&self, mut report: ReportMut<'_, Dynamic, Local>) {
        for hook in self.iter() {
            hook.on_local_creation(report.as_mut());
        }
    }

    #[track_caller]
    fn on_sendsync_creation(&self, mut report: ReportMut<'_, Dynamic, SendSync>) {
        for hook in self.iter() {
            hook.on_sendsync_creation(report.as_mut());
        }
    }
}

struct Hook<H, A>
where
    H: 'static + Sync,
    A: 'static + Send + Sync,
{
    hook: H,
    _hooked_type: PhantomData<fn() -> A>,
}

impl<H: ReportCreationHook> fmt::Debug for Hook<H, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CreationHook<{}>", any::type_name::<H>())
    }
}

impl<A, C> fmt::Debug for Hook<C, (A,)>
where
    A: 'static + Send + Sync,
    C: AttachmentCollector<A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AttachmentCollector<{}, {}, {}>",
            core::any::type_name::<A>(),
            core::any::type_name::<C::Handler>(),
            core::any::type_name::<C>(),
        )
    }
}

/// Internal trait for stored report creation hooks.
trait StoredHook: 'static + Send + Sync + core::fmt::Debug {
    #[track_caller]
    fn on_local_creation(&self, report: ReportMut<'_, Dynamic, Local>);

    #[track_caller]
    fn on_sendsync_creation(&self, report: ReportMut<'_, Dynamic, SendSync>);
}

static LOCATION: Hook<LocationHook, (Location,)> = Hook {
    hook: LocationHook,
    _hooked_type: PhantomData,
};

#[track_caller]
pub(crate) fn run_creation_hooks_local(report: ReportMut<'_, Dynamic, Local>) {
    use_hooks(|hook_data: Option<&HookData>| {
        if let Some(hook_data) = hook_data {
            hook_data.report_creation.on_local_creation(report);
        } else {
            LOCATION.on_local_creation(report);
        }
    })
}

#[track_caller]
pub(crate) fn run_creation_hooks_sendsync(report: ReportMut<'_, Dynamic, SendSync>) {
    use_hooks(|hook_data: Option<&HookData>| {
        if let Some(hook_data) = hook_data {
            hook_data.report_creation.on_sendsync_creation(report);
        } else {
            LOCATION.on_sendsync_creation(report);
        }
    })
}

impl<H> StoredHook for Hook<H, ()>
where
    H: ReportCreationHook,
{
    fn on_local_creation(&self, report: ReportMut<'_, Dynamic, Local>) {
        self.hook.on_local_creation(report);
    }

    fn on_sendsync_creation(&self, report: ReportMut<'_, Dynamic, SendSync>) {
        self.hook.on_sendsync_creation(report);
    }
}

impl<A, C: AttachmentCollector<A>> StoredHook for Hook<C, (A,)>
where
    A: 'static + Send + Sync,
{
    #[track_caller]
    fn on_local_creation(&self, mut report: ReportMut<'_, Dynamic, Local>) {
        let attachment = self.hook.collect();
        report
            .attachments_mut()
            .push(ReportAttachment::new_local_custom::<C::Handler>(attachment).into_dynamic());
    }

    #[track_caller]
    fn on_sendsync_creation(&self, mut report: ReportMut<'_, Dynamic, SendSync>) {
        let attachment = self.hook.collect();
        report
            .attachments_mut()
            .push(ReportAttachment::new_sendsync_custom::<C::Handler>(attachment).into_dynamic());
    }
}
