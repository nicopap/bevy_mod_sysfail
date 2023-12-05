/*!
[`FailureMode`]: FailureMode
[`LogLevelOverride`]: LogLevelOverride
[`Failure`]: Failure
[`quick_sysfail`]: quick_sysfail
[`sysfail`]: sysfail
*/
#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]

use bevy_ecs::event::{Event, EventWriter};
use bevy_ecs::system::lifetimeless::SRes;
use bevy_ecs::system::{Local, StaticSystemParam, SystemParam};
use bevy_time::Time;
use bevy_utils::Duration;
use core::fmt;
use std::hash::Hash;
use std::marker::PhantomData;

pub use bevy_mod_sysfail_macros::*;
pub use logged_errors::LoggedErrors;

mod logged_errors;

#[doc(hidden)]
pub mod __macro {
    pub type LoggedErrorsParam<'s, T> = bevy_ecs::prelude::Local<'s, LoggedErrors<T>>;
    pub type TimeParam<'s> = bevy_ecs::prelude::Res<'s, bevy_time::Time>;
    pub use crate::{Failure, LoggedErrors};
    pub use bevy_ecs::system::StaticSystemParam;
    pub use bevy_log::{debug, error, info, trace, warn};
}

/// This crate's traits.
///
/// Useful if you need to use extension methods, for example on
/// [`LogLevelOverride`].
pub mod traits {
    pub use crate::{Failure, FailureMode};
}

pub trait LogLevelModifier {
    fn log(t: impl fmt::Display);
}

/// Something that can be logged in a [`sysfail`] handler
pub trait FailureMode {
    /// Used to de-duplicate identical messages to avoid spamming the log.
    type ID: Hash + Eq + Send + Sync;

    /// How long an error must not be produced in order to be displayed again.
    ///
    /// This controls when [`Failure::get_error`] returns an error.
    ///
    /// Return `Duration::ZERO` to trigger [`Failure::get_error`] each time there
    /// is an error.
    fn cooldown(&self) -> Duration {
        Duration::from_secs(1)
    }

    /// What constitutes "distinct" error types.
    fn identify(&self) -> Self::ID;
}

impl FailureMode for () {
    type ID = Self;
    fn identify(&self) {}
}
impl FailureMode for &'_ str {
    type ID = Self;
    fn identify(&self) -> Self {
        self
    }
}
impl FailureMode for Box<dyn std::error::Error> {
    type ID = ();
    /// By default, only print a single error per system.
    fn identify(&self) {}
}
impl FailureMode for anyhow::Error {
    type ID = ();
    /// By default, only print a single error per system.
    fn identify(&self) {}
}

/// Something that can be returned by a function marked with `#[sysfail(log)]`.
pub trait Failure {
    /// The actual error's type in this failure.
    type Error: FailureMode;

    /// The system param used by [`Self::get_error`].
    type Param: SystemParam;

    /// Get the error if it is ready to be logged.
    fn get_error(self, param: StaticSystemParam<Self::Param>);
}

impl Failure for Option<()> {
    type Error = ();
    type Param = ();

    fn get_error(self, _: StaticSystemParam<Self::Param>) {}
}
impl Failure for Result<(), ()> {
    type Error = ();
    type Param = ();

    fn get_error(self, _: StaticSystemParam<()>) {}
}

/// As the `Err` of the return value of a `sysfail` system, send the `E` event.
pub struct Emit<E>(pub E);

impl<E> From<E> for Emit<E> {
    fn from(value: E) -> Self {
        Self(value)
    }
}

impl<E: Event + 'static> Failure for Result<(), Emit<E>> {
    type Error = ();
    type Param = EventWriter<'static, E>;

    fn get_error(self, params: StaticSystemParam<Self::Param>) {
        let mut event_writer = params.into_inner();
        let Err(Emit(event)) = self else {
            return;
        };
        event_writer.send(event);
    }
}

/// As the `Err` of the return value of a `sysfail` system, log `T`.
///
/// # Example
///
/// This will log `MyError` using [`error!`]
/// ```rust
/// #[sysfail]
/// fn failable_system(q: Query<&Transform>) -> Result<(), Log<MyError>> {
///     let () = Err(MyError)?;
///     // ...
/// }
/// ```
///
/// Note that if `MyError` implements [`std::error::Error`], `Log` is not necessary.
pub struct Log<T, Lvl = Warn>(pub T, PhantomData<Lvl>);

impl<T, Lvl> Log<T, Lvl> {
    pub fn new(t: T) -> Self {
        Self(t, PhantomData)
    }
}

impl<T, Lvl> From<T> for Log<T, Lvl> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T, Lvl> Failure for Result<(), Log<T, Lvl>>
where
    T: fmt::Display + FailureMode + 'static,
    Lvl: LogLevelModifier + 'static,
{
    type Error = T;
    type Param = (SRes<Time>, Local<'static, LoggedErrors<Self>>);

    fn get_error(self, params: StaticSystemParam<Self::Param>) {
        let (time, mut logged_errors) = params.into_inner();
        let Err(Log(error, _)) = self else {
            return;
        };
        let cooldown = error.cooldown();
        let now = time.elapsed();
        let last_shown = logged_errors.0.insert(error.identify(), now);
        let should_log = last_shown.map_or(true, |d| now < d + cooldown);
        if should_log {
            Lvl::log(error);
        }
    }
}

/// Never log anything.
pub enum Silent {}
/// `trace!`
pub enum Trace {}
/// `debug!`
pub enum Debug {}
/// `info!`
pub enum Info {}
/// `warn!`
pub enum Warn {}
/// `error!`
pub enum Error {}

macro_rules! impl_log_level {
    ($($tys:ty => $level:ident),* $(,)?) => {
        $(impl LogLevelModifier for $tys {
            #[track_caller]
            fn log(t: impl fmt::Display) {
                bevy_log::$level!("{t}");
            }
        })*
    }
}
impl LogLevelModifier for Silent {
    fn log(_: impl fmt::Display) {}
}
impl_log_level![
    Trace => trace,
    Debug => debug,
    Info => info,
    Warn => warn,
    Error => error,
];

macro_rules! impl_failure {
    ($failure_mode:ty) => {
        impl Failure for Result<(), $failure_mode> {
            type Error = $failure_mode;
            type Param = <Result<(), Log<$failure_mode>> as Failure>::Param;

            #[track_caller]
            fn get_error(self, params: StaticSystemParam<Self::Param>) {
                self.map_err(Log::new).get_error(params);
            }
        }
    };
}
impl_failure!(&'static str);
impl_failure!(anyhow::Error);
impl_failure!(Box<dyn std::error::Error>);
