use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn callback(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let fn_name = &input.sig.ident;
    let vis = &input.vis;
    let body = &input.block;
    let args = input.sig.inputs;

    let expanded = quote! {
        #vis fn #fn_name(#args) -> BoxFuture<'static, Response> {
            Box::pin(async move #body)
        }
    };

    TokenStream::from(expanded)
}
