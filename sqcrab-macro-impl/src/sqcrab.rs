use proc_macro2::TokenStream;
// use quote::quote;
// use syn::parse::Parser;

/*
// Not neccessary, see sqcrab_builder::domain
#[derive(Debug)]
struct SqCrabParams {

}

impl SqCrabParams {
    const OUTER_NAME: &'static str = "sqcrab";
}

fn sqcrab_inner(item: TokenStream) -> syn::Result<TokenStream> {
    let attributes = syn::Attribute::parse_outer.parse2(item)?;
    let attribute = attributes.iter().find(
        |attr| attr.path().get_ident().map_or(
            false, |i| i.to_string() == SqCrabParams::OUTER_NAME));
    if attribute.is_none() {
        return Ok(quote! {});
    }
    let attribute = attribute.unwrap();
    if let syn::Meta::List(ml) = &attribute.meta {

    }
    Ok(quote! {})
}
*/

pub fn sqcrab(_: TokenStream, annotated_item: TokenStream) -> TokenStream {
    annotated_item
}