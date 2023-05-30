use {
    super::Language,
    crate::{
        graph::Style,
        lsp_types::{DocumentSymbol, SymbolKind},
    },
};

pub(crate) struct Java;

impl Language for Java {
    fn symbol_style(&self, symbol: &DocumentSymbol) -> Vec<Style> {
        match symbol.kind {
            // different style for constructor?
            SymbolKind::Function | SymbolKind::Method | SymbolKind::Constructor => {
                vec![Style::CssClass("fn".to_string()), Style::Rounded]
            }
            SymbolKind::Interface => {
                vec![Style::CssClass("interface".to_string()), Style::Rounded]
            }
            SymbolKind::Class => {
                vec![Style::CssClass("method-block".to_string()), Style::Rounded]
            }
            _ => vec![],
        }
    }
}
