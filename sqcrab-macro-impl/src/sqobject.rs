use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{ Parse, Parser };

pub fn sqobject(item: TokenStream) -> TokenStream {
    sqobject_inner(item).unwrap_or_else(|e| e.to_compile_error())
}

fn generate_squirrel_type_definition(name: &str, mutable: bool) -> TokenStream {
    let ref_type = match mutable {
        true => quote! { &'a mut },
        false => quote! { &'a }
    };
    let name = syn::Ident::new(name, Span::call_site());
    quote! {
        impl<'a> ::sqcrab::squirrel::type_cnv::CanSquirrel for #ref_type #name {
            type Into = ::sqcrab::squirrel::obj_type::UserPointer<Self>;
            const RETURNS: bool = true;

            fn into_squirrel(&self) -> Self::Into {
                ::sqcrab::squirrel::obj_type::UserPointer::<Self>::new(self)
            }

            fn from_squirrel(v: Self::Into) -> Self {
                unsafe { *v.as_ptr() }
            }
        }
    }
}

fn sqobject_inner(item: TokenStream) -> syn::Result<TokenStream> {
    let target_struct = syn::ItemStruct::parse.parse2(item)?;
    let struct_name = target_struct.ident.to_string();
    let def = generate_squirrel_type_definition(&struct_name, false);
    let def_mut = generate_squirrel_type_definition(&struct_name, true);
    Ok(quote! {
        #def
        #def_mut
    })
}