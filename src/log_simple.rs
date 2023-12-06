use std::{fmt, marker::PhantomData};

use bevy_utils::tracing::level_filters::{LevelFilter, STATIC_MAX_LEVEL};

use crate::{log_levels::Warn, Callsite, Failure, Level, LogLevelModifier};

/// Similar to [`Log`](crate::prelude::Log), but doesn't have any deduplication handling.
///
/// This is useful in exclusive systems, as this doesn't require a `Res<Time>`
/// parameter.
///
/// However, if the same system returns an `Err` each frame, you will be _flooded_
/// with error messages, so be warned.
pub struct LogSimply<T, Lvl = Warn>(pub T, PhantomData<Lvl>);

impl<U: From<T>, T: fmt::Debug, L> From<T> for LogSimply<U, L> {
    fn from(t: T) -> Self {
        Self(t.into(), PhantomData)
    }
}

impl<T: fmt::Display, Lvl: LogLevelModifier> Failure for LogSimply<T, Lvl> {
    type Param = ();

    const LEVEL: Level = Lvl::LEVEL;

    fn handle_error(self, (): (), callsite: Option<&'static impl Callsite>) {
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
