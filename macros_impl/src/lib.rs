use std::error::Error;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Paren, AttributeArgs, Expr, FnArg, ItemFn,
    Lit, Meta, MetaNameValue, NestedMeta, Path, PathSegment, ReturnType, Type, TypeTuple,
};

#[proc_macro_attribute]
pub fn sys_chain(attr: TokenStream1, input: TokenStream1) -> TokenStream1 {
    // <this> in `#[sys_chain(<this>)]`
    let attr = parse_macro_input!(attr as AttributeArgs);
    // We expect a function always
    let input = parse_macro_input!(input as ItemFn);
    ChainStyle::try_from(attr)
        .expect("3")
        .impl_failable(input)
        .expect("4")
        .into()
}

macro_rules! path_ident {
    ($value:expr, $ident_name:literal) => {
        matches!(
            $value,
            NestedMeta::Meta(Meta::Path( Path { segments, .. } ))
                if segments.first().map_or(false, |t| t.ident == $ident_name)
        )
    }
}
macro_rules! foo {
    ($value:expr, $ident_name:literal) => {
        matches!(
            $value,
            NestedMeta::Meta(Meta::NameValue( MetaNameValue {
                path: Path { segments, .. },
                ..
            } ))
                if segments.first().map_or(false, |t| t.ident == $ident_name)
        )
    }
}
macro_rules! bar {
    ($value:expr, |$binding:ident| $body:expr) => {
        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue { lit: $binding, .. })) = $value {
            $body
        } else {
            unreachable!()
        }
    };
}

enum ChainStyle {
    Log,
    Ignore,
    // System(Expr, Type),
    Level(Lit),
    Cooldown(Lit),
    // Both { level: Expr, cooldown: Expr },
}
impl TryFrom<AttributeArgs> for ChainStyle {
    type Error = ();
    fn try_from(args: AttributeArgs) -> Result<Self, ()> {
        // in `#[sys_chain(<a>, <b>)]`, args has two values
        let value = args.first().ok_or(())?;
        match () {
            () if path_ident!(value, "log") => Ok(ChainStyle::Log),
            () if path_ident!(value, "ignore") => Ok(ChainStyle::Ignore),
            () if foo!(value, "cooldown") => {
                bar!(value, |cooldown| Ok(ChainStyle::Cooldown(cooldown.clone())))
            }
            () if foo!(value, "level") => bar!(value, |level| Ok(ChainStyle::Level(level.clone()))),
            _ => Err(()),
        }
    }
}

impl ChainStyle {
    /// The system
    fn handler_inputs(&self, sys_output: &ReturnType) -> Result<Vec<FnArg>, Box<dyn Error>> {
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
            ChainStyle::Log => Ok(vec![
                syn::parse(
                    quote! {
                    mut last_error_occurence: ::bevy::ecs::prelude::Local<
                        ::bevy::utils::HashMap<
                            < < #sys_output
                                as ::bevy_mod_system_tools::traits::Failure>::Error
                                as ::bevy_mod_system_tools::traits::FailureMode>::ID,
                            ::bevy::utils::Duration,
                        >,
                    >
                    }
                    .into(),
                )
                .expect("5"),
                syn::parse(quote!(time: ::bevy::ecs::prelude::Res<::bevy::time::Time>).into())
                    .expect("6"),
            ]),
            ChainStyle::Ignore => Ok(Vec::new()),
            ChainStyle::Level(_) => Ok(Vec::new()),
            ChainStyle::Cooldown(_) => Ok(Vec::new()),
        }
    }
    fn handler_body(&self, result: &Ident) -> TokenStream {
        let default_cooldown = quote!(::bevy::utils::Duration::from_secs(1));
        match self {
            ChainStyle::Log => quote! {
                use ::bevy_mod_system_tools::{traits::*, LogLevel};
                let current = time.time_since_startup();
                let show_again = |last_show: &::bevy::utils::Duration|
                    *last_show < current.saturating_sub(#default_cooldown);
                if let Some(error) = #result.error() {
                    let error_id = error.identify();
                    if last_error_occurence.get(&error_id).map_or(true, show_again) {
                        error.log();
                    }
                    last_error_occurence.insert(error_id, current);
                }
            },
            ChainStyle::Ignore => quote!(let _ = #result;),
            ChainStyle::Level(_) => todo!(),
            ChainStyle::Cooldown(_) => todo!(),
        }
    }
    fn impl_failable(&self, mut ast: ItemFn) -> Result<TokenStream, Box<dyn Error>> {
        let result = Ident::new("result", Span::call_site());
        let chain_body = self.handler_body(&result);
        let system_body = ast.block;
        let system_output = &ast.sig.output;
        ast.sig
            .inputs
            .extend(self.handler_inputs(system_output).expect("7"));
        ast.block = syn::parse(
            quote! { {
                    let original_system = move || #system_output #system_body ;
                    let #result = original_system();
                    #chain_body
            } }
            .into(),
        )
        .expect("1");
        ast.sig.output = ReturnType::Default;
        Ok(quote!(#ast))
    }
}
