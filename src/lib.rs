/*!
[`FailureMode`]: FailureMode
[`LogLevelOverride`]: LogLevelOverride
[`Failure`]: Failure
[`quick_sysfail`]: quick_sysfail
[`sysfail`]: sysfail
*/
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]

use bevy_time::Time;
use bevy_utils::Duration;
use std::hash::Hash;

pub use bevy_mod_sysfail_macros::*;
pub use logged_errors::LoggedErrors;

mod logged_errors;

#[doc(hidden)]
pub mod __macro {
    pub type LoggedErrorsParam<'s, T> = bevy_ecs::prelude::Local<'s, LoggedErrors<T>>;
    pub type TimeParam<'s> = bevy_ecs::prelude::Res<'s, bevy_time::Time>;
    pub use crate::{Failure, LoggedErrors};
    pub use bevy_log::{debug, error, info, trace, warn};
}
/// The [`quick_sysfail`] and [`sysfail`] macros.
#[deprecated(
    since = "4.0.0",
    note = "Directly import the macros with bevy_mod_sysfail::{sysfail, quick_sysfail}"
)]
pub mod macros {
    pub use crate::{quick_sysfail, sysfail};
}
/// This crate's traits.
///
/// Useful if you need to use extension methods, for example on
/// [`LogLevelOverride`].
pub mod traits {
    #[allow(deprecated)]
    pub use crate::{Failure, FailureMode, LogLevelOverride};
}

/// Deprecated, `bevy_mod_sysfail` doesn't respect the runtime-set log level.
#[deprecated(since = "4.0.0", note = "runtime log levels are not respected anymore")]
pub struct OverrideLevel<T> {
    inner: T,
}
#[allow(deprecated)]
impl<T> OverrideLevel<T> {
    /// Log `inner`, but always with provided `level`.
    pub fn new(_: LogLevel, inner: T) -> Self {
        Self { inner }
    }
}
#[allow(deprecated)]
impl<T: FailureMode> FailureMode for OverrideLevel<T> {
    fn log_level(&self) -> LogLevel {
        LogLevel::Silent
    }
    type ID = T::ID;

    fn identify(&self) -> Self::ID {
        self.inner.identify()
    }
    fn display(&self) -> Option<String> {
        None
    }
    fn cooldown(&self) -> Duration {
        self.inner.cooldown()
    }
}

/// Deprecated: This is completely ignored by `bevy_mod_sysfail`.
#[deprecated(since = "4.0.0", note = "runtime log levels are not respected anymore")]
#[allow(deprecated)]
pub trait LogLevelOverride: Sized {
    /// The type resulting from applying the override.
    type Output;

    /// Deprecated: This is completely ignored by `bevy_mod_sysfail`.
    fn set_level(self, level: LogLevel) -> Self::Output;

    /// Deprecated: This is completely ignored by `bevy_mod_sysfail`.
    fn silent(self) -> Self::Output {
        self.set_level(LogLevel::Silent)
    }
    /// Deprecated: This is completely ignored by `bevy_mod_sysfail`.
    fn trace(self) -> Self::Output {
        self.set_level(LogLevel::Trace)
    }
    /// Deprecated: This is completely ignored by `bevy_mod_sysfail`.
    fn debug(self) -> Self::Output {
        self.set_level(LogLevel::Debug)
    }
    /// Deprecated: This is completely ignored by `bevy_mod_sysfail`.
    fn info(self) -> Self::Output {
        self.set_level(LogLevel::Info)
    }
    /// Deprecated: This is completely ignored by `bevy_mod_sysfail`.
    fn warn(self) -> Self::Output {
        self.set_level(LogLevel::Warn)
    }
    /// Deprecated: This is completely ignored by `bevy_mod_sysfail`.
    fn error(self) -> Self::Output {
        self.set_level(LogLevel::Error)
    }
}
#[allow(deprecated)]
impl<T: FailureMode> LogLevelOverride for T {
    type Output = OverrideLevel<Self>;
    fn set_level(self, level: LogLevel) -> OverrideLevel<Self> {
        OverrideLevel::new(level, self)
    }
}
#[allow(deprecated)]
impl<T: FailureMode> LogLevelOverride for Result<(), T> {
    type Output = Result<(), OverrideLevel<T>>;
    fn set_level(self, level: LogLevel) -> Self::Output {
        self.map_err(|err| err.set_level(level))
    }
}

