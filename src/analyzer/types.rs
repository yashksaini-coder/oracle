//! Type definitions for analyzed Rust code items

use std::fmt;
use std::path::PathBuf;

/// Source location information
#[derive(Debug, Clone, Default)]
pub struct SourceLocation {
    pub file: Option<PathBuf>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl SourceLocation {
    pub fn new(file: PathBuf, line: usize) -> Self {
        Self {
            file: Some(file),
            line: Some(line),
            column: None,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.file, self.line) {
            (Some(file), Some(line)) => write!(f, "{}:{}", file.display(), line),
            (Some(file), None) => write!(f, "{}", file.display()),
            _ => write!(f, "unknown"),
        }
    }
}

/// Visibility of a Rust item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    Public,
    Crate,
    Super,
    SelfOnly,
    #[default]
    Private,
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Visibility::Public => write!(f, "pub"),
            Visibility::Crate => write!(f, "pub(crate)"),
            Visibility::Super => write!(f, "pub(super)"),
            Visibility::SelfOnly => write!(f, "pub(self)"),
            Visibility::Private => write!(f, ""),
        }
    }
}

/// Analyzed item from Rust source code
#[derive(Debug, Clone)]
pub enum AnalyzedItem {
    Function(FunctionInfo),
    Struct(StructInfo),
    Enum(EnumInfo),
    Trait(TraitInfo),
    Impl(ImplInfo),
    Module(ModuleInfo),
    TypeAlias(TypeAliasInfo),
    Const(ConstInfo),
    Static(StaticInfo),
}

impl AnalyzedItem {
    pub fn name(&self) -> &str {
        match self {
            AnalyzedItem::Function(f) => &f.name,
            AnalyzedItem::Struct(s) => &s.name,
            AnalyzedItem::Enum(e) => &e.name,
            AnalyzedItem::Trait(t) => &t.name,
            AnalyzedItem::Impl(i) => &i.self_ty,
            AnalyzedItem::Module(m) => &m.name,
            AnalyzedItem::TypeAlias(t) => &t.name,
            AnalyzedItem::Const(c) => &c.name,
            AnalyzedItem::Static(s) => &s.name,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            AnalyzedItem::Function(_) => "fn",
            AnalyzedItem::Struct(_) => "struct",
            AnalyzedItem::Enum(_) => "enum",
            AnalyzedItem::Trait(_) => "trait",
            AnalyzedItem::Impl(_) => "impl",
            AnalyzedItem::Module(_) => "mod",
            AnalyzedItem::TypeAlias(_) => "type",
            AnalyzedItem::Const(_) => "const",
            AnalyzedItem::Static(_) => "static",
        }
    }

    pub fn visibility(&self) -> Option<Visibility> {
        match self {
            AnalyzedItem::Function(f) => Some(f.visibility),
            AnalyzedItem::Struct(s) => Some(s.visibility),
            AnalyzedItem::Enum(e) => Some(e.visibility),
            AnalyzedItem::Trait(t) => Some(t.visibility),
            AnalyzedItem::Impl(_) => None,
            AnalyzedItem::Module(m) => Some(m.visibility),
            AnalyzedItem::TypeAlias(t) => Some(t.visibility),
            AnalyzedItem::Const(c) => Some(c.visibility),
            AnalyzedItem::Static(s) => Some(s.visibility),
        }
    }

    pub fn documentation(&self) -> Option<&str> {
        match self {
            AnalyzedItem::Function(f) => f.documentation.as_deref(),
            AnalyzedItem::Struct(s) => s.documentation.as_deref(),
            AnalyzedItem::Enum(e) => e.documentation.as_deref(),
            AnalyzedItem::Trait(t) => t.documentation.as_deref(),
            AnalyzedItem::Impl(_) => None,
            AnalyzedItem::Module(m) => m.documentation.as_deref(),
            AnalyzedItem::TypeAlias(t) => t.documentation.as_deref(),
            AnalyzedItem::Const(c) => c.documentation.as_deref(),
            AnalyzedItem::Static(s) => s.documentation.as_deref(),
        }
    }

    pub fn source_location(&self) -> Option<&SourceLocation> {
        match self {
            AnalyzedItem::Function(f) => Some(&f.source_location),
            AnalyzedItem::Struct(s) => Some(&s.source_location),
            AnalyzedItem::Enum(e) => Some(&e.source_location),
            AnalyzedItem::Trait(t) => Some(&t.source_location),
            AnalyzedItem::Impl(i) => Some(&i.source_location),
            AnalyzedItem::Module(m) => Some(&m.source_location),
            AnalyzedItem::TypeAlias(t) => Some(&t.source_location),
            AnalyzedItem::Const(c) => Some(&c.source_location),
            AnalyzedItem::Static(s) => Some(&s.source_location),
        }
    }

    /// Get the module path for this item (e.g., ["serde", "de"])
    pub fn module_path(&self) -> &[String] {
        match self {
            AnalyzedItem::Function(f) => &f.module_path,
            AnalyzedItem::Struct(s) => &s.module_path,
            AnalyzedItem::Enum(e) => &e.module_path,
            AnalyzedItem::Trait(t) => &t.module_path,
            AnalyzedItem::Impl(i) => &i.module_path,
            AnalyzedItem::Module(m) => &m.module_path,
            AnalyzedItem::TypeAlias(t) => &t.module_path,
            AnalyzedItem::Const(c) => &c.module_path,
            AnalyzedItem::Static(s) => &s.module_path,
        }
    }

