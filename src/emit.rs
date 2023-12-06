use bevy_ecs::event::{Event, EventWriter};

use crate::{Callsite, Failure, Level};

/// As the `Err` of the return value of a `sysfail` system, send the `E` event.
pub struct Emit<E>(pub E);

impl<E> From<E> for Emit<E> {
    fn from(value: E) -> Self {
        Self(value)
    }
}

impl<E: Event + 'static> Failure for Emit<E> {
    type Param = EventWriter<'static, E>;

    const LEVEL: Level = Level::INFO;

    fn handle_error(self, mut event_writer: EventWriter<E>, _: Option<&'static impl Callsite>) {
        event_writer.send(self.0);
    }
}
