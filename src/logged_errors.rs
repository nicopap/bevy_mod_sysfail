use bevy_utils::{Duration, HashMap};

use crate::Failure;

type ErrorId<T> = <<T as Failure>::Error as crate::FailureMode>::ID;

/// Tracks when specific errors were logged.
pub struct LoggedErrors<T: Failure>(pub(crate) HashMap<ErrorId<T>, Duration>);

impl<T: Failure> Default for LoggedErrors<T> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}
