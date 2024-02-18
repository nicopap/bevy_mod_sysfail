## Bevy system error handling 

[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![Latest version](https://img.shields.io/crates/v/bevy_mod_sysfail.svg)](https://crates.io/crates/bevy_mod_sysfail)
[![Apache 2.0](https://img.shields.io/badge/license-Apache-blue.svg)](./LICENSE)
[![Documentation](https://docs.rs/bevy_mod_sysfail/badge.svg)](https://docs.rs/bevy_mod_sysfail/)

Decorate your bevy system with the [`sysfail`] macro attribute to handle failure.

#### Before

```rust,no_run
use bevy::prelude::*;
use bevy::utils::Duration;

use thiserror::Error;

#[derive(Error, Debug)]
enum GizmoError {
    #[error("A Gizmo error")]
    Error,
}

#[derive(Debug, PartialEq, Eq, Hash, SystemSet, Clone)]
enum TransformGizmoSystem { Drag, Place }

fn main() {
    let mut app = App::new();
    app.add_plugins(bevy::time::TimePlugin)
        .add_systems(Update, (
            drag_gizmo
                .pipe(print_gizmo_error)
                .in_set(TransformGizmoSystem::Drag),
            delete_gizmo
                .pipe(|In(_)| {})
                .after(TransformGizmoSystem::Place),
            place_gizmo
                .pipe(print_gizmo_error)
                .in_set(TransformGizmoSystem::Place)
                .after(TransformGizmoSystem::Drag),
        ));
    app.update();
}

fn print_gizmo_error(
    In(result): In<Result<(), Box<dyn std::error::Error>>>,
    mut last_error_occurence: Local<Option<Duration>>,
    time: Res<Time>,
) {
  // error boilerplate, may include
  // - avoiding printing multiple times the same error
  // - Formatting and chosing the log level
}

fn drag_gizmo(time: Res<Time>) -> Result<(), Box<dyn std::error::Error>> {
    println!("drag time is: {}", time.elapsed_seconds());
    let _ = Err(GizmoError::Error)?;
    println!("This will never print");
    Ok(())
}

fn place_gizmo() -> Result<(), Box<dyn std::error::Error>> {
    let () = Result::<(), &'static str>::Ok(())?;
    println!("this line should actually show up");
    let _ = Err("Ah, some creative use of info logging I see")?;
    Ok(())
}

fn delete_gizmo(time: Res<Time>) -> Option<()> {
    println!("delete time is: {}", time.elapsed_seconds());
    let _ = None?;
    println!("This will never print");
    Some(())
}
```

#### After

```rust,no_run
use bevy::prelude::*;
use bevy_mod_sysfail::prelude::*;

use thiserror::Error;

#[derive(Error, Debug)]
enum GizmoError {
    #[error("A Gizmo error")]
    Error,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(bevy::time::TimePlugin)
        .add_systems(Update, (
            drag_gizmo,
            delete_gizmo.after(place_gizmo),
            place_gizmo.after(drag_gizmo)
        ));
    app.update();
}

#[sysfail]
fn drag_gizmo(time: Res<Time>) {
    println!("drag time is: {}", time.elapsed_seconds());
    let _ = Err(GizmoError::Error)?;
    println!("This will never print");
}

#[sysfail(Log<&'static str, Info>)]
fn place_gizmo() {
    let () = Result::<(), &'static str>::Ok(())?;
    println!("this line should actually show up");
    let _ = Err("Ah, some creative use of info logging I see")?;
}

#[sysfail(Ignore)]
fn delete_gizmo(time: Res<Time>) {
    println!("delete time is: {}", time.elapsed_seconds());
    let _ = Err(342_i32)?;
    println!("This will never print");
}
```

### `sysfail` attribute

[`sysfail`] is an attribute macro you can slap on top of your systems to define
the handling of errors. Unlike `pipe`, this is done directly at the definition
site, and not when adding to the app. As a result, it's easy to see at a glance
what kind of error handling is happening in the system, it also allows using
the system name as a label in system dependency specification.

`sysfail(E)` systems return a value of type `Result<(), E>`. The return type
is added by the macro, so do not add it yourself!

`E` is a type that implements the `Failure` trait. `bevy_mod_sysfail` exports
several types that implement `Failure`:

- [`Log<Err, Lvl = Warn>`][`Log`]: Will log `Err` to the tracing logger.
   - The first type parameter `Err` implements the [`Dedup`] trait. You can
     implement `Dedup` for your own types, but you can always use the
     `anyhow::Error`, `Box<dyn std::error::Error>` and `&'static str` types,
     as those already implement `Dedup`.
   - The second type parameter specifies the level of the log. It is optional
     and by default it is `Warn`
- [`LogSimply`]: Is similar to `Log`, but without deduplication.
- [`Emit<Ev>`][`Emit`]: Will emit the `Ev` bevy [`Event`] whenever the system returns an `Err`
- [`Ignore`]: Ignore errors, do as if nothing happened.

Example usages:

```rust
use bevy::prelude::*;
use bevy_mod_sysfail::prelude::*;
use thiserror::Error;

// -- Log a value --

#[derive(Error, Debug)]
enum MyCustomError {
    #[error("A Custom error")]
    Error,
}

// Equivalent to #[sysfail(Log<Box<dyn std::error::Error>>)]
#[sysfail]
fn generic_failure() { /* ... */ }

#[sysfail(Log<&'static str>)]
fn log_a_str_message() {
    let _ = Err("Yep, just like that")?;
}

#[sysfail(Log<anyhow::Error>)]
fn log_an_anyhow_error() {
    let _ = Err(MyCustomError::Error)?;
}

#[sysfail(LogSimply<MyCustomError, Trace>)]
fn log_trace_on_failure() { /* ... */ }

#[sysfail(LogSimply<MyCustomError, Error>)]
fn log_error_on_failure() { /* ... */ }

// -- Emit an event --
use bevy::app::AppExit;

#[derive(Event)]
enum ChangeMenu {
    Main,
    Tools,
}

#[sysfail(Emit<ChangeMenu>)]
fn change_menu() { /* ... */ }

#[sysfail(Emit<AppExit>)]
fn quit_app_on_error() { /* ... */ }

// -- Ignore all errors --

#[sysfail(Ignore)]
fn do_not_care_about_failure() { /* ... */ }
```

### Exclusive systems

For exclusive systems, use the `#[exclusive_sysfail]` macro. Note that only
`Failure<Param = ()>` work with exclusive systems. This excludes `Log`, so
make sure to use `LogSimply` instead.

### Custom handling

`bevy_mod_sysfail` is not limited to the predefined set of `Failure`s, you can
define your own by implementing it yourself.
See the [custom_failure example] for sample code.

### Change log

See the [CHANGELOG].

### Version Matrix

| bevy | latest supporting version      |
|------|--------|
| 0.13 | 7.0.0 |
| 0.12 | 6.0.0 |
| 0.11 | 4.3.0 |
| 0.10 | 2.0.0 |
| 0.9  | 1.1.0 |
| 0.8  | 0.1.0 |

## License

Copyright © 2022 Nicola Papale

This software is licensed under Apache 2.0.

[CHANGELOG]: https://github.com/nicopap/bevy_mod_sysfail/blob/v7.0.0/CHANGELOG.md
[custom_failure example]: https://github.com/nicopap/bevy_mod_sysfail/blob/v7.0.0/examples/custom_failure.rs
[`Dedup`]: https://docs.rs/bevy_mod_sysfail/7.0.0/bevy_mod_sysfail/trait.Dedup.html
[`Failure`]: https://docs.rs/bevy_mod_sysfail/7.0.0/bevy_mod_sysfail/trait.Failure.html
[`sysfail`]: https://docs.rs/bevy_mod_sysfail/7.0.0/bevy_mod_sysfail/attr.sysfail.html
[`Emit`]: https://docs.rs/bevy_mod_sysfail/7.0.0/bevy_mod_sysfail/prelude/struct.Emit.html
[`Log`]: https://docs.rs/bevy_mod_sysfail/7.0.0/bevy_mod_sysfail/prelude/struct.Log.html
[`LogSimply`]: https://docs.rs/bevy_mod_sysfail/7.0.0/bevy_mod_sysfail/prelude/struct.LogSimply.html
[`Ignore`]: https://docs.rs/bevy_mod_sysfail/7.0.0/bevy_mod_sysfail/prelude/struct.Ignore.html
[`Event`]: https://docs.rs/bevy/0.12/bevy/ecs/event/trait.Event.html
