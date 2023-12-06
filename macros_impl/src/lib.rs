#![doc = include_str!("../README.md")]
use proc_macro::TokenStream as TokenStream1;
use syn::parse_macro_input;

mod generate;

/// `sysfail` is an attribute macro you can slap on top of your systems to define
/// the handling of errors.
///
/// See [`macro@exclusive_sysfail`] for **exclusive systems** handling.
#[proc_macro_attribute]
pub fn sysfail(attrs: TokenStream1, input: TokenStream1) -> TokenStream1 {
    let mut config = generate::FnConfig::new();

    if !attrs.is_empty() {
        config.error_type = parse_macro_input!(attrs as syn::Type);
    }
    let input = parse_macro_input!(input as syn::ItemFn);
    generate::sysfail(&config, input).into()
}
/// Similar to [`macro@sysfail`] but allows usage on **exclusive systems**.
#[proc_macro_attribute]
pub fn exclusive_sysfail(attrs: TokenStream1, input: TokenStream1) -> TokenStream1 {
    let mut config = generate::FnConfig::new();
    config.exclusive = true;

    if !attrs.is_empty() {
        config.error_type = parse_macro_input!(attrs as syn::Type);
    }
    let input = parse_macro_input!(input as syn::ItemFn);
    generate::sysfail(&config, input).into()
}
