## Bevy system error handling 

[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![Latest version](https://img.shields.io/crates/v/bevy_mod_sysfail.svg)](https://crates.io/crates/bevy_mod_sysfail)
[![Apache 2.0](https://img.shields.io/badge/license-Apache-blue.svg)](./LICENSE)
[![Documentation](https://docs.rs/bevy_mod_sysfail/badge.svg)](https://docs.rs/bevy_mod_sysfail/)

Decorate your bevy system with the [`sysfail`] macro attribute to handle failure.

#### Before

```rust
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
    app.add_plugin(bevy::time::TimePlugin)
        .add_system(
            drag_gizmo
                .pipe(print_gizmo_error)
                .in_set(TransformGizmoSystem::Drag),
        )
        .add_system(
            delete_gizmo
                .pipe(|In(_)| {})
                .after(TransformGizmoSystem::Place))
        .add_system(
            place_gizmo
                .pipe(print_gizmo_error)
                .in_set(TransformGizmoSystem::Place)
                .after(TransformGizmoSystem::Drag),
        );
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

```rust
use bevy::prelude::*;
use bevy_mod_sysfail::macros::*;

use thiserror::Error;

#[derive(Error, Debug)]
enum GizmoError {
    #[error("A Gizmo error")]
    Error,
}

fn main() {
    let mut app = App::new();
    app.add_plugin(bevy::time::TimePlugin)
        .add_system(drag_gizmo)
        .add_system(delete_gizmo.after(place_gizmo))
        .add_system(place_gizmo.after(drag_gizmo));
    app.update();
}

#[sysfail(log)]
fn drag_gizmo(time: Res<Time>) -> Result<(), anyhow::Error> {
    println!("drag time is: {}", time.elapsed_seconds());
    let _ = Err(GizmoError::Error)?;
    println!("This will never print");
    Ok(())
}

#[sysfail(log(level = "info"))]
fn place_gizmo() -> Result<(), &'static str> {
    let () = Result::<(), &'static str>::Ok(())?;
    println!("this line should actually show up");
    let _ = Err("Ah, some creative use of info logging I see")?;
    Ok(())
}

#[quick_sysfail]
fn delete_gizmo(time: Res<Time>) {
    println!("delete time is: {}", time.elapsed_seconds());
    let _ = None?;
    println!("This will never print");
}
```

### `sysfail` attribute

[`sysfail`] is an attribute macro you can slap on top of your systems to define
the handling of errors. Unlike `pipe`, this is done directly at the definition
site, and not when adding to the app. As a result, it's easy to see at a glance
what kind of error handling is happening in the system, it also allows using
the system name as a label in system dependency specification.

The [`sysfail`] attribute can only be used on systems returning a type
implementing the [`Failure`] trait. [`Failure`] is implemented for 
`Result<(), impl FailureMode>` and `Option<()>`.
[`sysfail`] takes a single argument, it is one of the following:

- `log`: print the `Err` of the `Result` return value, prints a very
  generic "A none value" when the return type is `Option`.
  By default, most things are logged at `Warn` level, but it is
  possible to customize the log level based on the error value.
- `log(level = "{silent,trace,debug,info,warn,error}")`: This forces
  logging of errors at a certain level (make sure to add the quotes)
- `ignore`: This is like `log(level="silent")` but simplifies the
  generated code.

Note that with `log`, the macro generates a new system with additional
parameters.

### `quick_sysfail` attribute

[`quick_sysfail`] is like `sysfail(ignore)` but only works on `Option<()>`.
This attribute, unlike `sysfail` allows you to elide the final `Some(())`
and the type signature of the system. It's for the maximally lazy, like
me.

```rust
use bevy_mod_sysfail::macros::*;

#[sysfail(ignore)]
fn place_gizmo() -> Option<()> {
  // …
  Some(())
}
// equivalent to:
#[quick_sysfail]
fn quick_place_gizmo() {
  // …
}
```

### Traits

How error is handled is not very customizable, but there is a few behaviors
controllable by the user, always through traits.

#### `Failure` trait

[`Failure`] is implemented for `Result<(), impl FailureMode>` and `Option<()>`.

Systems marked with the [`sysfail`] attribute **must** return a type implementing
[`Failure`].

#### `FailureMode` trait

[`FailureMode`] defines how the failure is handled. By implementing the
trait on your own error types, you can specify:

- What constitutes "distinct" error types.
- The log level of specific values.
- How long an error must not be produced in order to be displayed again.

- [ ] TODO: provide a derive macro that allows setting log level and cooldown.

[`FailureMode`] is implemented for `Box<dyn Error>`, `anyhow::Error`, `()`
and `&'static str`.

#### `LogLevelOverride` trait

[`LogLevelOverride`] is an extension trait that allows you to override the 
log level of a failure. Use the `warn`, `trace`, `debug`, `silent`,
`error` and `info` methods to specify the level of logging of a failure.

### Change log

* `1.0.0`: Update to bevy `0.9`
* `1.1.0`: Allow usage of mutable queries (oops)
* `2.0.0`: **Breaking**: Update to bevy `0.10`

### Version Matrix

| bevy | latest supporting version      |
|------|--------|
| 0.10 | 2.0.0 |
| 0.9  | 1.1.0 |
| 0.8  | 0.1.0 |

## License

Copyright © 2022 Nicola Papale

This software is licensed under Apache 2.0.


[`FailureMode`]: https://docs.rs/bevy_mod_sysfail/1.1.0/bevy_mod_sysfail/trait.FailureMode.html
[`LogLevelOverride`]: https://docs.rs/bevy_mod_sysfail/1.1.0/bevy_mod_sysfail/trait.LogLevelOverride.html
[`Failure`]: https://docs.rs/bevy_mod_sysfail/1.1.0/bevy_mod_sysfail/trait.Failure.html
[`quick_sysfail`]: https://docs.rs/bevy_mod_sysfail/1.1.0/bevy_mod_sysfail/attr.quick_sysfail.html
[`sysfail`]: https://docs.rs/bevy_mod_sysfail/1.1.0/bevy_mod_sysfail/attr.sysfail.html