    /// Get fully qualified path (e.g., "serde::de::Deserialize")
    pub fn qualified_name(&self) -> String {
        let path = self.module_path();
        if path.is_empty() {
            self.name().to_string()
        } else {
            format!("{}::{}", path.join("::"), self.name())
        }
    }

    /// Get full definition as code string
    pub fn definition(&self) -> String {
        match self {
            AnalyzedItem::Function(f) => f.signature.clone(),
            AnalyzedItem::Struct(s) => s.full_definition(),
            AnalyzedItem::Enum(e) => e.full_definition(),
            AnalyzedItem::Trait(t) => t.full_definition(),
            AnalyzedItem::Impl(i) => i.full_definition(),
            AnalyzedItem::Module(m) => format!("mod {}", m.name),
            AnalyzedItem::TypeAlias(t) => format!("type {} = {}", t.name, t.ty),
            AnalyzedItem::Const(c) => format!("const {}: {}", c.name, c.ty),
            AnalyzedItem::Static(s) => {
                let mut_str = if s.is_mut { "mut " } else { "" };
                format!("static {}{}: {}", mut_str, s.name, s.ty)
            }
        }
    }
}

/// Information about a function
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub signature: String,
    pub visibility: Visibility,
    pub is_async: bool,
    pub is_const: bool,
    pub is_unsafe: bool,
    pub generics: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub documentation: Option<String>,
    pub attributes: Vec<String>,
    pub where_clause: Option<String>,
    pub source_location: SourceLocation,
    /// Module path for fully qualified naming (e.g., ["serde", "de"])
    pub module_path: Vec<String>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: String,
    pub is_self: bool,
    pub is_mut: bool,
    pub is_ref: bool,
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_self {
            if self.is_ref && self.is_mut {
                write!(f, "&mut self")
            } else if self.is_ref {
                write!(f, "&self")
            } else if self.is_mut {
                write!(f, "mut self")
            } else {
                write!(f, "self")
            }
        } else {
            write!(f, "{}: {}", self.name, self.ty)
        }
    }
}

/// Information about a struct
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub visibility: Visibility,
    pub generics: Vec<String>,
    pub fields: Vec<Field>,
    pub kind: StructKind,
    pub documentation: Option<String>,
    pub derives: Vec<String>,
    pub attributes: Vec<String>,
    pub where_clause: Option<String>,
    pub source_location: SourceLocation,
    /// Module path for fully qualified naming
    pub module_path: Vec<String>,
}

impl StructInfo {
    pub fn full_definition(&self) -> String {
        let vis = if self.visibility == Visibility::Public {
            "pub "
        } else {
            ""
        };
        let generics = if self.generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", self.generics.join(", "))
        };

        match self.kind {
            StructKind::Named => {
                let fields: Vec<String> = self
                    .fields
                    .iter()
                    .map(|f| {
                        let fvis = if f.visibility == Visibility::Public {
                            "pub "
                        } else {
                            ""
                        };
                        format!("    {}{}: {}", fvis, f.name, f.ty)
                    })
                    .collect();
                format!(
                    "{}struct {}{} {{\n{}\n}}",
                    vis,
                    self.name,
                    generics,
                    fields.join(",\n")
                )
            }
            StructKind::Tuple => {
                let fields: Vec<String> = self
                    .fields
                    .iter()
                    .map(|f| {
                        let fvis = if f.visibility == Visibility::Public {
                            "pub "
                        } else {
                            ""
                        };
                        format!("{}{}", fvis, f.ty)
                    })
                    .collect();
                format!(
                    "{}struct {}{}({});",
                    vis,
                    self.name,
                    generics,
                    fields.join(", ")
                )
            }
            StructKind::Unit => format!("{}struct {}{};", vis, self.name, generics),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructKind {
    Named,
    Tuple,
    Unit,
}

/// Struct/enum field
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: String,
    pub visibility: Visibility,
    pub documentation: Option<String>,
}

/// Information about an enum
#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub visibility: Visibility,
    pub generics: Vec<String>,
    pub variants: Vec<Variant>,
    pub documentation: Option<String>,
    pub derives: Vec<String>,
    pub attributes: Vec<String>,
    pub where_clause: Option<String>,
    pub source_location: SourceLocation,
    /// Module path for fully qualified naming
    pub module_path: Vec<String>,
}

impl EnumInfo {
    pub fn full_definition(&self) -> String {
        let vis = if self.visibility == Visibility::Public {
            "pub "
        } else {
            ""
        };
        let generics = if self.generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", self.generics.join(", "))
        };

