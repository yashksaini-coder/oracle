//! Rust source code parser using syn

use crate::analyzer::types::*;
use crate::error::Result;
use quote::ToTokens;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{
    File, Item, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemStatic, ItemStruct, ItemTrait, ItemType,
};

/// Rust source code analyzer using syn for parsing
pub struct RustAnalyzer {
    include_private: bool,
}

impl RustAnalyzer {
    pub fn new() -> Self {
        Self {
            include_private: true,
        }
    }

    pub fn with_private(mut self, include: bool) -> Self {
        self.include_private = include;
        self
    }

    /// Analyze a Rust source file
    pub fn analyze_file(&self, path: &Path) -> Result<Vec<AnalyzedItem>> {
        let content = fs::read_to_string(path)?;
        self.analyze_source_with_path(&content, Some(path.to_path_buf()))
    }

    /// Analyze a Rust source file with a base module path prefix
    pub fn analyze_file_with_module(
        &self,
        path: &Path,
        module_path: Vec<String>,
    ) -> Result<Vec<AnalyzedItem>> {
        let content = fs::read_to_string(path)?;
        self.analyze_source_with_module(&content, Some(path.to_path_buf()), module_path)
    }

    /// Analyze Rust source code from a string
    pub fn analyze_source(&self, source: &str) -> Result<Vec<AnalyzedItem>> {
        self.analyze_source_with_path(source, None)
    }

    /// Analyze Rust source code with optional file path
    pub fn analyze_source_with_path(
        &self,
        source: &str,
        path: Option<PathBuf>,
    ) -> Result<Vec<AnalyzedItem>> {
        // Derive module path from file path
        let module_path = path
            .as_ref()
            .map(|p| Self::derive_module_path(p))
            .unwrap_or_default();
        self.analyze_source_with_module(source, path, module_path)
    }

    /// Analyze Rust source code with explicit module path
    pub fn analyze_source_with_module(
        &self,
        source: &str,
        path: Option<PathBuf>,
        module_path: Vec<String>,
    ) -> Result<Vec<AnalyzedItem>> {
        let syntax_tree: File = syn::parse_str(source)?;
        let mut items = Vec::new();

        for item in syntax_tree.items {
            // Inline modules: expand inner items as first-class AnalyzedItems with synthetic path
            if let Item::Mod(md) = &item {
                if let Some((_, ref content)) = &md.content {
                    let child_path: Vec<String> = {
                        let mut p = module_path.clone();
                        p.push(md.ident.to_string());
                        p
                    };
                    let inner = self.collect_inline_module_items(content, &path, child_path);
                    items.extend(inner);
                }
            }

            if let Some(mut analyzed) = self.analyze_item(&item, &path) {
                Self::set_module_path(&mut analyzed, module_path.clone());

                if let Some(ref file_path) = path {
                    if let Some(span) = Self::get_item_span(&item) {
                        let line = span.start().line;
                        Self::set_source_location(&mut analyzed, file_path.clone(), line);
                    }
                }

                if self.include_private || self.is_public(&analyzed) {
                    items.push(analyzed);
                }
            }
        }

        Ok(items)
    }

    /// Recursively collect items from inline module content as first-class AnalyzedItems.
    fn collect_inline_module_items(
        &self,
        content: &[Item],
        path: &Option<PathBuf>,
        module_path: Vec<String>,
    ) -> Vec<AnalyzedItem> {
        let mut items = Vec::new();
        for item in content {
            if let Item::Mod(md) = item {
                if let Some((_, ref inner_content)) = &md.content {
                    let child_path: Vec<String> = {
                        let mut p = module_path.clone();
                        p.push(md.ident.to_string());
                        p
                    };
                    let inner = self.collect_inline_module_items(inner_content, path, child_path);
                    items.extend(inner);
                }
            }
            if let Some(mut analyzed) = self.analyze_item(item, path) {
                Self::set_module_path(&mut analyzed, module_path.clone());
                if let Some(ref file_path) = path {
                    if let Some(span) = Self::get_item_span(item) {
                        let line = span.start().line;
                        Self::set_source_location(&mut analyzed, file_path.clone(), line);
                    }
                }
                if self.include_private || self.is_public(&analyzed) {
                    items.push(analyzed);
                }
            }
        }
        items
    }

