# `6.0.0`

Major **Breaking** release.

- Removed all deprecated methods
- Remove `#[quick_sysfail]` in favor of `#[sysfail(Ignore)]`
- Remove support for `Option<()>`. Consider replacing `option?` by `option.ok(())?`
- Replace the `log = "foo"` syntax by `Log<ErrorType, Foo>`.
- Automatically add return type to system. This means that you should remove
  the return type from your `#[sysfail]` systems.
- `Failure` is now a trait on the **error type** returned by the `#[sysfail]`
  system, rather than the whole return type.
- `Failure` now has an associated type: `Param`. It allows accessing arbitrary
  system parameters in the error handling code.
- Renamed `FailureMode` to `Dedup`.
- Now `bevy_mod_sysfail` directly depends on `bevy`, rather than its subcrates
- Added the `full` features, enabled by default. Disabling removes the `bevy`
  dependency, to only depend on `bevy_ecs`, but at the cost of removing
  the `Log` `Failure` definition.
- Added the `Emit` `Failure`, which sends `Err`s as bevy `Event`s.
- Added `LogSimply`, a variant of `Log` that works without `Res<Time>`
- Added example showing how to extend with your own behavior the `Failure` trait.
- Added the system name to the log message's "target" field (by default, this is
  the bit of text before the "ERROR" colored text)
- Added `#[exclusive_sysfail]`, works like `#[sysfail]` but is fully supported
  by exclusive systems. It only supports `Failure`s where `Param = ()`

# `4.1.0`

* fix macro dependency of the non-macro crate

# `4.0.0`

* Now the module reported in the error message is the one the system is in
* Deprecated `OverrideLevel` and other runtime logging levels, this makes
  implementation much easier
* **Breaking**: Now, runtime logging levels are completely ignored.
* **Breaking**: The default error level for `&str` and `Option` stuff has been set to `error`.
* Remove the `darling` dependency
* Update `syn` to version 2, this reduces cold compile times

# `3.0.0`

**Breaking**: Update to bevy `0.11`

# `2.0.0`

**Breaking**: Update to bevy `0.10`

# `1.1.0`

Allow usage of mutable queries (oops)

## `1.0.0`

Update to bevy `0.9`

