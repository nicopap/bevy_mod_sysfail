use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

pub struct FnConfig {
    pub error_type: syn::Type,
}
impl FnConfig {
    pub fn new() -> Self {
        Self {
            error_type: parse_quote![
                ::bevy_mod_sysfail::prelude::Log<::std::boxed::Box<dyn ::std::error::Error>>
            ],
        }
    }
}

const QUICK_MSG: &str = "#[sysfail] systems have no return types.";

fn is_log(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(syn::TypePath{path, ..})
        if path.segments.last().is_some_and(|p| p.ident.to_string().contains("Log"))
    )
}

pub fn sysfail(config: &FnConfig, function: syn::ItemFn) -> TokenStream {
    match sysfail_inner(config, function) {
        Ok(token_stream) => token_stream,
        Err(syn_error) => syn_error.into_compile_error(),
    }
}
fn sysfail_inner(config: &FnConfig, mut function: syn::ItemFn) -> syn::Result<TokenStream> {
    if !matches!(function.sig.output, syn::ReturnType::Default) {
        return Err(syn::Error::new_spanned(function.sig.output, QUICK_MSG));
    }
    let ret_type = &config.error_type;
    let body = &function.block.stmts;
    let vis = &function.vis;
    let fn_ident = &function.sig.ident;

    // Add comma at end so that we can add the #extra_params
    if !function.sig.inputs.is_empty() && !function.sig.inputs.trailing_punct() {
        function.sig.inputs.push_punct(syn::token::Comma::default());
    }
    let params = &function.sig.inputs;
    let params_gen = &function.sig.generics.params;
    let where_gen = &function.sig.generics.where_clause;
    let attrs = &function.attrs;
    let prefix = quote!(::bevy_mod_sysfail::__macro);
    let callsite = if is_log(ret_type) {
        quote! {Some({
            static META: #prefix::Metadata<'static> = #prefix::Metadata::new(
                concat!(file!(), ":", line!()),
                concat!(module_path!(), "::", stringify!(#fn_ident)),
                <#ret_type as #prefix::Failure>::LEVEL,
                Some(file!()),
                Some(line!()),
                Some(concat!(module_path!(), "::", stringify!(#fn_ident))),
                #prefix::FieldSet::new(&["message"], #prefix::Identifier(match &CALLSITE {
                    None => panic!(),
                    Some(c) => c,
                })),
                #prefix::metadata::Kind::EVENT,
            );
            #prefix::DefaultCallsite::new(&META)
        })}
    } else {
        quote!(None)
    };
    Ok(quote! {
        #(#attrs)*
        #vis fn #fn_ident <#params_gen> (
            #params
            __sysfail_params: #prefix::StaticSystemParam<<#ret_type as #prefix::Failure>::Param>
        ) #where_gen {
            use ::bevy_mod_sysfail::Failure;
            let mut inner_system = move || -> ::core::result::Result<(), #ret_type> {
                #(#body)*;
                return ::core::result::Result::Ok(());
            };
            if let Err(err) = inner_system() {
                static CALLSITE: Option<#prefix::DefaultCallsite> = #callsite;
                err.handle_error(__sysfail_params.into_inner(), CALLSITE.as_ref());
            }
        }
    })
}
