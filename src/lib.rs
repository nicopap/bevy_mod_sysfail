use bevy_log::{debug, error, info, trace, warn};
use std::hash::Hash;

pub use bevy_mod_system_tools_macros::sys_chain;

pub mod traits {
    pub use crate::{AutoTry, Failure, FailureMode};
}

// #[macro_export]
// macro_rules! SystemType {
//     (fn $_1:ident ( $($_2:pat_param : $typ:ty ),* $(,)? )) => {
//         fn($( $typ ),*)
//     }
// }

pub enum LogLevel {
    /// Never log anything
    Silent,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
/// The log level of the `Err` side of a `Result` in a `sys_chain(log)`.
pub trait FailureMode {
    fn log_level(&self) -> LogLevel;

    /// Used to de-duplicate identical messages to avoid spamming the log.
    type ID: Hash + Eq;
    fn identify(&self) -> Self::ID;

    fn display(&self) -> Option<String>;

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

/// Something that can be returned by a function in a `sys_chain(log)`.
pub trait Failure {
    type Error: FailureMode;
    fn error(&self) -> Option<&Self::Error>;
}
impl<T: FailureMode> Failure for Result<(), T> {
    type Error = T;

    fn error(&self) -> Option<&Self::Error> {
        self.as_ref().err()
    }
}
impl Failure for Option<()> {
    type Error = ();

    fn error(&self) -> Option<&Self::Error> {
        None
    }
}
impl Failure for () {
    type Error = ();

    fn error(&self) -> Option<&Self::Error> {
        None
    }
}

// AutoTry

pub trait AutoTry {
    const DEFAULT: Self;
}
impl<T> AutoTry for Result<(), T> {
    const DEFAULT: Self = Ok(());
}
impl AutoTry for Option<()> {
    const DEFAULT: Self = Some(());
}
