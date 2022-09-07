## Bevy system error handling macros


Decorate your system with the `failable` macro attribute
to make them handle cleanly failure mods.

#### Before

```rust
fn main() {
  // app boilerplate
  app.add_system(
    drag_gizmo
      .chain(print_gizmo_error)
      .label(TransformGizmoSystem::Drag)
      .before(TransformSystem::TransformPropagate),
  )
  .add_system(
    place_gizmo
      .chain(print_gizmo_error)
      .label(TransformGizmoSystem::Place)
      .after(TransformSystem::TransformPropagate)
      .after(TransformGizmoSystem::Drag),
  )
  .add_system(
    delete_gizmo
      .chain(different_error_handling)
      .after(TransformGizmoSystem::Place),
  );
  app.run();
}

fn print_gizmo_error(
    In(result): In<Result<(), GizmoError>>,
    mut last_error_occurence: Local<HashMap<GizmoError, Duration>>,
    time: Res<Time>,
) {
  // error boilerplate, may include
  // - avoiding printing multiple times the same error
  // - Formatting and chosing the log level
}

fn different_error_handling(
    In(result): In<Result<(), GizmoError>>,
    mut last_error_occurence: Local<HashMap<GizmoError, Duration>>,
    time: Res<Time>,
) {
  // A different, custom handling of errors
}

fn drag_gizmo(…) -> Result<(), GizmoError> {
  // …
  Ok(())
}
fn place_gizmo(…) -> Result<(), GizmoError> {
  // …
  Ok(())
}
fn delete_gizmo(…) -> Result<(), GizmoError> {
  // …
  Ok(())
}
```

#### After

```rust
fn main() {
  app.add_system(
    drag_gizmo.before(TransformSystem::TransformPropagate),
  )
  .add_system(
    place_gizmo
      .after(TransformSystem::TransformPropagate)
      .after(drag_gizmo),
  )
}

type DifferentErrorHandlingType = SystemType!(
  fn different_error_handling(
      In(result): In<Result<(), GizmoError>>,
      mut last_error_occurence: Local<HashMap<GizmoError, Duration>>,
      time: Res<Time>,
  )
);
fn different_error_handling(
    In(result): In<Result<(), GizmoError>>,
    mut last_error_occurence: Local<HashMap<GizmoError, Duration>>,
    time: Res<Time>,
) {
  // A different, custom handling of errors
}

#[failable(log)]
fn drag_gizmo(…) -> Result<(), GizmoError> {
  // …
  Ok(())
}
#[failable(log)]
fn place_gizmo(…) -> Result<(), GizmoError> {
  // …
  Ok(())
}
#[failable(system(different_error_handling: DifferentErrorHandlingType))]
fn delete_gizmo(…) -> Result<(), GizmoError> {
  // …
  Ok(())
}
```

Under the hood `failable` defines the system inside another system.

### `failable` attribute arguments

- `log`: print the `Err` side of `Result` returns,
  and the system name when `None`. Equivalent to `level(LogLevel::Warn)`
- `ignore`: just do literally nothing with the return value.
- `level = l`, l: `LogLevel`: Print errors with a provided level.
- `cooldown = d`, d: `Duration`: How much time to allow between reprinting
  of the same error, defaults to one second.
- `system(s: s_type)`, s: `impl SystemParamFunction`, s_type: type of `s`:
  Use a custom system provided as argument. The system type must be provided
  for this to work. This is a limitation of rust. For your convinience, the
  `SystemType!` macro let you copy/paste the system's type signature.
- You can combine `level` and `cooldown` as follow:
  `#[failable(level = LogLevel::Warn, cooldown = Duration::from_secs(10.0)))]`.

Note that if the provided handling system as more than the `In` parameter,
or if you are not using the `ignore` option, the system will have a different
type signature than the one declared.

### `FailureMode` trait

```rust
enum LogLevel {
  /// Never log anything
  Silent,
  Trace,
  Debug,
  Info,
  Warn,
  Error,
}
trait FailureMode {
  fn log_level(&self) -> LogLevel;
}
```

`FailureMode` let you chose the log level of your errors in the `log`
error handling style.

Anything that implements `Error` will implement `FailureMode`
and be logged with `warn!`. `warn!` is the default because the
the app keeps working even after the early return.

`bevy_mod_system_tools` provides an extension trait to `Result` and `Option` to change
the log level of error. Use the `warn`, `trace`, `debug`, `silent` and `info` specify
the level of logging of an error.

```rust
#[failable(default)]
fn place_gizmo(
  mut baz: Query<&mut Baz>,
  bar: Query<&Bar>,
  oz: Query<&Oz>,
) {
  // …
  let mut my_baz = baz.get_single_mut().trace()?;
  let my_bar = bar.get_single()?;
  let my_oz = oz.get_single().silent()?;
  // …
}
```