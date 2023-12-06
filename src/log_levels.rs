use crate::Level;

/// Modifier for the [`Log`](crate::prelude::Log) and [`LogSimply`](crate::prelude::LogSimply)
/// [`Failure`](crate::Failure)s.
///
/// This let you control the level of logging for errors emitted by a specific system.
pub trait LogLevelModifier: 'static {
    /// The log level.
    const LEVEL: Level;
}

macro_rules! impl_loglevel_modifier {
    ($( $(#[$doc_c:meta])* $name:ident => $log_level:ident ),* $(,)?) => { $(
        $(#[$doc_c])*
        pub struct $name;
        impl LogLevelModifier for $name { const LEVEL: Level = Level::$log_level; }
    )* }
}
impl_loglevel_modifier![
    /// Log with the `TRACE` level, this is similar to `trace!`.
    Trace => TRACE,
    /// Log with the `DEBUG` level, this is similar to `debug!`.
    Debug => DEBUG,
    /// Log with the `INFO` level, this is similar to `info!`.
    Info => INFO,
    /// Log with the `WARN` level, this is similar to `warn!`.
    Warn => WARN,
    /// Log with the `ERROR` level, this is similar to `error!`.
    Error => ERROR,
];
