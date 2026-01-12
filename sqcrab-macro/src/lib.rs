use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn sqcrab(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    sqcrab_macro_impl::sqcrab::sqcrab(input.into(), annotated_item.into()).into()
}

#[proc_macro_derive(SqObject)]
pub fn derive_sq_object(item: TokenStream) -> TokenStream {
    sqcrab_macro_impl::sqobject::sqobject(item.into()).into()
}

#[proc_macro_attribute]
pub fn sqcrab_hint(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    sqcrab_macro_impl::hint::hint(input.into(), annotated_item.into()).into()
}