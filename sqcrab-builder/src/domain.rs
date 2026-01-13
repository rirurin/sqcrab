use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::time::Instant;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Parser};
use syn::Token;
use walkdir::{DirEntry, WalkDir};

const RUST_FILE_EXTENSION: &'static str = "rs";
const SQCRAB_HINT_ATTR: &'static str = "sqcrab_hint";
const SQCRAB_ATTR: &'static str = "sqcrab";

fn is_source_file(d: DirEntry) -> Option<DirEntry> {
    // this iterator checks from src, which should all be rust files
    match d.file_type().is_file() {
        true => Some(d),
        false => None
    }
}

#[allow(dead_code)]
fn is_rust_source_file(d: DirEntry) -> Option<DirEntry> {
    let ext = d.path().extension();
    if ext.is_none() { return None }
    let ext = ext.unwrap().to_str().unwrap();
    match d.file_type().is_file() && ext == RUST_FILE_EXTENSION {
        true => Some(d),
        false => None
    }
}

fn get_attribute_by_name<'a>(attrs: &'a [syn::Attribute], name: &'a str) -> Option<&'a syn::Attribute> {
    attrs.iter().find(|a| a.path().get_ident()
        .map_or(false, |b| &b.to_string() == name))
}

fn filter_impl_by_sqcrab_methods<'a>(i: &'a syn::ImplItem, t: &'a syn::Type) -> syn::Result<Option<ItemFunction<'a>>> {
    match i {
        syn::ImplItem::Fn(w) => {
            Ok(get_attribute_by_name(&w.attrs, SQCRAB_ATTR).map(
                |a| ItemFunction::try_from_method(w, a, t))
                .and_then(|b| b.ok()))
        },
        _ => Ok(None)
    }
}

pub struct SqcrabParams {
    name: Option<String>,
    domain: Option<String>,
    type_checking: bool
}

impl SqcrabParams {
    fn get_name_value(rhs: &syn::Expr) -> syn::Result<String> {
        if let syn::Expr::Lit(l) = rhs {
            if let syn::Lit::Str(s) = &l.lit {
                return Ok(s.value())
            }
        }
        Err(syn::Error::new(Span::call_site(), "Value type for name should be a string"))
    }
}

impl Parse for SqcrabParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let inputs = syn::punctuated::Punctuated::<syn::ExprAssign, Token![,]>::parse_terminated(input)?;
        let mut name: Option<String> = None;
        let mut domain: Option<String> = None;
        for input in inputs {
            if let syn::Expr::Path(p) = input.left.as_ref() {
                if let Some(p) = p.path.get_ident() {
                    match p.to_string().as_ref() {
                        "name" => name = Some(Self::get_name_value(input.right.as_ref())?),
                        "domain" => domain = Some(Self::get_name_value(input.right.as_ref())?),
                        _ => return Err(syn::Error::new(Span::call_site(), &format!("Unknown parameter {}", p.to_string())))
                    }
                } else {
                    return Err(syn::Error::new(Span::call_site(), "LHS is missing an identifier"));
                }
            } else {
                return Err(syn::Error::new(Span::call_site(), "LHS of argument should be a path"));
            }
        }
        /*
        if name.is_none() {
            return Err(syn::Error::new(Span::call_site(), "Name field is required"));
        }
        */
        Ok(Self { name, domain, type_checking: false })
    }
}

impl<'a> TryFrom<&'a syn::Attribute> for SqcrabParams {
    type Error = syn::Error;

    fn try_from(value: &'a syn::Attribute) -> Result<Self, Self::Error> {
        match &value.meta {
            syn::Meta::List(l) => SqcrabParams::parse.parse2(l.tokens.clone()),
            _ => Err(syn::Error::new(Span::call_site(), "Attribute format should be List")),
        }
    }
}

pub struct ItemFunction<'a> {
    // attribute: &'a syn::Attribute,
    attribute: SqcrabParams,
    this: Option<&'a syn::Type>,
    visibility: &'a syn::Visibility,
    sig: &'a syn::Signature,
    block: &'a syn::Block
}

impl<'a> ItemFunction<'a> {
    pub fn try_from_function(f: &'a syn::ItemFn, a: &'a syn::Attribute) -> syn::Result<Self> {
        Ok(Self {
            attribute: a.try_into()?,
            this: None,
            visibility: &f.vis,
            sig: &f.sig,
            block: f.block.as_ref()
        })
    }

    pub fn try_from_method(f: &'a syn::ImplItemFn, a: &'a syn::Attribute, t: &'a syn::Type) -> syn::Result<Self> {
        let attribute: SqcrabParams = match a.try_into() {
            Ok(p) => p,
            Err(e) => {
                println!("cargo:warning=Could not convert sqcrab attribute for '{}': '{}'", f.sig.ident.to_string(), e.to_string());
                return Err(e);
            }
        };
        Ok(Self {
            // attribute,
            attribute: a.try_into()?,
            this: Some(t),
            visibility: &f.vis,
            sig: &f.sig,
            block: &f.block
        })
    }

    fn get_squirrel_name(&self) -> String {
        match &self.attribute.name {
            Some(n) => n.clone(),
            None => self.sig.ident.to_string()
        }
    }

    fn type_name_from_impl_block(ty: &syn::Type) -> Option<&syn::Ident> {
        match ty {
            // syn::Type::Path(p) => p.path.get_ident().and_then(|i| Some(i.to_string())),
            syn::Type::Path(p) => p.path.get_ident(),
            _ => None
        }
    }