/// Deprecated: It is now completely unused.
#[deprecated(since = "4.0.0", note = "This is unusued and serves no purpose")]
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    /// Never log anything.
    Silent,
    /// `trace!`
    Trace,
    /// `debug!`
    Debug,
    /// `info!`
    Info,
    /// `warn!`
    Warn,
    /// `error!`
    Error,
}

/// Something that can be logged in a [`sysfail`] handler
pub trait FailureMode {
    /// Deprecated: this is ignored.
    #[deprecated(since = "4.0.0", note = "This is ignored")]
    #[allow(deprecated)]
    fn log_level(&self) -> LogLevel;

    /// How long an error must not be produced in order to be displayed again.
    ///
    /// This controls when [`Failure::get_error`] returns an error.
    ///
    /// Return `Duration::ZERO` to trigger [`Failure::get_error`] each time there
    /// is an error.
    fn cooldown(&self) -> Duration {
        Duration::from_secs(1)
    }

    /// Used to de-duplicate identical messages to avoid spamming the log.
    type ID: Hash + Eq;

    /// What constitutes "distinct" error types.
    fn identify(&self) -> Self::ID;

    /// Deprecated, this does nothing.
    #[deprecated(
        since = "4.0.0",
        note = "This crate now directly uses the fmt::Display impl on the error."
    )]
    fn display(&self) -> Option<String>;

    /// What happens, by default this logs based on the return value of
    /// [`FailureMode::log_level`].
    ///
    /// # Examples
    ///
    /// <https://crates.io/crates/bevy_debug_text_overlay> provides a wrapper
    /// struct to replace log-printing with displaying errors on screen.
    fn log(&self) {}
}

impl FailureMode for () {
    #[allow(deprecated)]
    fn log_level(&self) -> LogLevel {
        LogLevel::Silent
    }
    type ID = Self;
    fn identify(&self) {}
    fn display(&self) -> Option<String> {
        None
    }
}
impl FailureMode for &'static str {
    #[allow(deprecated)]
    fn log_level(&self) -> LogLevel {
        LogLevel::Warn
    }
    type ID = Self;
    fn identify(&self) -> Self {
        self
    }
    fn display(&self) -> Option<String> {
        Some((*self).to_string())
    }
}
impl FailureMode for Box<dyn std::error::Error> {
    #[allow(deprecated)]
    fn log_level(&self) -> LogLevel {
        LogLevel::Warn
    }
    type ID = ();
    /// By default, only print a single error per system.
    fn identify(&self) {}
    fn display(&self) -> Option<String> {
        Some(self.to_string())
    }
}
impl FailureMode for anyhow::Error {
    #[allow(deprecated)]
    fn log_level(&self) -> LogLevel {
        LogLevel::Warn
    }
    type ID = ();
    /// By default, only print a single error per system.
    fn identify(&self) {}
    fn display(&self) -> Option<String> {
        Some(self.to_string())
    }
}

/// Something that can be returned by a function marked with `#[sysfail(log)]`.
pub trait Failure {
    /// The actual error's type in this failure.
    type Error: FailureMode;

    /// The actual error in this failure, None if the failure isn't a failure.
    fn failure(self) -> Option<Self::Error>;

    /// Get the error if it is ready to be logged.
    fn get_error(self, time: &Time, logged_errors: &mut LoggedErrors<Self>) -> Option<Self::Error>
    where
        Self: Sized,
    {
        let error = self.failure()?;
        let cooldown = error.cooldown();
        let now = time.raw_elapsed();
        let last_shown = logged_errors.0.insert(error.identify(), now);
        let should_log = last_shown.map_or(true, |d| now < d + cooldown);
        should_log.then_some(error)
    }
}
impl<T: FailureMode> Failure for Result<(), T> {
    type Error = T;

    fn failure(self) -> Option<Self::Error> {
        self.err()
    }
}
impl Failure for Option<()> {
    type Error = &'static str;

    fn failure(self) -> Option<Self::Error> {
        Some("A none value")
    }
}
