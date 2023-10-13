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

