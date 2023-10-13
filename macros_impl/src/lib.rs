use proc_macro::TokenStream as TokenStream1;
use syn::parse_macro_input;

mod generate;

/// `sysfail` is an attribute macro you can slap on top of your systems to define
/// the handling of errors.
///
/// If you are lazy and don't care about the return value, use [`quick_sysfail`].
///
/// Unlike `chain`, this is done directly at the definition
/// site, and not when adding to the app. As a result, it's easy to see at a glance
/// what kind of error handling is happening in the system, it also allows using
/// the system name as a label in system dependency specification.
///
/// The `sysfail` attribute can only be used on systems returning a type
/// implementing the `Failure` trait. `Failure` is implemented for
/// `sysfail` takes a single argument, it is one of the following:
///
/// - `log`: print the `Err` of the `Result` return value. Prints a very
///   generic "A none value" when the return type is `Option`.
///   By default, most things are logged at `Warn` level, but it is
///   possible to customize the log level based on the error value.
/// - `log(level = "{silent,trace,debug,info,warn,error}")`: This forces
///   logging of errors at a certain level (make sure to add the quotes)
/// - `ignore`: This is like `log(level="silent")` but simplifies the
///   generated code.
///
/// Note that with `log`, the macro generates a new system with additional
/// parameters.
///
/// [`quick_sysfail`]: macro@quick_sysfail
#[proc_macro_attribute]
pub fn sysfail(attrs: TokenStream1, input: TokenStream1) -> TokenStream1 {
    let mut config = generate::FnConfig::new();

    let config_parser = syn::meta::parser(|meta| config.parse(meta));
    parse_macro_input!(attrs with config_parser);

    let input = parse_macro_input!(input as syn::ItemFn);
    generate::sysfail(config, input).into()
}

/// `quick_sysfail` is like [`sysfail(ignore)`] but only works on `Option<()>`.
///
/// This attribute, unlike `sysfail` allows you to elide the final `Some(())`
/// and the type signature of the system. It's for the maximally lazy, like
/// me.
///
/// ## Example
///
/// ```rust
/// use bevy_mod_sysfail::macros::*;
///
/// #[sysfail(ignore)]
/// fn place_gizmo() -> Option<()> {
///   // …
///   Some(())
/// }
/// // equivalent to:
/// #[quick_sysfail]
/// fn quick_place_gizmo() {
///   // …
/// }
/// ```
///
/// [`sysfail(ignore)`]: macro@sysfail
#[proc_macro_attribute]
pub fn quick_sysfail(_: TokenStream1, input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as syn::ItemFn);
    generate::sysfail(generate::FnConfig::quick(), input).into()
}
