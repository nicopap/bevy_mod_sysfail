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

use bevy_log::{debug, error, info, trace, warn};
use bevy_utils::Duration;
use std::hash::Hash;

pub use bevy_mod_sysfail_macros::*;

/// This crate's traits.
///
/// Useful if you need to use extension methods, for example on
/// [`LogLevelOverride`].
pub mod traits {
    pub use crate::{Failure, FailureMode, LogLevelOverride};
}
/// This crate's macros.
pub mod macros {
    pub use crate::{quick_sysfail, sysfail};
}

/// Override the log level of [`FailureMode`] to always be of a given level.
pub struct OverrideLevel<T> {
    level: LogLevel,
    inner: T,
}
impl<T> OverrideLevel<T> {
    /// Log `inner`, but always with provided `level`.
    pub fn new(level: LogLevel, inner: T) -> Self {
        Self { level, inner }
    }
}
impl<T: FailureMode> FailureMode for OverrideLevel<T> {
    /// Override the inner value's log level with the override's level.
    fn log_level(&self) -> LogLevel {
        self.level
    }
    type ID = T::ID;
    /// Proxy the inner type's identity.
    fn identify(&self) -> Self::ID {
        self.inner.identify()
    }
    /// Proxy the inner type's display value.
    fn display(&self) -> Option<String> {
        self.inner.display()
    }
    /// Proxy the inner type's cooldown.
    fn cooldown(&self) -> Duration {
        self.inner.cooldown()
    }
}

/// Extension trait with methods to override the log level of
/// [`Failure`] and [`FailureMode`].
pub trait LogLevelOverride: Sized {
    /// The type resulting from applying the override.
    type Output;

    /// Set `level` to provided value, regardless of underlying implementation.
    fn set_level(self, level: LogLevel) -> Self::Output;

    /// Set `level` to `Silent`, regardless of underlying implementation.
    fn silent(self) -> Self::Output {
        self.set_level(LogLevel::Silent)
    }
    /// Set `level` to `Trace`, regardless of underlying implementation.
    fn trace(self) -> Self::Output {
        self.set_level(LogLevel::Trace)
    }
    /// Set `level` to `Debug`, regardless of underlying implementation.
    fn debug(self) -> Self::Output {
        self.set_level(LogLevel::Debug)
    }
    /// Set `level` to `Info`, regardless of underlying implementation.
    fn info(self) -> Self::Output {
        self.set_level(LogLevel::Info)
    }
    /// Set `level` to `Warn`, regardless of underlying implementation.
    fn warn(self) -> Self::Output {
        self.set_level(LogLevel::Warn)
    }
    /// Set `level` to `Error`, regardless of underlying implementation.
    fn error(self) -> Self::Output {
        self.set_level(LogLevel::Error)
    }
}
impl<T: FailureMode> LogLevelOverride for T {
    type Output = OverrideLevel<Self>;
    fn set_level(self, level: LogLevel) -> OverrideLevel<Self> {
        OverrideLevel::new(level, self)
    }
}
impl<T: FailureMode> LogLevelOverride for Result<(), T> {
    type Output = Result<(), OverrideLevel<T>>;
    fn set_level(self, level: LogLevel) -> Self::Output {
        self.map_err(|err| err.set_level(level))
    }
}

/// The level of logging to log a [`FailureMode`] to.
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
    /// The log level of specific values.
    fn log_level(&self) -> LogLevel;

    /// How long an error must not be produced in order to be displayed again.
    ///
    /// This controls when [`FailureMode::log`] is called.
    ///
    /// Return `Duration::ZERO` to trigger [`FailureMode::log`] each time there
    /// is an error.
    fn cooldown(&self) -> Duration {
        Duration::from_secs(1)
    }

    /// Used to de-duplicate identical messages to avoid spamming the log.
    type ID: Hash + Eq;

    /// What constitutes "distinct" error types.
    fn identify(&self) -> Self::ID;

    /// What to log, the default [`FailureMode::log`] impl does nothing if
    /// `None`.
    fn display(&self) -> Option<String>;

    /// What happens, by default this logs based on the return value of
    /// [`FailureMode::log_level`].
    ///
    /// # Examples
    ///
    /// <https://crates.io/crates/bevy_debug_text_overlay> provides a wrapper
    /// struct to replace log-printing with displaying errors on screen.
    fn log(&self) {
        let message = match self.display() {
            None => return,
            Some(value) => value,
        };
        match self.log_level() {
            LogLevel::Silent => {}
            LogLevel::Trace => trace!("{message}"),
            LogLevel::Debug => debug!("{message}"),
            LogLevel::Info => info!("{message}"),
            LogLevel::Warn => warn!("{message}"),
            LogLevel::Error => error!("{message}"),
        }
    }
}

impl FailureMode for () {
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
    fn log_level(&self) -> LogLevel {
        LogLevel::Warn
    }
    type ID = Self;
    fn identify(&self) -> Self {
        *self
    }
    fn display(&self) -> Option<String> {
        Some((*self).to_string())
    }
}
impl FailureMode for Box<dyn std::error::Error> {
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
