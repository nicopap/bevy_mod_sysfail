use crate::{Callsite, Failure, Level};

/// Do nothing with errors in `#[sysfail]` systems.
pub struct Ignore;

impl<T: std::fmt::Debug> From<T> for Ignore {
    fn from(_: T) -> Self {
        Self
    }
}

impl Failure for Ignore {
    type Param = ();

    const LEVEL: Level = Level::TRACE;

    fn handle_error(self, (): (), _: Option<&'static impl Callsite>) {}
}
