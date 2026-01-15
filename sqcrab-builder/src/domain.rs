use std::collections::{HashMap, HashSet};
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

fn is_source_file<P: AsRef<Path>>(base: P, d: DirEntry, conf: &DomainBuilderConfig) -> Option<DirEntry> {
    // this iterator checks from src, which should all be rust files
    let base_str = base.as_ref().to_str().unwrap();
    match d.file_type().is_file() {
        true => {
            let path_str = d.path().to_str().unwrap();
            let name = &path_str[base_str.len() + 1..path_str.rfind(".").unwrap_or(path_str.len())];
            if &conf.get_output_file() == name {
                return None;
            }
            match conf.get_includes() {
                Some(p) => {
                    match p.contains(&name) {
                        true => Some(d),
                        false => None
                    }
                },
                None => Some(d)
            }
        },
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
    type_checking: bool,
    local_pointer: bool
}

impl SqcrabParams {

    const DEFAULT_DOMAIN: &'static str = "Default";

    fn get_name_value(rhs: &syn::Expr) -> syn::Result<String> {
        if let syn::Expr::Lit(l) = rhs {
            if let syn::Lit::Str(s) = &l.lit {
                return Ok(s.value())
            }
        }
        Err(syn::Error::new(Span::call_site(), "Value type for name should be a string"))
    }

    fn get_bool_value(rhs: &syn::Expr) -> syn::Result<bool> {
        if let syn::Expr::Lit(l) = rhs {
            if let syn::Lit::Bool(b) = &l.lit {
                return Ok(b.value())
            }
        }
        Err(syn::Error::new(Span::call_site(), "Value type for name should be a bool"))
    }
}

impl Parse for SqcrabParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let inputs = syn::punctuated::Punctuated::<syn::ExprAssign, Token![,]>::parse_terminated(input)?;
        let mut name: Option<String> = None;
        let mut domain: Option<String> = None;
        let mut type_checking = false;
        let mut local_pointer = false;
        for input in inputs {
            if let syn::Expr::Path(p) = input.left.as_ref() {
                if let Some(p) = p.path.get_ident() {
                    match p.to_string().as_ref() {
                        "name" => name = Some(Self::get_name_value(input.right.as_ref())?),
                        "domain" => domain = Some(Self::get_name_value(input.right.as_ref())?),
                        "type_checking" => type_checking = Self::get_bool_value(input.right.as_ref())?,
                        "local_pointer" => type_checking = Self::get_bool_value(input.right.as_ref())?,
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
        Ok(Self { name, domain, type_checking, local_pointer })
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

    fn get_domain(&self) -> String {
        match &self.attribute.domain {
            Some(n) => n.clone(),
            None => SqcrabParams::DEFAULT_DOMAIN.to_string()
        }
    }

    fn get_type_checking(&self) -> bool {
        self.attribute.type_checking
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

    fn build_struct_path(this_name: &syn::Ident, sup: &DomainBuilderSupportItem) -> syn::Result<TokenStream> {
        Ok(if sup.declared_structs.contains(&this_name.to_string()) {
            sup.module_path.parse()?
        } else {
            quote! { }
        })
    }

    fn build_method_call(&self, sup: &DomainBuilderSupportItem) -> syn::Result<TokenStream> {
        match &self.this {
            Some(ty) => if let Some(this_name) = Self::type_name_from_impl_block(ty) {
                let path = Self::build_struct_path(this_name, sup)?;
                let method = &self.sig.ident;
                Ok(quote! { #path #this_name :: #method })
            } else {
                Err(syn::Error::new(Span::call_site(), "Impl declaration should have a name"))
            },
            None => {
                let ident = &self.sig.ident;
                Ok(quote! { #ident })
            }
        }
    }

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
                let elem = Self::get_path_tokens(p.elem.as_ref());
                let m = match is_mut {
                    true => quote! { &mut },
                    false => quote! { & }
                };
                quote! { #m #elem }
            }
            _ => quote! {}
        }
    }

    pub fn build(&self, sup: &DomainBuilderSupportItem) -> syn::Result<TokenStream> {
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
                            // let path_tokens: TokenStream = path.parse()?;
                            let path = Self::build_struct_path(this_name, sup)?;
                            // TODO: Type checking
                            if self.attribute.local_pointer {
                                param_decls.push(quote! {
                                    let #param_name = vm.get::<#ref_ty #path #this_name>(#stack_idx).unwrap();
                                });
                            } else {
                                param_decls.push(quote! {
                                    // let #param_name = vm.get_this::<#ref_ty #path #this_name>().unwrap();
                                    let #param_name = unsafe { vm.get_this::<#path #this_name>().unwrap() };
                                });
                            }
                        }
                    }
                },
                syn::FnArg::Typed(p) => {

                    let name = Self::get_path_tokens(p.ty.as_ref());
                    param_decls.push(quote! {
                        let #param_name = vm.get::<#name>(#stack_idx).unwrap();
                    })
                }
            }
            stack_idx -= 1;
        }
        let method_call = self.build_method_call(sup)?;
        let param_list: Vec<syn::Ident> = param_decls.iter().enumerate().map(|(i, _)|
            syn::Ident::new(
                &format!("p{}", param_decls.len() - i),
                Span::call_site())).collect();
        let sq_name = self.get_squirrel_name();
        let params_returned: i64 = match &self.sig.output {
            syn::ReturnType::Default => 0,
            syn::ReturnType::Type(_, t) => {
                match t.as_ref() {
                    // note: tuple with no elements is void
                    syn::Type::Tuple(t) => (t.elems.iter().count() != 0).into(),
                    _ => 1
                }
            }
        };
        let ret_type = match &self.sig.output {
            syn::ReturnType::Type(_, t) => {
                Some(Self::get_path_tokens(t.as_ref()))
            },
            _ => None
        };
        let method_call = match params_returned {
            0 => quote! { #method_call(#(#param_list),*); },
            _ => {
                let ret_type = ret_type.unwrap();
                quote! {
                    let ret = #method_call(#(#param_list),*);
                    vm.push::<#ret_type>(&ret);
                }
            }
        };
        Ok(quote! {
            vm.add_function(#sq_name, |vm| {
                #(#param_decls)*
                #method_call
                #params_returned
            })?;
        })
    }
}

pub struct DomainBuilderSupportItem<'a> {
    // for structs that are declared in the current module
    module_path: String,
    imports: Vec<&'a syn::ItemUse>,
    declared_structs: HashSet<String>
}

impl<'a> DomainBuilderSupportItem<'a> {
    pub fn new(module_path: String) -> Self {
        Self { module_path, imports: vec![], declared_structs: HashSet::new() }
    }
}

pub struct DomainBuilderConfig {
    table: Option<toml::Table>
}

impl DomainBuilderConfig {

    const DEFAULT_OUTPUT: &'static str = "sqcrab_domains.rs";

    pub fn new<P: AsRef<Path>>(dir: P) -> Result<Self, Box<dyn Error>> {
        let sqcrab_toml = dir.as_ref().join("sqcrab.toml");
        let table = match std::fs::exists(sqcrab_toml.as_path())? {
            true => Some(std::fs::read_to_string(sqcrab_toml.as_path())?.parse::<toml::Table>()?),
            false => None
        };
        Ok(Self { table })
    }

    #[inline]
    fn get_output_file_inner(&self) -> Option<String> {
        self.table.as_ref()
            .and_then(|t| t.get("output"))
            .and_then(|v| v.as_str())
            .map(|v| format!("{}.rs", v))
    }

    pub fn get_output_file(&self) -> String {
        self.get_output_file_inner().unwrap_or(Self::DEFAULT_OUTPUT.to_string())
    }

    pub fn get_includes(&self) -> Option<Vec<&str>> {
        self.table.as_ref()
            .and_then(|t| t.get("include"))
            .and_then(|a| a.as_array().map(|v|
                v.iter().filter_map(|w| w.as_str()).collect()))
    }
}

#[derive(Debug)]
pub struct DomainBuilder {
    domains: HashMap<String, Vec<TokenStream>>
}

impl DomainBuilder {
    pub fn new() -> Self {
        Self {
            domains: HashMap::new()
        }
    }

    /*
    fn get_imports_from_use(&self, tree: &syn::UseTree, s: &mut String) {
        match tree {
            syn::UseTree::Path(p) => {
                s.push_str(&p.ident.to_string());
                s.push_str("::");
                self.get_imports_from_use(p.tree.as_ref(), s);
            },
            syn::UseTree::Name(p) => {
                s.push_str(&p.ident.to_string());
                println!("Add string {}", s);
            },
            syn::UseTree::Rename(p) => {
                s.push_str(&p.ident.to_string());
                s.push_str(" as ");
                s.push_str(&p.rename.to_string());
                println!("Add string {}", s);
            },
            _ => ()
        }
    }
    */

    fn add_to_domain(&mut self, s: String, v: TokenStream) {
        match self.domains.get_mut(&s) {
            Some(p) => p.push(v),
            None => { let _ = self.domains.insert(s, vec![v]); },
        }
    }

    pub fn build<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), Box<dyn Error>> {
        let config = DomainBuilderConfig::new(dir.as_ref())?;
        let path = dir.as_ref().join("src");
        let path_str = path.to_str().unwrap();
        let start = Instant::now();
        let entries: Vec<_> = WalkDir::new(path.as_path()).into_iter()
            .filter_map(|v| v.ok().and_then(|f| is_source_file(path.as_path(), f, &config))).collect();
        for entry in &entries {
            // get path relative to the source folder (to build our crate path)
            let entry_path_str = entry.path().to_str().unwrap();
            let entry_range = match entry_path_str.rfind(".") {
                Some(i) => path_str.len() + 1..i,
                None => path_str.len() + 1..entry_path_str.len()
            };
            let entry_rel_path = &entry.path().to_str().unwrap()[entry_range];
            let mut support_items = DomainBuilderSupportItem::new(
                format!("crate::{}::", entry_rel_path.replace(std::path::MAIN_SEPARATOR_STR, "::")));
            let tree = syn::parse_file(&std::fs::read_to_string(entry.path())?)?;
            // let mut imports = vec![];
            // let mut declared_structs = HashSet::new();
            for item in &tree.items {
                match item {
                    syn::Item::Impl(v) => {
                        if get_attribute_by_name(&v.attrs, SQCRAB_HINT_ATTR).is_some() {
                            for func in v.items.iter()
                                .filter_map(|i| filter_impl_by_sqcrab_methods(i, v.self_ty.as_ref())
                                    .unwrap_or_else(|_| None)) {
                                self.add_to_domain(func.get_domain(), func.build(&support_items)?);
                            }
                        }
                    },
                    syn::Item::Struct(v) => {
                        support_items.declared_structs.insert(v.ident.to_string());
                    },
                    syn::Item::Fn(v) => if let Some(a) = get_attribute_by_name(
                        &v.attrs, SQCRAB_ATTR) {
                        let func = ItemFunction::try_from_function(v, a)?;
                        self.add_to_domain(func.get_domain(), func.build(&support_items)?);
                    },
                    syn::Item::Use(v) => {
                        support_items.imports.push(v);
                    },
                    // syn::Item::Use(v) => self.get_imports_from_use(&v.tree, &mut String::new()),
                    _ => continue
                }
            }
            // for import in &support_items.imports {
            //     println!("{}", import.to_token_stream().to_string());
            // }
        }

        let mut domain_tokens = vec![];
        for (name, decl) in &self.domains {
            let name = syn::Ident::new(name, Span::call_site());
            domain_tokens.push(quote! {
                pub struct #name;

                impl ::sqcrab::domain::DomainRegistrar for #name {
                    fn add_functions(vm: &mut ::sqcrab::squirrel::vm::SquirrelVM) -> Result<(), ::sqcrab::squirrel::err::SquirrelError> {
                        #(#decl)*
                        Ok(())
                    }
                }
            });
        }
        // let file = syn::File::parse.parse2(quote! { #(#domain_tokens)* })?;
        std::fs::write(path.join(config.get_output_file()), quote! { #(#domain_tokens)* }.to_string())?;
        let ms = Instant::now().duration_since(start).as_millis() as f64;
        if ms > 100. {
            println!("cargo:warning=Parsing {} files took {} ms. To keep compile times low, it's recommended to only check certain files using sqcrab.toml", entries.len(), ms);
        }
        Ok(())
    }
}

pub fn build_domain_initalization<P: AsRef<Path>>(dir: P) -> Result<(), Box<dyn Error>> {
    DomainBuilder::new().build(dir)
}