use std::mem;

use proc_macro2::TokenStream;
use quote::quote;
use syn::meta::ParseNestedMeta;

#[derive(Clone, Copy)]
enum MacroLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Silent,
    Quick,
}

pub struct FnConfig {
    level: MacroLogLevel,
}
impl FnConfig {
    pub fn new() -> Self {
        Self { level: MacroLogLevel::Error }
    }
    pub fn quick() -> Self {
        Self { level: MacroLogLevel::Quick }
    }
    pub fn parse(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        match () {
            () if meta.path.is_ident("log") && meta.input.peek(syn::token::Paren) => {
                meta.parse_nested_meta(|meta| self.parse_level(meta))
            }
            () if meta.path.is_ident("log") => Ok(()),
            () if meta.path.is_ident("ignore") => {
                self.level = MacroLogLevel::Silent;
                Ok(())
            }
            () => {
                let msg = "Only the 'log' and 'ignore' meta attributes are supported for sysfail";
                Err(meta.error(msg))
            }
        }
    }
    fn parse_level(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        if !meta.path.is_ident("level") {
            let msg = "Only the 'level' attribute is supported for sysfail(log(…))";
            return Err(meta.error(msg));
        }
        let literal: syn::LitStr = meta.value()?.parse()?;
        let ident: syn::Ident = literal.parse()?;
        let msg = "invalid log level, available: silent, trace, debug, info, warn, error";
        match () {
            () if ident == "trace" => self.level = MacroLogLevel::Trace,
            () if ident == "debug" => self.level = MacroLogLevel::Debug,
            () if ident == "info" => self.level = MacroLogLevel::Info,
            () if ident == "warn" => self.level = MacroLogLevel::Warn,
            () if ident == "error" => self.level = MacroLogLevel::Error,
            () if ident == "silent" => self.level = MacroLogLevel::Silent,
            () => return Err(meta.error(msg)),
        };
        Ok(())
    }
}

const QUICK_MSG: &str =
    "quick_sysfail systems must have no return type, the macro adds Option<()>.";
const NO_RET_MSG: &str = "sysfail systems must have a return type.\n\
 - Do not use `sysfail` if the system doesn't fail\n\
 - Add the `Result` type if the system can fail, it will be removed by the macro\n\
 - Use the `quick_sysfail` if the return type is `Option<()>` and you just want to skip error handling";

fn option_trailing_stmt() -> syn::Stmt {
    syn::parse_quote!(return ::core::option::Option::Some(());)
}
fn option_ret_type() -> syn::ReturnType {
    syn::parse_quote!(-> ::core::option::Option<()>)
}
fn extra_params(ret_type: &syn::Type) -> TokenStream {
    quote! {
        __sysfail_time: ::bevy_mod_sysfail::__macro::TimeParam,
        mut __sysfail_logged_errors: ::bevy_mod_sysfail::__macro::LoggedErrorsParam<#ret_type>,
    }
}
pub fn sysfail(config: FnConfig, function: syn::ItemFn) -> TokenStream {
    match sysfail_inner(config, function) {
        Ok(token_stream) => token_stream,
        Err(syn_error) => syn_error.into_compile_error(),
    }
}
fn sysfail_inner(config: FnConfig, mut function: syn::ItemFn) -> syn::Result<TokenStream> {
    use MacroLogLevel::{Quick, Silent};

    if matches!(config.level, Quick) {
        if !matches!(function.sig.output, syn::ReturnType::Default) {
            return Err(syn::Error::new_spanned(function, QUICK_MSG));
        }
        function.sig.output = option_ret_type();
    }
    let mut body = mem::take(&mut function.block.stmts);
    if matches!(config.level, Quick) {
        body.push(option_trailing_stmt());
    }
    let ret_type = match &function.sig.output {
        syn::ReturnType::Default => return Err(syn::Error::new_spanned(function, NO_RET_MSG)),
        syn::ReturnType::Type(_, ret_type) => &**ret_type,
    };
    let vis = &function.vis;
    let fn_ident = &function.sig.ident;

    // Add comma at end so that we can add the #extra_params
    if !function.sig.inputs.is_empty() && !function.sig.inputs.trailing_punct() {
        function.sig.inputs.push_punct(syn::token::Comma::default());
    }
    let extra_params = (!matches!(config.level, Quick | Silent)).then(|| extra_params(ret_type));
    let params = &function.sig.inputs;
    let params_gen = &function.sig.generics.params;
    let where_gen = &function.sig.generics.where_clause;
    let attrs = &function.attrs;
    let log_error = log_error(config.level);
    Ok(quote! {
        #(#attrs)*
        #vis fn #fn_ident <#params_gen> (#params #extra_params) #where_gen {
            let mut inner_system = move || -> #ret_type { #(#body)* };
            let result = inner_system();
            #log_error
        }
    })
}

fn log_error(level: MacroLogLevel) -> TokenStream {
    let simple_log = |level: TokenStream| {
        quote! {
            if let Some(error) = ::bevy_mod_sysfail::Failure::get_error(result, &*__sysfail_time, &mut __sysfail_logged_errors) {
                ::bevy_mod_sysfail::__macro::#level("{error}");
            }
        }
    };
    match level {
        MacroLogLevel::Trace => simple_log(quote!(trace!)),
        MacroLogLevel::Debug => simple_log(quote!(debug!)),
        MacroLogLevel::Info => simple_log(quote!(info!)),
        MacroLogLevel::Warn => simple_log(quote!(warn!)),
        MacroLogLevel::Error => simple_log(quote!(error!)),
        MacroLogLevel::Silent | MacroLogLevel::Quick => quote!(let _ = result;),
    }
}