        let variants: Vec<String> = self
            .variants
            .iter()
            .map(|v| {
                let fields = match &v.fields {
                    VariantFields::Named(fields) => {
                        let f: Vec<_> = fields
                            .iter()
                            .map(|f| format!("{}: {}", f.name, f.ty))
                            .collect();
                        format!(" {{ {} }}", f.join(", "))
                    }
                    VariantFields::Unnamed(types) => format!("({})", types.join(", ")),
                    VariantFields::Unit => String::new(),
                };
                format!("    {}{}", v.name, fields)
            })
            .collect();

        format!(
            "{}enum {}{} {{\n{}\n}}",
            vis,
            self.name,
            generics,
            variants.join(",\n")
        )
    }
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub fields: VariantFields,
    pub discriminant: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone)]
pub enum VariantFields {
    Named(Vec<Field>),
    Unnamed(Vec<String>),
    Unit,
}

/// Information about a trait
#[derive(Debug, Clone)]
pub struct TraitInfo {
    pub name: String,
    pub visibility: Visibility,
    pub generics: Vec<String>,
    pub supertraits: Vec<String>,
    pub methods: Vec<TraitMethod>,
    pub associated_types: Vec<AssociatedType>,
    pub associated_consts: Vec<AssociatedConst>,
    pub documentation: Option<String>,
    pub is_unsafe: bool,
    pub is_auto: bool,
    pub where_clause: Option<String>,
    pub source_location: SourceLocation,
    /// Module path for fully qualified naming
    pub module_path: Vec<String>,
}

impl TraitInfo {
    pub fn full_definition(&self) -> String {
        let vis = if self.visibility == Visibility::Public {
            "pub "
        } else {
            ""
        };
        let unsafe_str = if self.is_unsafe { "unsafe " } else { "" };
        let auto_str = if self.is_auto { "auto " } else { "" };
        let generics = if self.generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", self.generics.join(", "))
        };
        let bounds = if self.supertraits.is_empty() {
            String::new()
        } else {
            format!(": {}", self.supertraits.join(" + "))
        };

        let mut items = Vec::new();
        for at in &self.associated_types {
            items.push(format!("    type {};", at.name));
        }
        for method in &self.methods {
            items.push(format!("    {};", method.signature));
        }

        format!(
            "{}{}{}trait {}{}{} {{\n{}\n}}",
            vis,
            unsafe_str,
            auto_str,
            self.name,
            generics,
            bounds,
            items.join("\n")
        )
    }
}

/// Trait method signature
#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub signature: String,
    pub has_default: bool,
    pub is_async: bool,
    pub documentation: Option<String>,
}

/// Associated type in a trait
#[derive(Debug, Clone)]
pub struct AssociatedType {
    pub name: String,
    pub bounds: Vec<String>,
    pub default: Option<String>,
}

/// Associated const in a trait
#[derive(Debug, Clone)]
pub struct AssociatedConst {
    pub name: String,
    pub ty: String,
    pub default: Option<String>,
}

/// Information about an impl block
#[derive(Debug, Clone)]
pub struct ImplInfo {
    pub self_ty: String,
    pub trait_name: Option<String>,
    pub generics: Vec<String>,
    pub methods: Vec<FunctionInfo>,
    pub is_unsafe: bool,
    pub is_negative: bool,
    pub where_clause: Option<String>,
    pub source_location: SourceLocation,
    /// Module path for fully qualified naming
    pub module_path: Vec<String>,
}

impl ImplInfo {
    pub fn full_definition(&self) -> String {
        let unsafe_str = if self.is_unsafe { "unsafe " } else { "" };
        let negative_str = if self.is_negative { "!" } else { "" };
        let generics = if self.generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", self.generics.join(", "))
        };

        match &self.trait_name {
            Some(trait_name) => format!(
                "{}impl{} {}{} for {}",
                unsafe_str, generics, negative_str, trait_name, self.self_ty
            ),
            None => format!("impl{} {}", generics, self.self_ty),
        }
    }
}

/// Information about a module
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub path: String,
    pub visibility: Visibility,
    pub items: Vec<String>,
    pub submodules: Vec<String>,
    pub documentation: Option<String>,
    pub is_inline: bool,
    pub source_location: SourceLocation,
    /// Module path for fully qualified naming
    pub module_path: Vec<String>,
}

/// Type alias information
#[derive(Debug, Clone)]
pub struct TypeAliasInfo {
    pub name: String,
    pub visibility: Visibility,
    pub generics: Vec<String>,
    pub ty: String,
    pub documentation: Option<String>,
    pub where_clause: Option<String>,
    pub source_location: SourceLocation,
    /// Module path for fully qualified naming
    pub module_path: Vec<String>,
}

/// Const item information
#[derive(Debug, Clone)]
pub struct ConstInfo {
    pub name: String,
    pub visibility: Visibility,
    pub ty: String,
    pub value: Option<String>,
    pub documentation: Option<String>,
    pub source_location: SourceLocation,
    /// Module path for fully qualified naming
    pub module_path: Vec<String>,
}

/// Static item information
#[derive(Debug, Clone)]
pub struct StaticInfo {
    pub name: String,
    pub visibility: Visibility,
    pub ty: String,
    pub is_mut: bool,
    pub documentation: Option<String>,
    pub source_location: SourceLocation,
    /// Module path for fully qualified naming
    pub module_path: Vec<String>,
}
