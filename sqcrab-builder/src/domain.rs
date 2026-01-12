use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::time::Instant;
use proc_macro2::Span;
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

fn filter_impl_by_sqcrab_methods(i: &'_ syn::ImplItem) -> syn::Result<Option<ItemFunction<'_>>> {
    match i {
        syn::ImplItem::Fn(w) => {
            Ok(get_attribute_by_name(&w.attrs, SQCRAB_ATTR).map(
                |a| ItemFunction::try_from_method(w, a))
                .and_then(|b| b.ok()))
        },
        _ => Ok(None)
    }
}

pub struct SqcrabParams {
    name: Option<String>,
    domain: Option<String>
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
        Ok(Self { name, domain })
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
    visibility: &'a syn::Visibility,
    sig: &'a syn::Signature,
    block: &'a syn::Block
}

impl<'a> ItemFunction<'a> {
    pub fn try_from_function(f: &'a syn::ItemFn, a: &'a syn::Attribute) -> syn::Result<Self> {
        Ok(Self {
            attribute: a.try_into()?,
            visibility: &f.vis,
            sig: &f.sig,
            block: f.block.as_ref()
        })
    }

    pub fn try_from_method(f: &'a syn::ImplItemFn, a: &'a syn::Attribute) -> syn::Result<Self> {
        Ok(Self {
            attribute: a.try_into()?,
            visibility: &f.vis,
            sig: &f.sig,
            block: &f.block
        })
    }

    pub fn build(&self) {
        println!("found sqcrab");
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
        let start = Instant::now();
        let entries: Vec<_> = WalkDir::new(path).into_iter()
            .filter_map(|v| v.ok().and_then(is_source_file)).collect();
        for entry in &entries {
            let tree = syn::parse_file(&std::fs::read_to_string(entry.path())?)?;
            for item in &tree.items {
                match item {
                    syn::Item::Impl(v) => {
                        if get_attribute_by_name(&v.attrs, SQCRAB_HINT_ATTR).is_some() {
                            for func in v.items.iter()
                                .filter_map(|i| filter_impl_by_sqcrab_methods(i).unwrap_or_else(|_| None)) {
                                func.build();
                            }
                        }
                    },
                    syn::Item::Fn(v) => if let Some(a) = get_attribute_by_name(
                        &v.attrs, SQCRAB_ATTR) {
                        ItemFunction::try_from_function(v, a)?.build();
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