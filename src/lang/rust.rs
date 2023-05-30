use {
    super::Language,
    crate::{
        graph::Style,
        lsp_types::{DocumentSymbol, SymbolKind},
    },
};

pub(crate) struct Rust;

impl Language for Rust {
    fn symbol_style(&self, symbol: &DocumentSymbol) -> Vec<Style> {
        match symbol.kind {
            SymbolKind::Function | SymbolKind::Method => {
                vec![Style::CssClass("fn".to_string()), Style::Rounded]
            }
            SymbolKind::Interface => {
                vec![
                    Style::CssClass("interface".to_string()),
                    Style::Border(0),
                    Style::Rounded,
                ]
            }
            SymbolKind::Object if symbol.name.starts_with("impl") => {
                vec![Style::CssClass("method-block".to_string()), Style::Rounded]
            }
            SymbolKind::Module => {
                vec![Style::CssClass("module".to_string()), Style::Rounded]
            }
            _ => vec![],
        }
    }
}
