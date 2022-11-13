use std::error::Error;

use darling::FromMeta;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Paren, AttributeArgs, FnArg, ItemFn,
    ReturnType, Stmt, TypeTuple,
};

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
pub fn sysfail(attr: TokenStream1, input: TokenStream1) -> TokenStream1 {
    // <this> in `#[sys_chain(<this>)]`
    // We expect a function always
    let input = parse_macro_input!(input as ItemFn);
    let attr = parse_macro_input!(attr as AttributeArgs);
    let attr = Sysfail::from_args(&attr).expect("11");
    attr.chain_style().impl_sysfail(input).expect("4").into()
}

#[derive(FromMeta)]
struct Log {
    level: Option<Ident>,
}
#[derive(FromMeta)]
struct Sysfail {
    ignore: Option<()>,
    log: Option<Log>,
}
#[derive(FromMeta)]
struct SysfailAlt {
    log: Option<()>,
}
impl From<SysfailAlt> for Sysfail {
    fn from(alt: SysfailAlt) -> Self {
        Self {
            ignore: None,
            log: alt.log.map(|()| Log { level: None }),
        }
    }
}
impl Sysfail {
    fn from_args(attr: &AttributeArgs) -> Result<Self, Box<dyn std::error::Error>> {
        // handle "log" without arguments.
        let word_log = |_| Ok(SysfailAlt::from_list(attr)?.into());
        Self::from_list(attr).or_else(word_log)
    }
    fn chain_style(&self) -> ChainStyle {
        match (&self.log, self.ignore.is_some()) {
            (Some(Log { level }), false) => match level {
                Some(level) => ChainStyle::from_ident(level),
                None => ChainStyle::no_override(),
            },
            (None, true) => ChainStyle::Ignore,
            (None, false) => todo!("TODO: handle when no explicit chain style is selected"),
            (Some(_), true) => todo!("TODO: handle when too many explicit chain style is selected"),
        }
    }
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
    let mut input = parse_macro_input!(input as ItemFn);
    add_option(&mut input);
    ChainStyle::Ignore.impl_sysfail(input).expect("8").into()
}
fn add_option(system: &mut ItemFn) {
    let stmts = &mut system.block.stmts;
    stmts.push(Stmt::Expr(
        syn::parse2(quote!(Option::Some(()))).expect("9"),
    ));
    system.sig.output = syn::parse2(quote!(-> Option<()>)).expect("10");
}

#[derive(Debug, Clone, Copy)]
enum LogLevel {
    /// Never log anything
    Silent,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
enum ChainStyle {
    Log { set_level: Option<LogLevel> },
    Ignore,
}
impl ChainStyle {
    const fn no_override() -> Self {
        Self::Log { set_level: None }
    }
    fn from_ident(level: &Ident) -> Self {
        let level = match () {
            () if level == "silent" => LogLevel::Silent,
            () if level == "trace" => LogLevel::Trace,
            () if level == "debug" => LogLevel::Debug,
            () if level == "info" => LogLevel::Info,
            () if level == "warn" => LogLevel::Warn,
            () if level == "error" => LogLevel::Error,
            () => todo!("TODO: handle unsupported log level"),
        };
        Self::Log {
            set_level: Some(level),
        }
    }
}

impl ChainStyle {
    /// The system
    fn handler_inputs(&self, sys_output: &ReturnType) -> Result<Vec<FnArg>, Box<dyn Error>> {
        use ChainStyle::{Ignore, Log};
        use LogLevel::Silent;
        let default_output = Box::new(
            TypeTuple {
                paren_token: Paren::default(),
                elems: Punctuated::new(),
            }
            .into(),
        );
        let sys_output = match sys_output {
            ReturnType::Default => &default_output,
            ReturnType::Type(_, ty) => ty,
        };
        match self {
            Ignore
            | Log {
                set_level: Some(Silent),
            } => Ok(Vec::new()),

            Log { .. } => Ok(vec![
                syn::parse2(quote! {
                    mut _last_error_occurence_bevy_mod_sysfail: ::bevy::ecs::prelude::Local<
                        ::bevy::utils::HashMap<
                            < < #sys_output
                                as ::bevy_mod_sysfail::traits::Failure>::Error
                                as ::bevy_mod_sysfail::traits::FailureMode>::ID,
                            ::bevy::utils::Duration,
                        >,
                    >
                })
                .expect("5"),
                syn::parse2(quote!(
                    _time_bevy_mod_sysfail: ::bevy::ecs::prelude::Res<::bevy::time::Time>
                ))
                .expect("6"),
            ]),
        }
    }
    fn handler_body(&self, result: &Ident) -> TokenStream {
        use ChainStyle::{Ignore, Log};
        use LogLevel::Silent;
        match self {
            Ignore
            | Log {
                set_level: Some(Silent),
            } => quote!(let _ = #result;),

            Log { set_level } => {
                let extra = match set_level {
                    Some(LogLevel::Silent) => quote!(.silent()),
                    Some(LogLevel::Trace) => quote!(.trace()),
                    Some(LogLevel::Debug) => quote!(.debug()),
                    Some(LogLevel::Info) => quote!(.info()),
                    Some(LogLevel::Warn) => quote!(.warn()),
                    Some(LogLevel::Error) => quote!(.error()),
                    None => quote!(),
                };
                quote! {
                    use ::bevy_mod_sysfail::traits::*;
                    let current = _time_bevy_mod_sysfail.elapsed();
                    let show_again = |last_show: &::bevy::utils::Duration, cooldown|
                        *last_show < current.saturating_sub(cooldown);
                    if let Some(error) = #result.failure() {
                        let error_id = error.identify();
                        let show_again = |last_show| show_again(last_show, error.cooldown());
                        let last_occurences = &mut _last_error_occurence_bevy_mod_sysfail;
                        if last_occurences.get(&error_id).map_or(true, show_again) {
                            error #extra .log();
                        }
                        last_occurences.insert(error_id, current);
                    }
                }
            }
        }
    }
    fn impl_sysfail(&self, mut ast: ItemFn) -> Result<TokenStream, Box<dyn Error>> {
        let result = Ident::new("result", Span::call_site());
        let chain_body = self.handler_body(&result);
        let system_body = ast.block;
        let system_output = &ast.sig.output;
        ast.sig
            .inputs
            .extend(self.handler_inputs(system_output).expect("7"));
        ast.block = syn::parse2(quote! { {
                let original_system = move || #system_output #system_body ;
                let #result = original_system();
                #chain_body
        } })
        .expect("1");
        ast.sig.output = ReturnType::Default;
        Ok(quote!(#ast))
    }
}