    fn build_type_ref(is_ref: bool, is_mut: bool) -> TokenStream {
        match is_ref {
            true => match is_mut {
                true => quote! { &mut },
                false => quote! { & }
            },
            false => match is_mut {
                true => quote! { mut },
                false => quote! { }
            }
        }
    }

    fn build_method_call(&self) -> TokenStream {
        match &self.this {
            Some(ty) => if let Some(this_name) = Self::type_name_from_impl_block(ty) {
                let method = &self.sig.ident;
                quote! { #this_name :: #method }
            } else {
                quote! { }
            },
            None => {
                let ident = &self.sig.ident;
                quote! { #ident }
            }
        }
    }

    pub fn build(&self, path: &str) {
        let args: Vec<&syn::FnArg> = self.sig.inputs.iter().collect();
        let mut stack_idx = args.len();
        let mut param_decls = vec![];
        for arg in &args {
            let param_name = syn::Ident::new(&format!("p{}", stack_idx), Span::call_site());
            match arg {
                syn::FnArg::Receiver(p) => {
                    // don't use receiver's ty since that will represent &Self instead of
                    // &TypeName, which doesn't make sense outside of an impl context
                    let is_ref = p.reference.is_some();
                    let is_mut = p.mutability.is_some();
                    if let Some(this_ty) = self.this {
                        if let Some(this_name) = Self::type_name_from_impl_block(this_ty) {
                            let ref_ty = Self::build_type_ref(is_ref, is_mut);
                            // let this_name = syn::Ident::new(&this_name, Span::call_site());
                            // TODO: Type checking
                            param_decls.push(quote! {
                                let #param_name = vm.get::<#ref_ty #this_name>(#stack_idx).unwrap();
                            });
                        }
                    }
                },
                syn::FnArg::Typed(p) => {
                    fn get_path_tokens(ty: &syn::Type) -> TokenStream {
                        match ty {
                            syn::Type::Path(p) => {
                                match p.path.get_ident() {
                                    Some(ident) => quote! { #ident },
                                    None => quote! { }
                                }
                            },
                            syn::Type::Reference(p) => {
                                let is_mut = p.mutability.is_some();
                                let elem = get_path_tokens(p.elem.as_ref());
                                let m = match is_mut {
                                    true => quote! { &mut },
                                    false => quote! { & }
                                };
                                quote! { #m #elem }
                            }
                            _ => quote! {}
                        }
                    }

                    let name = get_path_tokens(p.ty.as_ref());
                    param_decls.push(quote! {
                        let #param_name = vm.get::<#name>(#stack_idx).unwrap();
                    })
                }
            }
            stack_idx -= 1;
        }
        let method_call = self.build_method_call();
        let param_list: Vec<syn::Ident> = param_decls.iter().enumerate().map(|(i, _)|
            syn::Ident::new(
                &format!("p{}", param_decls.len() - i),
                Span::call_site())).collect();
        let sq_name = self.get_squirrel_name();
        let tokens = quote! {
            sqvm.add_function(#sq_name, |vm| {
                #(#param_decls)*
                #method_call(#(#param_list),*);
            })?;
        };
        println!("{}", tokens.to_string());
    }
}

pub struct DomainBuilder {
    domains: HashMap<String, Vec<u8>>
}

impl DomainBuilder {
    pub fn new() -> Self {
        Self {
            domains: HashMap::new()
        }
    }

    pub fn build<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), Box<dyn Error>> {
        let path = dir.as_ref().join("src");
        let path_str = path.to_str().unwrap();
        let start = Instant::now();
        let entries: Vec<_> = WalkDir::new(path.as_path()).into_iter()
            .filter_map(|v| v.ok().and_then(is_source_file)).collect();
        for entry in &entries {
            // get path relative to the source folder (to build our crate path)
            let entry_path_str = entry.path().to_str().unwrap();
            let entry_range = match entry_path_str.rfind(".") {
                Some(i) => path_str.len() + 1..i,
                None => path_str.len() + 1..entry_path_str.len()
            };
            let entry_rel_path = &entry.path().to_str().unwrap()[entry_range];
            let module_path = format!("crate::{}", entry_rel_path.replace(std::path::MAIN_SEPARATOR_STR, "::"));
            let tree = syn::parse_file(&std::fs::read_to_string(entry.path())?)?;
            for item in &tree.items {
                match item {
                    syn::Item::Impl(v) => {
                        if get_attribute_by_name(&v.attrs, SQCRAB_HINT_ATTR).is_some() {
                            for func in v.items.iter()
                                .filter_map(|i| filter_impl_by_sqcrab_methods(i, v.self_ty.as_ref())
                                    .unwrap_or_else(|_| None)) {
                                func.build(&module_path);
                            }
                        }
                    },
                    syn::Item::Fn(v) => if let Some(a) = get_attribute_by_name(
                        &v.attrs, SQCRAB_ATTR) {
                        ItemFunction::try_from_function(v, a)?.build(&module_path);
                    },
                    _ => continue
                }
            }
        }
        let ms = Instant::now().duration_since(start).as_millis() as f64;
        if ms > 100. {
            println!("cargo:warning=Parsing {} files took {} ms. To keep compile times low, it's recommended to only check certain files using .sqdefs", entries.len(), ms);
        }
        Ok(())
    }
}

pub fn build_domain_initalization<P: AsRef<Path>>(dir: P) -> Result<(), Box<dyn Error>> {
    DomainBuilder::new().build(dir)
}