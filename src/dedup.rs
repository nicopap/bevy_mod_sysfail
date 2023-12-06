use std::{fmt, hash::Hash};

use bevy_utils::Duration;

/// An error type with a cooldown and a category.
///
/// This is used by [`Log`](crate::prelude::Log) to avoid repetitively logging
/// the same error. This avoids spamming errors in the console.
pub trait Dedup: fmt::Display {
    /// Used to de-duplicate identical messages to avoid spamming the log.
    type ID: Hash + Eq + Send + Sync + 'static;

    /// How long an error must not be produced in order to be displayed again.
    ///
    /// This controls when [`Log`](crate::prelude::Log) returns an error.
    ///
    /// If it returns `Duration::ZERO`, `Log` will print the error each time
    /// it is emitted, acting like [`LogSimply`](crate::prelude::LogSimply).
    fn cooldown(&self) -> Duration {
        Duration::from_secs(1)
    }

    /// What constitutes "distinct" error types.
    fn identify(&self) -> Self::ID;
}

impl Dedup for &'static str {
    type ID = Self;
    fn identify(&self) -> Self {
        self
    }
}
impl Dedup for Box<dyn std::error::Error> {
    type ID = ();
    /// By default, only print a single error per system.
    fn identify(&self) {}
}
impl Dedup for anyhow::Error {
    type ID = ();
    /// By default, only print a single error per system.
    fn identify(&self) {}
}