    /// Derive module path from file path (e.g., src/analyzer/parser.rs -> ["analyzer", "parser"])
    fn derive_module_path(path: &Path) -> Vec<String> {
        let mut components: Vec<String> = path
            .iter()
            .filter_map(|c| c.to_str())
            .map(|s| s.to_string())
            .collect();

        // Remove file extension from last component
        if let Some(last) = components.last_mut() {
            if last.ends_with(".rs") {
                *last = last.trim_end_matches(".rs").to_string();
            }
        }

        // Find 'src' directory and take everything after it
        if let Some(src_pos) = components.iter().position(|c| c == "src") {
            components = components[src_pos + 1..].to_vec();
        }

        // Remove special module names
        components.retain(|c| c != "lib" && c != "main" && c != "mod");

        components
    }

    fn set_module_path(item: &mut AnalyzedItem, path: Vec<String>) {
        match item {
            AnalyzedItem::Function(f) => f.module_path = path,
            AnalyzedItem::Struct(s) => s.module_path = path,
            AnalyzedItem::Enum(e) => e.module_path = path,
            AnalyzedItem::Trait(t) => t.module_path = path,
            AnalyzedItem::Impl(i) => i.module_path = path,
            AnalyzedItem::Module(m) => m.module_path = path,
            AnalyzedItem::TypeAlias(t) => t.module_path = path,
            AnalyzedItem::Const(c) => c.module_path = path,
            AnalyzedItem::Static(s) => s.module_path = path,
        }
    }

    fn get_item_span(item: &Item) -> Option<proc_macro2::Span> {
        match item {
            Item::Fn(f) => Some(f.sig.ident.span()),
            Item::Struct(s) => Some(s.ident.span()),
            Item::Enum(e) => Some(e.ident.span()),
            Item::Trait(t) => Some(t.ident.span()),
            Item::Impl(i) => Some(i.impl_token.span),
            Item::Mod(m) => Some(m.ident.span()),
            Item::Type(t) => Some(t.ident.span()),
            Item::Const(c) => Some(c.ident.span()),
            Item::Static(s) => Some(s.ident.span()),
            _ => None,
        }
    }

    fn set_source_location(item: &mut AnalyzedItem, file: PathBuf, line: usize) {
        let loc = SourceLocation::new(file, line);
        match item {
            AnalyzedItem::Function(f) => f.source_location = loc,
            AnalyzedItem::Struct(s) => s.source_location = loc,
            AnalyzedItem::Enum(e) => e.source_location = loc,
            AnalyzedItem::Trait(t) => t.source_location = loc,
            AnalyzedItem::Impl(i) => i.source_location = loc,
            AnalyzedItem::Module(m) => m.source_location = loc,
            AnalyzedItem::TypeAlias(t) => t.source_location = loc,
            AnalyzedItem::Const(c) => c.source_location = loc,
            AnalyzedItem::Static(s) => s.source_location = loc,
        }
    }

    fn is_public(&self, item: &AnalyzedItem) -> bool {
        matches!(item.visibility(), Some(Visibility::Public))
    }

    fn analyze_item(&self, item: &Item, _path: &Option<PathBuf>) -> Option<AnalyzedItem> {
        match item {
            Item::Fn(func) => Some(self.analyze_function(func)),
            Item::Struct(st) => Some(self.analyze_struct(st)),
            Item::Enum(en) => Some(self.analyze_enum(en)),
            Item::Trait(tr) => Some(self.analyze_trait(tr)),
            Item::Impl(im) => Some(self.analyze_impl(im)),
            Item::Mod(md) => Some(self.analyze_module(md)),
            Item::Type(ty) => Some(self.analyze_type_alias(ty)),
            Item::Const(c) => Some(self.analyze_const(c)),
            Item::Static(s) => Some(self.analyze_static(s)),
            _ => None,
        }
    }

    fn analyze_function(&self, func: &ItemFn) -> AnalyzedItem {
        let name = func.sig.ident.to_string();
        let signature = func.sig.to_token_stream().to_string();
        let visibility = Self::parse_visibility(&func.vis);
        let is_async = func.sig.asyncness.is_some();
        let is_const = func.sig.constness.is_some();
        let is_unsafe = func.sig.unsafety.is_some();

        let generics = Self::extract_generics(&func.sig.generics);
        let parameters = Self::extract_parameters(&func.sig.inputs);
        let return_type = Self::extract_return_type(&func.sig.output);
        let where_clause = Self::extract_where_clause(&func.sig.generics.where_clause);
        let documentation = Self::extract_docs(&func.attrs);
        let attributes = Self::extract_attributes(&func.attrs);

        AnalyzedItem::Function(FunctionInfo {
            name,
            signature,
            visibility,
            is_async,
            is_const,
            is_unsafe,
            generics,
            parameters,
            return_type,
            documentation,
            attributes,
            where_clause,
            source_location: SourceLocation::default(),
            module_path: Vec::new(),
        })
    }

