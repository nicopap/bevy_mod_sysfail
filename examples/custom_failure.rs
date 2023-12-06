use std::fmt;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_mod_sysfail::prelude::*;
use bevy_mod_sysfail::{Callsite, Level, LogLevelModifier};
use thiserror::Error;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_debug_text_overlay::OverlayPlugin { font_size: 23.0, ..default() })
        .add_systems(Startup, setup)
        .add_systems(Update, (print_generic, print_specialized))
        .run();
}

#[derive(Error, Debug)]
enum CustomError {
    #[error("A Zoob error")]
    Zoob,
    #[error("Do the bonzo!")]
    Bonzo,
    #[error("The Zartrub was located at {0:#?}")]
    Zartrub(Transform),
}

pub struct ScreenLog<T, Lvl = Info>(pub T, PhantomData<fn(Lvl)>);

impl<U: From<T>, T: fmt::Debug, L> From<T> for ScreenLog<U, L> {
    fn from(t: T) -> Self {
        Self(t.into(), PhantomData)
    }
}

impl<T: fmt::Display, Lvl: LogLevelModifier> Failure for ScreenLog<T, Lvl> {
    type Param = ();

    const LEVEL: Level = Lvl::LEVEL;

    fn handle_error(self, _: (), callsite: Option<&'static impl Callsite>) {
        use bevy_debug_text_overlay::{InvocationSiteKey, COMMAND_CHANNELS};
        let metadata = callsite.unwrap().metadata();
        let key = InvocationSiteKey {
            file: metadata.file().unwrap(),
            line: metadata.line().unwrap(),
            column: 0,
        };
        let color = match *metadata.level() {
            Level::ERROR => Color::RED,
            Level::WARN => Color::ORANGE,
            Level::DEBUG => Color::BLUE,
            Level::TRACE => Color::PURPLE,
            _ => Color::GREEN,
        };
        COMMAND_CHANNELS.refresh_text(key, || format!("{}", self.0), 1., Some(color));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// Usage:
#[sysfail(ScreenLog<CustomError, Warn>)]
fn print_specialized(time: Res<Time>) {
    let delta = time.delta_seconds_f64();
    let current_time = time.elapsed_seconds_f64();
    let at_interval = |t: f64| current_time % t < delta;
    if at_interval(6.) {
        let transform = Transform::from_translation(Vec3::splat(current_time as f32));
        let _ = Err(CustomError::Zartrub(transform))?;
    }
}

#[sysfail(ScreenLog<anyhow::Error>)]
fn print_generic(time: Res<Time>) {
    let delta = time.delta_seconds_f64();
    let current_time = time.elapsed_seconds_f64();
    let at_interval = |t: f64| current_time % t < delta;
    if at_interval(3.) {
        let _ = Err(CustomError::Zoob)?;
    }
    if at_interval(5.) {
        let _ = Err(CustomError::Bonzo)?;
    }
}
