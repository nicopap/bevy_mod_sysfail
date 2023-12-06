use std::{fmt, marker::PhantomData};

use bevy::time::Time;
use bevy_ecs::system::{lifetimeless::SRes, Local, SystemParam};
use bevy_utils::tracing::level_filters::{LevelFilter, STATIC_MAX_LEVEL};
use bevy_utils::{Duration, HashMap};

use crate::{log_levels::Warn, Callsite, Dedup, Failure, Level, LogLevelModifier};

/// Log `T`.
///
/// # Example
///
/// This will log `MyError` using [`Warn`]:
/// ```rust
/// use bevy_mod_sysfail::prelude::*;
/// use bevy::prelude::*;
/// # type MyError = &'static str;
/// # const MyError: &'static str = "FOOBAR";
///
/// #[sysfail(Log<MyError>)]
/// fn failable_system(q: Query<&Transform>) {
///     let () = Err(MyError)?;
///     // ...
/// }
/// ```
///
/// To log using a different level, specify the second type parameter of `Log`:
/// ```rust
/// use bevy_mod_sysfail::prelude::*;
/// use bevy::prelude::*;
/// # type MyError = &'static str;
/// # const MyError: &'static str = "FOOBAR";
///
/// #[sysfail(Log<MyError, Error>)]
/// fn failable_system(q: Query<&Transform>) {
///     let () = Err(MyError)?;
///     // ...
/// }
/// ```
/// Available as second argument are `Trace`, `Debug`, `Info`, `Warn`, `Error`.
pub struct Log<T, Lvl = Warn>(pub T, PhantomData<Lvl>);

impl<U: From<T>, T: fmt::Debug, L> From<T> for Log<U, L> {
    fn from(t: T) -> Self {
        Self(t.into(), PhantomData)
    }
}

impl<T: Dedup, Lvl: LogLevelModifier> Failure for Log<T, Lvl> {
    type Param = (SRes<Time>, Local<'static, HashMap<T::ID, Duration>>);

    const LEVEL: Level = Lvl::LEVEL;

    fn handle_error(
        self,
        (time, mut logged): <Self::Param as SystemParam>::Item<'_, '_>,
        callsite: Option<&'static impl Callsite>,
    ) {
        let cooldown = self.0.cooldown();
        let now = time.elapsed();
        let last_shown = logged.insert(self.0.identify(), now);
        let should_log = last_shown.map_or(true, |d| now < d + cooldown);
        if should_log {
            let meta = callsite.unwrap().metadata();
            if Lvl::LEVEL <= STATIC_MAX_LEVEL && Lvl::LEVEL <= LevelFilter::current() {
                let mut iter = meta.fields().iter();
                bevy_utils::tracing::Event::dispatch(
                    meta,
                    &meta.fields().value_set(&[(
                        &(iter.next().expect("FieldSet corrupted (this is a bug)")),
                        Some(&format_args!("{}", self.0) as &dyn bevy_utils::tracing::field::Value),
                    )]),
                );
            }
        }
    }
}