    fn analyze_struct(&self, st: &ItemStruct) -> AnalyzedItem {
        let name = st.ident.to_string();
        let visibility = Self::parse_visibility(&st.vis);
        let generics = Self::extract_generics(&st.generics);
        let where_clause = Self::extract_where_clause(&st.generics.where_clause);

        let (fields, kind) = match &st.fields {
            syn::Fields::Named(named) => {
                let fields = named
                    .named
                    .iter()
                    .map(|f| Field {
                        name: f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default(),
                        ty: f.ty.to_token_stream().to_string(),
                        visibility: Self::parse_visibility(&f.vis),
                        documentation: Self::extract_docs(&f.attrs),
                    })
                    .collect();
                (fields, StructKind::Named)
            }
            syn::Fields::Unnamed(unnamed) => {
                let fields = unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, f)| Field {
                        name: i.to_string(),
                        ty: f.ty.to_token_stream().to_string(),
                        visibility: Self::parse_visibility(&f.vis),
                        documentation: Self::extract_docs(&f.attrs),
                    })
                    .collect();
                (fields, StructKind::Tuple)
            }
            syn::Fields::Unit => (vec![], StructKind::Unit),
        };

        let derives = Self::extract_derives(&st.attrs);
        let documentation = Self::extract_docs(&st.attrs);
        let attributes = Self::extract_attributes(&st.attrs);

        AnalyzedItem::Struct(StructInfo {
            name,
            visibility,
            generics,
            fields,
            kind,
            documentation,
            derives,
            attributes,
            where_clause,
            source_location: SourceLocation::default(),
            module_path: Vec::new(),
        })
    }

    fn analyze_enum(&self, en: &ItemEnum) -> AnalyzedItem {
        let name = en.ident.to_string();
        let visibility = Self::parse_visibility(&en.vis);
        let generics = Self::extract_generics(&en.generics);
        let where_clause = Self::extract_where_clause(&en.generics.where_clause);

        let variants = en
            .variants
            .iter()
            .map(|v| {
                let fields = match &v.fields {
                    syn::Fields::Named(named) => {
                        let fields = named
                            .named
                            .iter()
                            .map(|f| Field {
                                name: f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default(),
                                ty: f.ty.to_token_stream().to_string(),
                                visibility: Self::parse_visibility(&f.vis),
                                documentation: Self::extract_docs(&f.attrs),
                            })
                            .collect();
                        VariantFields::Named(fields)
                    }
                    syn::Fields::Unnamed(unnamed) => {
                        let types = unnamed
                            .unnamed
                            .iter()
                            .map(|f| f.ty.to_token_stream().to_string())
                            .collect();
                        VariantFields::Unnamed(types)
                    }
                    syn::Fields::Unit => VariantFields::Unit,
                };

                let discriminant = v
                    .discriminant
                    .as_ref()
                    .map(|(_, expr)| expr.to_token_stream().to_string());

                Variant {
                    name: v.ident.to_string(),
                    fields,
                    discriminant,
                    documentation: Self::extract_docs(&v.attrs),
                }
            })
            .collect();

        let derives = Self::extract_derives(&en.attrs);
        let documentation = Self::extract_docs(&en.attrs);
        let attributes = Self::extract_attributes(&en.attrs);

        AnalyzedItem::Enum(EnumInfo {
            name,
            visibility,
            generics,
            variants,
            documentation,
            derives,
            attributes,
            where_clause,
            source_location: SourceLocation::default(),
            module_path: Vec::new(),
        })
    }

    fn analyze_trait(&self, tr: &ItemTrait) -> AnalyzedItem {
        let name = tr.ident.to_string();
        let visibility = Self::parse_visibility(&tr.vis);
        let is_unsafe = tr.unsafety.is_some();
        let is_auto = tr.auto_token.is_some();
        let generics = Self::extract_generics(&tr.generics);
        let where_clause = Self::extract_where_clause(&tr.generics.where_clause);

        let supertraits = tr
            .supertraits
            .iter()
            .map(|t| t.to_token_stream().to_string())
            .collect();

        let mut methods = Vec::new();
        let mut associated_types = Vec::new();
        let mut associated_consts = Vec::new();

        for item in &tr.items {
            match item {
                syn::TraitItem::Fn(method) => {
                    methods.push(TraitMethod {
                        name: method.sig.ident.to_string(),
                        signature: method.sig.to_token_stream().to_string(),
                        has_default: method.default.is_some(),
                        is_async: method.sig.asyncness.is_some(),
                        documentation: Self::extract_docs(&method.attrs),
                    });
                }
                syn::TraitItem::Type(ty) => {
                    associated_types.push(AssociatedType {
                        name: ty.ident.to_string(),
                        bounds: ty
                            .bounds
                            .iter()
                            .map(|b| b.to_token_stream().to_string())
                            .collect(),
                        default: ty
                            .default
                            .as_ref()
                            .map(|(_, t)| t.to_token_stream().to_string()),
                    });
                }
                syn::TraitItem::Const(c) => {
                    associated_consts.push(AssociatedConst {
                        name: c.ident.to_string(),
                        ty: c.ty.to_token_stream().to_string(),
                        default: c
                            .default
                            .as_ref()
                            .map(|(_, e)| e.to_token_stream().to_string()),
                    });
                }
                _ => {}
            }
        }

        let documentation = Self::extract_docs(&tr.attrs);

        AnalyzedItem::Trait(TraitInfo {
            name,
            visibility,
            generics,
            supertraits,
            methods,
            associated_types,
            associated_consts,
            documentation,
            is_unsafe,
            is_auto,
            where_clause,
            source_location: SourceLocation::default(),
            module_path: Vec::new(),
        })
    }

    fn analyze_impl(&self, im: &ItemImpl) -> AnalyzedItem {
        let self_ty = im.self_ty.to_token_stream().to_string();
        let trait_name = im
            .trait_
            .as_ref()
            .map(|(_, path, _)| path.to_token_stream().to_string());
        let is_unsafe = im.unsafety.is_some();
        let is_negative = im
            .trait_
            .as_ref()
            .is_some_and(|(bang, _, _)| bang.is_some());
        let generics = Self::extract_generics(&im.generics);
        let where_clause = Self::extract_where_clause(&im.generics.where_clause);

        let methods = im
            .items
            .iter()
            .filter_map(|item| {
                if let syn::ImplItem::Fn(method) = item {
                    Some(self.extract_impl_method(method))
                } else {
                    None
                }
            })
            .collect();

        AnalyzedItem::Impl(ImplInfo {
            self_ty,
            trait_name,
            generics,
            methods,
            is_unsafe,
            is_negative,
            where_clause,
            source_location: SourceLocation::default(),
            module_path: Vec::new(),
        })
    }

    fn analyze_module(&self, md: &syn::ItemMod) -> AnalyzedItem {
        let name = md.ident.to_string();
        let visibility = Self::parse_visibility(&md.vis);
        let documentation = Self::extract_docs(&md.attrs);
        let is_inline = md.content.is_some();

        let (items, submodules) = if let Some((_, content)) = &md.content {
            let mut item_names = Vec::new();
            let mut submod_names = Vec::new();

            for item in content {
                match item {
                    Item::Mod(m) => submod_names.push(m.ident.to_string()),
                    Item::Fn(f) => item_names.push(format!("fn {}", f.sig.ident)),
                    Item::Struct(s) => item_names.push(format!("struct {}", s.ident)),
                    Item::Enum(e) => item_names.push(format!("enum {}", e.ident)),
                    Item::Trait(t) => item_names.push(format!("trait {}", t.ident)),
                    Item::Impl(i) => {
                        let ty = i.self_ty.to_token_stream().to_string();
                        if let Some((_, path, _)) = &i.trait_ {
                            item_names.push(format!("impl {} for {}", path.to_token_stream(), ty));
                        } else {
                            item_names.push(format!("impl {}", ty));
                        }
                    }
                    Item::Type(t) => item_names.push(format!("type {}", t.ident)),
                    Item::Const(c) => item_names.push(format!("const {}", c.ident)),
                    Item::Static(s) => item_names.push(format!("static {}", s.ident)),
                    _ => {}
                }
            }

            (item_names, submod_names)
        } else {
            (Vec::new(), Vec::new())
        };

        AnalyzedItem::Module(ModuleInfo {
            name,
            path: String::new(),
            visibility,
            items,
            submodules,
            documentation,
            is_inline,
            source_location: SourceLocation::default(),
            module_path: Vec::new(),
        })
    }

    fn analyze_type_alias(&self, ty: &ItemType) -> AnalyzedItem {
        AnalyzedItem::TypeAlias(TypeAliasInfo {
            name: ty.ident.to_string(),
            visibility: Self::parse_visibility(&ty.vis),
            generics: Self::extract_generics(&ty.generics),
            ty: ty.ty.to_token_stream().to_string(),
            documentation: Self::extract_docs(&ty.attrs),
            where_clause: Self::extract_where_clause(&ty.generics.where_clause),
            source_location: SourceLocation::default(),
            module_path: Vec::new(),
        })
    }

    fn analyze_const(&self, c: &ItemConst) -> AnalyzedItem {
        AnalyzedItem::Const(ConstInfo {
            name: c.ident.to_string(),
            visibility: Self::parse_visibility(&c.vis),
            ty: c.ty.to_token_stream().to_string(),
            value: Some(c.expr.to_token_stream().to_string()),
            documentation: Self::extract_docs(&c.attrs),
            source_location: SourceLocation::default(),
            module_path: Vec::new(),
        })
    }

    fn analyze_static(&self, s: &ItemStatic) -> AnalyzedItem {
        let is_mut = matches!(s.mutability, syn::StaticMutability::Mut(_));
        AnalyzedItem::Static(StaticInfo {
            name: s.ident.to_string(),
            visibility: Self::parse_visibility(&s.vis),
            ty: s.ty.to_token_stream().to_string(),
            is_mut,
            documentation: Self::extract_docs(&s.attrs),
            source_location: SourceLocation::default(),
            module_path: Vec::new(),
        })
    }

    fn extract_impl_method(&self, method: &syn::ImplItemFn) -> FunctionInfo {
        FunctionInfo {
            name: method.sig.ident.to_string(),
            signature: method.sig.to_token_stream().to_string(),
            visibility: Self::parse_visibility(&method.vis),
            is_async: method.sig.asyncness.is_some(),
            is_const: method.sig.constness.is_some(),
            is_unsafe: method.sig.unsafety.is_some(),
            generics: Self::extract_generics(&method.sig.generics),
            parameters: Self::extract_parameters(&method.sig.inputs),
            return_type: Self::extract_return_type(&method.sig.output),
            documentation: Self::extract_docs(&method.attrs),
            attributes: Self::extract_attributes(&method.attrs),
            where_clause: Self::extract_where_clause(&method.sig.generics.where_clause),
            source_location: SourceLocation::default(),
            module_path: Vec::new(),
        }
    }

    fn parse_visibility(vis: &syn::Visibility) -> Visibility {
        match vis {
            syn::Visibility::Public(_) => Visibility::Public,
            syn::Visibility::Restricted(r) => {
                if r.path.is_ident("crate") {
                    Visibility::Crate
                } else if r.path.is_ident("super") {
                    Visibility::Super
                } else if r.path.is_ident("self") {
                    Visibility::SelfOnly
                } else {
                    Visibility::Private
                }
            }
            syn::Visibility::Inherited => Visibility::Private,
        }
    }

    fn extract_generics(generics: &syn::Generics) -> Vec<String> {
        generics
            .params
            .iter()
            .map(|p| p.to_token_stream().to_string())
            .collect()
    }

    fn extract_parameters(
        inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    ) -> Vec<Parameter> {
        inputs
            .iter()
            .map(|arg| match arg {
                syn::FnArg::Receiver(recv) => Parameter {
                    name: "self".to_string(),
                    ty: "Self".to_string(),
                    is_self: true,
                    is_mut: recv.mutability.is_some(),
                    is_ref: recv.reference.is_some(),
                },
                syn::FnArg::Typed(pat_type) => Parameter {
                    name: pat_type.pat.to_token_stream().to_string(),
                    ty: pat_type.ty.to_token_stream().to_string(),
                    is_self: false,
                    is_mut: false,
                    is_ref: false,
                },
            })
            .collect()
    }

    fn extract_return_type(output: &syn::ReturnType) -> Option<String> {
        match output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(ty.to_token_stream().to_string()),
        }
    }

    fn extract_where_clause(where_clause: &Option<syn::WhereClause>) -> Option<String> {
        where_clause
            .as_ref()
            .map(|w| w.to_token_stream().to_string())
    }

    fn extract_docs(attrs: &[syn::Attribute]) -> Option<String> {
        let docs: Vec<String> = attrs
            .iter()
            .filter_map(|attr| {
                if attr.path().is_ident("doc") {
                    attr.meta.require_name_value().ok().and_then(|nv| {
                        if let syn::Expr::Lit(expr_lit) = &nv.value {
                            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                return Some(lit_str.value().trim().to_string());
                            }
                        }
                        None
                    })
                } else {
                    None
                }
            })
            .collect();

        if docs.is_empty() {
            None
        } else {
            Some(docs.join("\n"))
        }
    }

    fn extract_derives(attrs: &[syn::Attribute]) -> Vec<String> {
        attrs
            .iter()
            .filter_map(|attr| {
                if attr.path().is_ident("derive") {
                    attr.parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    )
                    .ok()
                    .map(|paths| {
                        paths
                            .iter()
                            .map(|p| p.to_token_stream().to_string())
                            .collect::<Vec<_>>()
                    })
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }

    fn extract_attributes(attrs: &[syn::Attribute]) -> Vec<String> {
        attrs
            .iter()
            .filter(|attr| !attr.path().is_ident("doc") && !attr.path().is_ident("derive"))
            .map(|attr| attr.to_token_stream().to_string())
            .collect()
    }
}

impl Default for RustAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_function() {
        let source = r#"
            /// A test function
            pub fn hello(name: &str) -> String {
                format!("Hello, {}!", name)
            }
        "#;

        let analyzer = RustAnalyzer::new();
        let items = analyzer.analyze_source(source).unwrap();

        assert_eq!(items.len(), 1);
        if let AnalyzedItem::Function(f) = &items[0] {
            assert_eq!(f.name, "hello");
            assert_eq!(f.visibility, Visibility::Public);
            assert!(f.documentation.is_some());
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_analyze_struct() {
        let source = r#"
            #[derive(Debug, Clone)]
            pub struct Point {
                pub x: f64,
                pub y: f64,
            }
        "#;

        let analyzer = RustAnalyzer::new();
        let items = analyzer.analyze_source(source).unwrap();

        assert_eq!(items.len(), 1);
        if let AnalyzedItem::Struct(s) = &items[0] {
            assert_eq!(s.name, "Point");
            assert_eq!(s.fields.len(), 2);
            assert!(s.derives.contains(&"Debug".to_string()));
            assert!(s.derives.contains(&"Clone".to_string()));
        } else {
            panic!("Expected struct");
        }
    }

    #[test]
    fn test_analyze_enum() {
        let source = r#"
            pub enum Result<T, E> {
                Ok(T),
                Err(E),
            }
        "#;
        let analyzer = RustAnalyzer::new();
        let items = analyzer.analyze_source(source).unwrap();
        assert_eq!(items.len(), 1);
        if let AnalyzedItem::Enum(e) = &items[0] {
            assert_eq!(e.name, "Result");
            assert_eq!(e.variants.len(), 2);
        } else {
            panic!("Expected enum");
        }
    }

    #[test]
    fn test_analyze_module_path_from_path() {
        use std::path::Path;
        assert_eq!(
            RustAnalyzer::derive_module_path(Path::new("src/lib.rs")),
            Vec::<String>::new()
        );
        assert_eq!(
            RustAnalyzer::derive_module_path(Path::new("src/foo/bar.rs")),
            vec!["foo".to_string(), "bar".to_string()]
        );
    }

    #[test]
    fn test_analyze_source_with_module_prefix() {
        let source = "pub fn util() {}";
        let analyzer = RustAnalyzer::new();
        let items = analyzer
            .analyze_source_with_module(source, None, vec!["mymod".to_string()])
            .unwrap();
        assert_eq!(items.len(), 1);
        if let AnalyzedItem::Function(f) = &items[0] {
            assert_eq!(f.name, "util");
            assert_eq!(f.module_path.as_slice(), &["mymod"]);
        } else {
            panic!("Expected function");
        }
    }
}
