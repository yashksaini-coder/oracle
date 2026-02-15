//! Inspector panel for displaying code item details

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{
        block::BorderType,
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget, Wrap,
    },
};

use crate::analyzer::{
    AnalyzedItem, ConstInfo, EnumInfo, FunctionInfo, ImplInfo, ModuleInfo, 
    StaticInfo, StructInfo, StructKind, TraitInfo, TypeAliasInfo, VariantFields, Visibility,
};
use crate::ui::theme::Theme;

/// Panel for inspecting code items with scrolling support
pub struct InspectorPanel<'a> {
    item: Option<&'a AnalyzedItem>,
    /// All items (for "Implementations" of a trait)
    all_items: Option<&'a [AnalyzedItem]>,
    theme: &'a Theme,
    focused: bool,
    scroll_offset: usize,
}

impl<'a> InspectorPanel<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            item: None,
            all_items: None,
            theme,
            focused: false,
            scroll_offset: 0,
        }
    }

    pub fn item(mut self, item: Option<&'a AnalyzedItem>) -> Self {
        self.item = item;
        self
    }

    pub fn all_items(mut self, items: Option<&'a [AnalyzedItem]>) -> Self {
        self.all_items = items;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn scroll(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    fn section_header(&self, title: &str) -> Line<'static> {
        Line::from(vec![
            Span::styled("┏━ ", self.theme.style_border_focused()),
            Span::styled(title.to_string(), self.theme.style_accent().add_modifier(Modifier::BOLD)),
            Span::styled(" ", self.theme.style_border()),
            Span::styled("━━━━━━━━━━━━━", self.theme.style_border()),
        ])
    }

    fn key_value(&self, key: &str, value: String) -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("  {} ", key), self.theme.style_dim()),
            Span::styled(value, self.theme.style_normal()),
        ])
    }

    fn badge(&self, text: &str, is_warning: bool) -> Span<'static> {
        let style = if is_warning {
            self.theme.style_error().add_modifier(Modifier::BOLD)
        } else {
            self.theme.style_keyword()
        };
        Span::styled(format!(" [{}] ", text), style)
    }

    fn render_empty(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.style_border())
            .title(" ◆ INSPECTOR ◆ ");

        let inner = block.inner(area);
        block.render(area, buf);

        let help_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  ⟳ ", self.theme.style_border_focused()),
                Span::styled(
                    "No item selected",
                    self.theme.style_muted(),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  Select an item from the list to view",
                self.theme.style_dim(),
            )),
            Line::from(Span::styled(
                "  detailed API information.",
                self.theme.style_dim(),
            )),
            Line::from(""),
            Line::from(self.section_header("NAVIGATION")),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("[↑↓]", self.theme.style_accent()),
                Span::raw(" or "),
                Span::styled("[j/k]", self.theme.style_accent()),
                Span::raw("  Navigate"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("[Enter]", self.theme.style_accent()),
                Span::raw(" or "),
                Span::styled("[→]", self.theme.style_accent()),
                Span::raw("  View details"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("[/]", self.theme.style_accent()),
                Span::raw("              Search items"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("[1-4]", self.theme.style_accent()),
                Span::raw("            Switch tabs"),
            ]),
        ];

        Paragraph::new(help_text)
            .wrap(Wrap { trim: false })
            .render(inner, buf);
    }

    fn render_function(&self, func: &FunctionInfo, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        // Header with name and badges
        let mut header = vec![
            Span::styled("fn ", self.theme.style_keyword()),
            Span::styled(
                func.name.clone(),
                self.theme.style_accent_bold().add_modifier(Modifier::UNDERLINED),
            ),
        ];

        if func.is_async {
            header.push(self.badge("async", false));
        }
        if func.is_const {
            header.push(self.badge("const", false));
        }
        if func.is_unsafe {
            header.push(self.badge("unsafe", true));
        }

        lines.push(Line::from(header));
        
        // Show qualified path if present
        if !func.module_path.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  use ", self.theme.style_keyword()),
                Span::styled(format!("{}::{}", func.module_path.join("::"), func.name), self.theme.style_type()),
                Span::styled(";", self.theme.style_normal()),
            ]));
        }
        lines.push(Line::from(""));

        // Full signature with syntax highlighting
        lines.push(self.section_header("Signature"));
        lines.push(Line::from(""));
        for sig_line in func.signature.lines() {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(sig_line.to_string(), self.theme.style_function()),
            ]));
        }

        // Source Location
        if func.source_location.file.is_some() {
            lines.push(Line::from(""));
            lines.push(self.section_header("Source"));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  📍 "),
                Span::styled(func.source_location.to_string(), self.theme.style_muted()),
            ]));
        }

        // Overview
        lines.push(Line::from(""));
        lines.push(self.section_header("Overview"));
        lines.push(Line::from(""));
        lines.push(self.key_value("Visibility:", func.visibility.to_string()));
        
        // Function properties
        let mut props = Vec::new();
        if func.is_async { props.push("async"); }
        if func.is_const { props.push("const"); }
        if func.is_unsafe { props.push("unsafe"); }
        if !props.is_empty() {
            lines.push(self.key_value("Modifiers:", props.join(", ")));
        }
        
        if !func.generics.is_empty() {
            lines.push(self.key_value("Generics:", format!("<{}>", func.generics.join(", "))));
        }

        // Parameters section with detailed analysis
        if !func.parameters.is_empty() {
            lines.push(Line::from(""));
            lines.push(self.section_header(&format!("Parameters ({})", func.parameters.len())));
            lines.push(Line::from(""));

            for (i, param) in func.parameters.iter().enumerate() {
                if param.is_self {
                    let self_str = if param.is_ref && param.is_mut {
                        "&mut self"
                    } else if param.is_ref {
                        "&self"
                    } else if param.is_mut {
                        "mut self"
                    } else {
                        "self"
                    };
                    
                    let ownership_hint = if param.is_ref && param.is_mut {
                        "mutable borrow"
                    } else if param.is_ref {
                        "immutable borrow"
                    } else {
                        "takes ownership"
                    };
                    
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {}. ", i + 1), self.theme.style_number()),
                        Span::styled(self_str, self.theme.style_keyword()),
                    ]));
                    lines.push(Line::from(vec![
                        Span::raw("       "),
                        Span::styled(format!("↳ {}", ownership_hint), self.theme.style_muted()),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {}. ", i + 1), self.theme.style_number()),
                        Span::styled(param.name.clone(), self.theme.style_accent()),
                        Span::styled(": ", self.theme.style_muted()),
                        Span::styled(param.ty.clone(), self.theme.style_type()),
                    ]));
                    
                    // Ownership/borrowing analysis
                    let ty_str = &param.ty;
                    let hint = if ty_str.starts_with('&') && ty_str.contains("mut") {
                        Some("↳ mutable borrow")
                    } else if ty_str.starts_with('&') {
                        Some("↳ immutable borrow")
                    } else if ty_str.contains("impl ") || ty_str.contains("dyn ") {
                        Some("↳ trait object/impl")
                    } else if param.is_mut {
                        Some("↳ mutable binding")
                    } else {
                        None
                    };
                    
                    if let Some(h) = hint {
                        lines.push(Line::from(vec![
                            Span::raw("       "),
                            Span::styled(h.to_string(), self.theme.style_muted()),
                        ]));
                    }
                }
            }
        } else {
            lines.push(Line::from(""));
            lines.push(self.section_header("Parameters"));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("None (zero-argument function)", self.theme.style_muted()),
            ]));
        }

        // Return type section
        lines.push(Line::from(""));
        lines.push(self.section_header("Returns"));
        lines.push(Line::from(""));
        
        if let Some(ref ret) = func.return_type {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("→ ", self.theme.style_accent()),
                Span::styled(ret.clone(), self.theme.style_type()),
            ]));
            
            // Add helpful hints for common return types
            let ret_lower = ret.to_lowercase();
            if ret_lower.contains("result") {
                lines.push(Line::from(vec![
                    Span::raw("       "),
                    Span::styled("⚠ Can fail - handle errors appropriately", self.theme.style_warning()),
                ]));
            } else if ret_lower.contains("option") {
                lines.push(Line::from(vec![
                    Span::raw("       "),
                    Span::styled("⚠ May return None", self.theme.style_warning()),
                ]));
            }
        } else {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("→ ", self.theme.style_accent()),
                Span::styled("() ", self.theme.style_type()),
                Span::styled("(unit type)", self.theme.style_muted()),
            ]));
        }

        // Where clause
        if let Some(ref where_clause) = func.where_clause {
            lines.push(Line::from(""));
            lines.push(self.section_header("Constraints"));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("where ", self.theme.style_keyword()),
                Span::styled(where_clause.clone(), self.theme.style_type()),
            ]));
        }

        // Documentation
        if let Some(ref docs) = func.documentation {
            lines.push(Line::from(""));
            lines.push(self.section_header("Documentation"));
            lines.push(Line::from(""));
            for doc_line in docs.lines() {
                let trimmed = doc_line.trim_start_matches('/').trim_start();
                lines.push(Line::from(Span::styled(
                    format!("  {}", trimmed),
                    self.theme.style_comment(),
                )));
            }
        }

        // Attributes
        if !func.attributes.is_empty() {
            lines.push(Line::from(""));
            lines.push(self.section_header("Attributes"));
            lines.push(Line::from(""));
            for attr in &func.attributes {
                lines.push(Line::from(Span::styled(
                    format!("  #[{}]", attr),
                    self.theme.style_muted(),
                )));
            }
        }

        self.render_panel(" 🔧 Function ", lines, area, buf);
    }

    fn render_struct(&self, st: &StructInfo, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        // Header with type badge
        let kind_str = match st.kind {
            StructKind::Named => "struct",
            StructKind::Tuple => "tuple struct", 
            StructKind::Unit => "unit struct",
        };

        lines.push(Line::from(vec![
            Span::styled("struct ", self.theme.style_keyword()),
            Span::styled(st.name.clone(), self.theme.style_accent_bold().add_modifier(Modifier::UNDERLINED)),
            Span::raw(" "),
            Span::styled(format!("({})", kind_str), self.theme.style_muted()),
        ]));
        
        // Show qualified path if present
        if !st.module_path.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  use ", self.theme.style_keyword()),
                Span::styled(format!("{}::{}", st.module_path.join("::"), st.name), self.theme.style_type()),
                Span::styled(";", self.theme.style_normal()),
            ]));
        }
        lines.push(Line::from(""));

        // Full Definition
        lines.push(self.section_header("Definition"));
        lines.push(Line::from(""));
        for line in st.full_definition().lines() {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(line.to_string(), self.theme.style_function()),
            ]));
        }

        // Source Location
        if st.source_location.file.is_some() {
            lines.push(Line::from(""));
            lines.push(self.section_header("Source"));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  📍 "),
                Span::styled(st.source_location.to_string(), self.theme.style_muted()),
            ]));
        }

        // Overview
        lines.push(Line::from(""));
        lines.push(self.section_header("Overview"));
        lines.push(Line::from(""));
        lines.push(self.key_value("Visibility:", st.visibility.to_string()));
        lines.push(self.key_value("Kind:", kind_str.to_string()));
        lines.push(self.key_value("Field Count:", st.fields.len().to_string()));
        
        if !st.generics.is_empty() {
            lines.push(self.key_value("Generics:", format!("<{}>", st.generics.join(", "))));
        }

        if let Some(ref wc) = st.where_clause {
            lines.push(self.key_value("Where:", wc.clone()));
        }

        // Derives with categorization
        if !st.derives.is_empty() {
            lines.push(Line::from(""));
            lines.push(self.section_header("Derived Traits"));
            lines.push(Line::from(""));
            
            // Categorize derives
            let (standard, serde_derives, other): (Vec<_>, Vec<_>, Vec<_>) = st.derives.iter()
                .fold((vec![], vec![], vec![]), |mut acc, d| {
                    let d_lower = d.to_lowercase();
                    if ["debug", "clone", "copy", "default", "partialeq", "eq", "partialord", "ord", "hash"].contains(&d_lower.as_str()) {
                        acc.0.push(d);
                    } else if d_lower.contains("serde") || d_lower == "serialize" || d_lower == "deserialize" {
                        acc.1.push(d);
                    } else {
                        acc.2.push(d);
                    }
                    acc
                });
            
            if !standard.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("Standard: ", self.theme.style_dim()),
                    Span::styled(standard.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "), self.theme.style_type()),
                ]));
            }
            if !serde_derives.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("Serde: ", self.theme.style_dim()),
                    Span::styled(serde_derives.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "), self.theme.style_type()),
                ]));
            }
            if !other.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("Other: ", self.theme.style_dim()),
                    Span::styled(other.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "), self.theme.style_type()),
                ]));
            }
        }

        // Fields with detailed info
        if !st.fields.is_empty() {
            lines.push(Line::from(""));
            lines.push(self.section_header(&format!("Fields ({})", st.fields.len())));
            lines.push(Line::from(""));

            for (i, field) in st.fields.iter().enumerate() {
                let vis_str = match field.visibility {
                    Visibility::Public => "pub ",
                    Visibility::Crate => "pub(crate) ",
                    Visibility::Super => "pub(super) ",
                    _ => "",
                };
                
                lines.push(Line::from(vec![
                    Span::styled(format!("  {}. ", i + 1), self.theme.style_number()),
                    Span::styled(vis_str.to_string(), self.theme.style_keyword()),
                    Span::styled(field.name.clone(), self.theme.style_accent()),
                    Span::styled(": ", self.theme.style_muted()),
                    Span::styled(field.ty.clone(), self.theme.style_type()),
                ]));
                
                // Type analysis hints
                let ty_lower = field.ty.to_lowercase();
                if ty_lower.contains("option") {
                    lines.push(Line::from(vec![
                        Span::raw("       "),
                        Span::styled("⚪ Optional field", self.theme.style_muted()),
                    ]));
                } else if ty_lower.contains("vec") || ty_lower.contains("hashmap") || ty_lower.contains("btreemap") {
                    lines.push(Line::from(vec![
                        Span::raw("       "),
                        Span::styled("📦 Collection type", self.theme.style_muted()),
                    ]));
                } else if ty_lower.contains("box") || ty_lower.contains("rc") || ty_lower.contains("arc") {
                    lines.push(Line::from(vec![
                        Span::raw("       "),
                        Span::styled("🔗 Heap-allocated/Shared", self.theme.style_muted()),
                    ]));
                }
                
                if let Some(ref doc) = field.documentation {
                    for doc_line in doc.lines().take(2) {
                        let trimmed = doc_line.trim_start_matches('/').trim_start();
                        if !trimmed.is_empty() {
                            lines.push(Line::from(vec![
                                Span::raw("       "),
                                Span::styled(trimmed.to_string(), self.theme.style_comment()),
                            ]));
                        }
                    }
                }
            }
        }

        // Type usage hints
        lines.push(Line::from(""));
        lines.push(self.section_header("Usage"));
        lines.push(Line::from(""));
        
        // Construction hint
        match st.kind {
            StructKind::Named => {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("let instance = ", self.theme.style_dim()),
                    Span::styled(format!("{} {{ ... }};", st.name), self.theme.style_function()),
                ]));
            }
            StructKind::Tuple => {
                let placeholders = (0..st.fields.len()).map(|_| "_").collect::<Vec<_>>().join(", ");
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("let instance = ", self.theme.style_dim()),
                    Span::styled(format!("{}({});", st.name, placeholders), self.theme.style_function()),
                ]));
            }
            StructKind::Unit => {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("let instance = ", self.theme.style_dim()),
                    Span::styled(format!("{};", st.name), self.theme.style_function()),
                ]));
            }
        }

        // Show Default hint if derived
        if st.derives.iter().any(|d| d.to_lowercase() == "default") {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("let instance = ", self.theme.style_dim()),
                Span::styled(format!("{}::default();", st.name), self.theme.style_function()),
            ]));
        }

        // Documentation
        if let Some(ref docs) = st.documentation {
            lines.push(Line::from(""));
            lines.push(self.section_header("Documentation"));
            lines.push(Line::from(""));
            for doc_line in docs.lines() {
                let trimmed = doc_line.trim_start_matches('/').trim_start();
                lines.push(Line::from(Span::styled(
                    format!("  {}", trimmed),
                    self.theme.style_comment(),
                )));
            }
        }

        // Attributes
        if !st.attributes.is_empty() {
            lines.push(Line::from(""));
            lines.push(self.section_header("Attributes"));
            lines.push(Line::from(""));
            for attr in &st.attributes {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("#[{}]", attr), self.theme.style_muted()),
                ]));
            }
        }

        self.render_panel(" 📦 Struct ", lines, area, buf);
    }

    fn render_enum(&self, en: &EnumInfo, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("enum ", self.theme.style_keyword()),
            Span::styled(en.name.clone(), self.theme.style_accent_bold().add_modifier(Modifier::UNDERLINED)),
        ]));
        
        // Show qualified path if present
        if !en.module_path.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  use ", self.theme.style_keyword()),
                Span::styled(format!("{}::{}", en.module_path.join("::"), en.name), self.theme.style_type()),
                Span::styled(";", self.theme.style_normal()),
            ]));
        }
        lines.push(Line::from(""));

        // Overview
        lines.push(self.section_header("Overview"));
        lines.push(Line::from(""));
        lines.push(self.key_value("Visibility:", en.visibility.to_string()));
        lines.push(self.key_value("Variants:", en.variants.len().to_string()));
        
        if !en.generics.is_empty() {
            lines.push(self.key_value("Generics:", format!("<{}>", en.generics.join(", "))));
        }

        // Derives
        if !en.derives.is_empty() {
            lines.push(Line::from(""));
            lines.push(self.section_header("Derived Traits"));
            lines.push(Line::from(""));
            for derive in &en.derives {
                lines.push(Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(derive.clone(), self.theme.style_type()),
                ]));
            }
        }

        // Variants
        lines.push(Line::from(""));
        lines.push(self.section_header(&format!("Variants ({})", en.variants.len())));
        lines.push(Line::from(""));

        for (i, variant) in en.variants.iter().enumerate() {
            let fields_str = match &variant.fields {
                VariantFields::Named(fields) => {
                    let f: Vec<_> = fields.iter().map(|f| format!("{}: {}", f.name, f.ty)).collect();
                    format!(" {{ {} }}", f.join(", "))
                }
                VariantFields::Unnamed(types) => format!("({})", types.join(", ")),
                VariantFields::Unit => String::new(),
            };

            let discriminant = variant.discriminant.as_ref()
                .map(|d| format!(" = {}", d))
                .unwrap_or_default();

            lines.push(Line::from(vec![
                Span::styled(format!("  {}. ", i + 1), self.theme.style_dim()),
                Span::styled(variant.name.clone(), self.theme.style_type()),
                Span::styled(fields_str, self.theme.style_muted()),
                Span::styled(discriminant, self.theme.style_number()),
            ]));

            if let Some(ref doc) = variant.documentation {
                let first_line = doc.lines().next().unwrap_or("");
                lines.push(Line::from(vec![
                    Span::raw("       "),
                    Span::styled(first_line.to_string(), self.theme.style_comment()),
                ]));
            }
        }

        // Documentation
        if let Some(ref docs) = en.documentation {
            lines.push(Line::from(""));
            lines.push(self.section_header("Documentation"));
            lines.push(Line::from(""));
            for doc_line in docs.lines() {
                let trimmed = doc_line.trim_start_matches('/').trim_start();
                lines.push(Line::from(Span::styled(
                    format!("  {}", trimmed),
                    self.theme.style_comment(),
                )));
            }
        }

        self.render_panel(" 🏷️ Enum ", lines, area, buf);
    }

    fn render_trait(&self, tr: &TraitInfo, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        let mut header = vec![
            Span::styled("trait ", self.theme.style_keyword()),
            Span::styled(tr.name.clone(), self.theme.style_accent_bold().add_modifier(Modifier::UNDERLINED)),
        ];

        if tr.is_unsafe {
            header.push(self.badge("unsafe", true));
        }
        if tr.is_auto {
            header.push(self.badge("auto", false));
        }

        lines.push(Line::from(header));
        
        // Show qualified path if present
        if !tr.module_path.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  use ", self.theme.style_keyword()),
                Span::styled(format!("{}::{}", tr.module_path.join("::"), tr.name), self.theme.style_type()),
                Span::styled(";", self.theme.style_normal()),
            ]));
        }
        lines.push(Line::from(""));

        // Overview
        lines.push(self.section_header("Overview"));
        lines.push(Line::from(""));
        lines.push(self.key_value("Visibility:", tr.visibility.to_string()));
        lines.push(self.key_value("Methods:", tr.methods.len().to_string()));
        
        if !tr.associated_types.is_empty() {
            lines.push(self.key_value("Associated Types:", tr.associated_types.len().to_string()));
        }

        // Supertraits
        if !tr.supertraits.is_empty() {
            lines.push(Line::from(""));
            lines.push(self.section_header("Supertraits"));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(tr.supertraits.join(" + "), self.theme.style_type()),
            ]));
        }

        // Associated types
        if !tr.associated_types.is_empty() {
            lines.push(Line::from(""));
            lines.push(self.section_header("Associated Types"));
            lines.push(Line::from(""));
            for at in &tr.associated_types {
                let bounds = if at.bounds.is_empty() {
                    String::new()
                } else {
                    format!(": {}", at.bounds.join(" + "))
                };
                let default = at.default.as_ref()
                    .map(|d| format!(" = {}", d))
                    .unwrap_or_default();
                
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("type ", self.theme.style_keyword()),
                    Span::styled(at.name.clone(), self.theme.style_type()),
                    Span::styled(bounds, self.theme.style_muted()),
                    Span::styled(default, self.theme.style_type()),
                ]));
            }
        }

        // Implementations (impl Trait for Type)
        if let Some(all) = self.all_items {
            let impls: Vec<&ImplInfo> = all
                .iter()
                .filter_map(|i| {
                    if let AnalyzedItem::Impl(im) = i {
                        let matches = im.trait_name.as_deref().map_or(false, |tn| {
                            tn == tr.name || tn.ends_with(&format!("::{}", tr.name))
                        });
                        if matches {
                            Some(im)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            if !impls.is_empty() {
                lines.push(Line::from(""));
                lines.push(self.section_header(&format!("Implementations ({})", impls.len())));
                lines.push(Line::from(""));
                for (i, im) in impls.iter().enumerate() {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {}. ", i + 1), self.theme.style_dim()),
                        Span::styled(im.full_definition(), self.theme.style_type()),
                    ]));
                }
            }
        }

        // Methods
        if !tr.methods.is_empty() {
            lines.push(Line::from(""));
            lines.push(self.section_header(&format!("Methods ({})", tr.methods.len())));
            lines.push(Line::from(""));

            for (i, method) in tr.methods.iter().enumerate() {
                let mut method_line = vec![
                    Span::styled(format!("  {}. ", i + 1), self.theme.style_dim()),
                    Span::styled("fn ", self.theme.style_keyword()),
                    Span::styled(method.name.clone(), self.theme.style_function()),
                ];

                if method.has_default {
                    method_line.push(Span::styled(" [default]", self.theme.style_success()));
                }
                if method.is_async {
                    method_line.push(Span::styled(" async", self.theme.style_keyword()));
                }

                lines.push(Line::from(method_line));

                if let Some(ref doc) = method.documentation {
                    let first_line = doc.lines().next().unwrap_or("");
                    lines.push(Line::from(vec![
                        Span::raw("       "),
                        Span::styled(first_line.to_string(), self.theme.style_comment()),
                    ]));
                }
            }
        }

        // Documentation
        if let Some(ref docs) = tr.documentation {
            lines.push(Line::from(""));
            lines.push(self.section_header("Documentation"));
            lines.push(Line::from(""));
            for doc_line in docs.lines() {
                let trimmed = doc_line.trim_start_matches('/').trim_start();
                lines.push(Line::from(Span::styled(
                    format!("  {}", trimmed),
                    self.theme.style_comment(),
                )));
            }
        }

        self.render_panel(" 📜 Trait ", lines, area, buf);
    }

    fn render_impl(&self, im: &ImplInfo, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        let title = if let Some(ref trait_name) = im.trait_name {
            format!("impl {} for {}", trait_name, im.self_ty)
        } else {
            format!("impl {}", im.self_ty)
        };

        let mut header = vec![
            Span::styled("impl ", self.theme.style_keyword()),
            Span::styled(title, self.theme.style_accent_bold()),
        ];

        if im.is_unsafe {
            header.push(self.badge("unsafe", true));
        }
        if im.is_negative {
            header.push(self.badge("negative", true));
        }

        lines.push(Line::from(header));
        lines.push(Line::from(""));

        // Overview
        lines.push(self.section_header("Overview"));
        lines.push(Line::from(""));
        lines.push(self.key_value("Type:", im.self_ty.clone()));
        
        if let Some(ref trait_name) = im.trait_name {
            lines.push(self.key_value("Trait:", trait_name.clone()));
        }
        
        lines.push(self.key_value("Methods:", im.methods.len().to_string()));
        
        if !im.generics.is_empty() {
            lines.push(self.key_value("Generics:", format!("<{}>", im.generics.join(", "))));
        }

        // Methods
        if !im.methods.is_empty() {
            lines.push(Line::from(""));
            lines.push(self.section_header(&format!("Methods ({})", im.methods.len())));
            lines.push(Line::from(""));

            for (i, method) in im.methods.iter().enumerate() {
                let vis = if method.visibility == Visibility::Public {
                    "pub "
                } else {
                    ""
                };
                
                let mut method_line = vec![
                    Span::styled(format!("  {}. ", i + 1), self.theme.style_dim()),
                    Span::styled(vis.to_string(), self.theme.style_keyword()),
                    Span::styled("fn ", self.theme.style_keyword()),
                    Span::styled(method.name.clone(), self.theme.style_function()),
                ];

                // Show parameter count
                let param_count = method.parameters.len();
                method_line.push(Span::styled(
                    format!("({} params)", param_count),
                    self.theme.style_muted(),
                ));

                // Show return type hint
                if let Some(ref ret) = method.return_type {
                    method_line.push(Span::styled(" → ", self.theme.style_accent()));
                    method_line.push(Span::styled(ret.clone(), self.theme.style_type()));
                }

                lines.push(Line::from(method_line));
            }
        }

        // Where clause
        if let Some(ref where_clause) = im.where_clause {
            lines.push(Line::from(""));
            lines.push(self.section_header("Constraints"));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("where ", self.theme.style_keyword()),
                Span::styled(where_clause.clone(), self.theme.style_type()),
            ]));
        }

        self.render_panel(" ⚙️ Implementation ", lines, area, buf);
    }

    fn render_module(&self, module: &ModuleInfo, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("mod ", self.theme.style_keyword()),
            Span::styled(module.name.clone(), self.theme.style_accent_bold().add_modifier(Modifier::UNDERLINED)),
        ]));
        lines.push(Line::from(""));

        // Overview
        lines.push(self.section_header("Overview"));
        lines.push(Line::from(""));
        lines.push(self.key_value("Visibility:", module.visibility.to_string()));
        lines.push(self.key_value("Path:", module.path.clone()));
        lines.push(self.key_value("Inline:", if module.is_inline { "yes" } else { "no" }.to_string()));

        // Submodules (flow / tree)
        if !module.submodules.is_empty() {
            lines.push(Line::from(""));
            lines.push(self.section_header(&format!("Submodules / flow ({})", module.submodules.len())));
            lines.push(Line::from(""));
            let n = module.submodules.len();
            for (i, submod) in module.submodules.iter().enumerate() {
                let connector = if i == n - 1 { "└── " } else { "├── " };
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(connector, self.theme.style_muted()),
                    Span::styled(submod.clone(), self.theme.style_accent()),
                ]));
            }
        }

        // Items
        if !module.items.is_empty() {
            lines.push(Line::from(""));
            lines.push(self.section_header(&format!("Items ({})", module.items.len())));
            lines.push(Line::from(""));
            for item in module.items.iter().take(20) {
                lines.push(Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(item.clone(), self.theme.style_normal()),
                ]));
            }
            if module.items.len() > 20 {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("... and {} more", module.items.len() - 20), self.theme.style_muted()),
                ]));
            }
        }

        // Documentation
        if let Some(ref docs) = module.documentation {
            lines.push(Line::from(""));
            lines.push(self.section_header("Documentation"));
            lines.push(Line::from(""));
            for doc_line in docs.lines() {
                let trimmed = doc_line.trim_start_matches('/').trim_start();
                lines.push(Line::from(Span::styled(
                    format!("  {}", trimmed),
                    self.theme.style_comment(),
                )));
            }
        }

        self.render_panel(" 📁 Module ", lines, area, buf);
    }

    fn render_type_alias(&self, alias: &TypeAliasInfo, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("type ", self.theme.style_keyword()),
            Span::styled(alias.name.clone(), self.theme.style_accent_bold().add_modifier(Modifier::UNDERLINED)),
            Span::styled(" = ", self.theme.style_muted()),
            Span::styled(alias.ty.clone(), self.theme.style_type()),
        ]));
        lines.push(Line::from(""));

        lines.push(self.section_header("Overview"));
        lines.push(Line::from(""));
        lines.push(self.key_value("Visibility:", alias.visibility.to_string()));
        lines.push(self.key_value("Aliased Type:", alias.ty.clone()));
        
        if !alias.generics.is_empty() {
            lines.push(self.key_value("Generics:", format!("<{}>", alias.generics.join(", "))));
        }

        if let Some(ref docs) = alias.documentation {
            lines.push(Line::from(""));
            lines.push(self.section_header("Documentation"));
            lines.push(Line::from(""));
            for doc_line in docs.lines() {
                let trimmed = doc_line.trim_start_matches('/').trim_start();
                lines.push(Line::from(Span::styled(
                    format!("  {}", trimmed),
                    self.theme.style_comment(),
                )));
            }
        }

        self.render_panel(" 🔗 Type Alias ", lines, area, buf);
    }

    fn render_const(&self, c: &ConstInfo, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("const ", self.theme.style_keyword()),
            Span::styled(c.name.clone(), self.theme.style_accent_bold().add_modifier(Modifier::UNDERLINED)),
            Span::styled(": ", self.theme.style_muted()),
            Span::styled(c.ty.clone(), self.theme.style_type()),
        ]));
        lines.push(Line::from(""));

        lines.push(self.section_header("Overview"));
        lines.push(Line::from(""));
        lines.push(self.key_value("Visibility:", c.visibility.to_string()));
        lines.push(self.key_value("Type:", c.ty.clone()));
        
        if let Some(ref value) = c.value {
            lines.push(self.key_value("Value:", value.clone()));
        }

        if let Some(ref docs) = c.documentation {
            lines.push(Line::from(""));
            lines.push(self.section_header("Documentation"));
            lines.push(Line::from(""));
            for doc_line in docs.lines() {
                let trimmed = doc_line.trim_start_matches('/').trim_start();
                lines.push(Line::from(Span::styled(
                    format!("  {}", trimmed),
                    self.theme.style_comment(),
                )));
            }
        }

        self.render_panel(" 📌 Constant ", lines, area, buf);
    }

    fn render_static(&self, s: &StaticInfo, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        let mut header = vec![
            Span::styled("static ", self.theme.style_keyword()),
        ];
        
        if s.is_mut {
            header.push(Span::styled("mut ", self.theme.style_keyword()));
        }
        
        header.push(Span::styled(s.name.clone(), self.theme.style_accent_bold().add_modifier(Modifier::UNDERLINED)));
        header.push(Span::styled(": ", self.theme.style_muted()));
        header.push(Span::styled(s.ty.clone(), self.theme.style_type()));

        if s.is_mut {
            header.push(self.badge("mutable", true));
        }

        lines.push(Line::from(header));
        lines.push(Line::from(""));

        lines.push(self.section_header("Overview"));
        lines.push(Line::from(""));
        lines.push(self.key_value("Visibility:", s.visibility.to_string()));
        lines.push(self.key_value("Type:", s.ty.clone()));
        lines.push(self.key_value("Mutable:", if s.is_mut { "yes ⚠️" } else { "no" }.to_string()));

        if s.is_mut {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("⚠ Warning: ", self.theme.style_error()),
                Span::styled("Mutable statics are unsafe!", self.theme.style_warning()),
            ]));
        }

        if let Some(ref docs) = s.documentation {
            lines.push(Line::from(""));
            lines.push(self.section_header("Documentation"));
            lines.push(Line::from(""));
            for doc_line in docs.lines() {
                let trimmed = doc_line.trim_start_matches('/').trim_start();
                lines.push(Line::from(Span::styled(
                    format!("  {}", trimmed),
                    self.theme.style_comment(),
                )));
            }
        }

        self.render_panel(" 🌐 Static ", lines, area, buf);
    }

    fn render_panel(&self, title: &str, lines: Vec<Line<'static>>, area: Rect, buf: &mut Buffer) {
        let total_lines = lines.len();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if self.focused {
                self.theme.style_border_focused()
            } else {
                self.theme.style_border()
            })
            .title(title);

        let inner = block.inner(area);
        block.render(area, buf);

        // Apply scroll offset
        let visible_lines: Vec<Line> = lines
            .into_iter()
            .skip(self.scroll_offset)
            .collect();

        Paragraph::new(visible_lines)
            .wrap(Wrap { trim: false })
            .render(inner, buf);

        // Render scrollbar if content exceeds view
        if total_lines > inner.height as usize {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            
            let mut scrollbar_state = ScrollbarState::new(total_lines)
                .position(self.scroll_offset);
            
            scrollbar.render(inner, buf, &mut scrollbar_state);
        }
    }
}

impl Widget for InspectorPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.item {
            None => self.render_empty(area, buf),
            Some(AnalyzedItem::Function(f)) => self.render_function(f, area, buf),
            Some(AnalyzedItem::Struct(s)) => self.render_struct(s, area, buf),
            Some(AnalyzedItem::Enum(e)) => self.render_enum(e, area, buf),
            Some(AnalyzedItem::Trait(t)) => self.render_trait(t, area, buf),
            Some(AnalyzedItem::Impl(i)) => self.render_impl(i, area, buf),
            Some(AnalyzedItem::Module(m)) => self.render_module(m, area, buf),
            Some(AnalyzedItem::TypeAlias(t)) => self.render_type_alias(t, area, buf),
            Some(AnalyzedItem::Const(c)) => self.render_const(c, area, buf),
            Some(AnalyzedItem::Static(s)) => self.render_static(s, area, buf),
        }
    }
}
