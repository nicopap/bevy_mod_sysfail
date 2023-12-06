/*!
[`Dedup`]: Dedup
[`Failure`]: Failure
[`sysfail`]: sysfail
[`Emit`]: prelude::Emit
[`Log`]: prelude::Log
[`LogSimply`]: prelude::LogSimply
[`Ignore`]: prelude::Ignore
[`Event`]: bevy_ecs::event::Event
*/
#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]

use bevy_ecs::system::SystemParam;

/// See the [`crate`]-level documentation for usage and examples.
pub use bevy_mod_sysfail_macros::sysfail;

/// See the [`crate`]-level documentation for usage and examples.
pub use bevy_mod_sysfail_macros::exclusive_sysfail;
pub use bevy_utils::tracing::{Callsite, Level};
pub use dedup::Dedup;
pub use log_levels::LogLevelModifier;

mod dedup;
mod emit;
mod ignore;
#[cfg(feature = "full")]
mod log;
mod log_levels;
mod log_simple;

/// Useful set of [`Failure`] default implementations and [`LogLevelModifier`]s.
pub mod prelude {
    pub use crate::emit::Emit;
    pub use crate::ignore::Ignore;
    #[cfg(feature = "full")]
    pub use crate::log::Log;
    pub use crate::log_levels::{Debug, Error, Info, Trace, Warn};
    pub use crate::log_simple::LogSimply;
    pub use crate::{exclusive_sysfail, sysfail, Failure};
}

/// Symbols for the `sysfail` attribute macro.
#[doc(hidden)]
pub mod __macro {
    pub use crate::Failure;
    pub use bevy_ecs::system::StaticSystemParam;
    pub use bevy_utils::tracing::callsite::{DefaultCallsite, Identifier};
    pub use bevy_utils::tracing::{field::FieldSet, metadata, Metadata};
}

/// The `Err` side of the return type of `#[sysfail]`.
pub trait Failure {
    /// The system param used by [`Self::handle_error`].
    type Param: SystemParam;

    /// If this `Failure` logs something, use this log level.
    const LEVEL: Level;

    /// Do something whenever a `#[sysfail]` system returns an `Err(Self)`.
    ///
    /// # Callsite
    ///
    /// Note the `callsite` parameter. This parameter is used for logging. Due
    /// to a limitation of the `tracing` crate, it is required to build a
    /// `Callsite` at the macro invocation position and then pass it, otherwise
    /// the metadata for file and system position is all messed up.
    ///
    /// Due to the overhead of creating a `Callsite`, **it is only
    /// `Some` if the `Failure` type name contains the string `"Log"`**, such
    /// as in `Log` or `LogSimply`.
    fn handle_error(
        self,
        param: <Self::Param as SystemParam>::Item<'_, '_>,
        callsite: Option<&'static impl Callsite>,
    );
}
