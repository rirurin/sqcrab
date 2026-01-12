use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, Parser };

fn hint_inner(annotated_item: TokenStream) -> syn::Result<TokenStream> {
    let item_impl = syn::ItemImpl::parse.parse2(annotated_item)?;
    Ok(item_impl.to_token_stream())
}

pub fn hint(_: TokenStream, annotated_item: TokenStream) -> TokenStream {
    hint_inner(annotated_item).unwrap_or_else(|e| e.to_compile_error())
}