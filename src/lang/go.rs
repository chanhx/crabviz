use {
    super::Language,
    crate::{
        graph::Style,
        lsp_types::{DocumentSymbol, SymbolKind},
    },
};

pub(crate) struct Go;

impl Language for Go {
    fn symbol_style(&self, symbol: &DocumentSymbol) -> Vec<Style> {
        match symbol.kind {
            SymbolKind::Function | SymbolKind::Method => {
                vec![Style::CssClass("fn".to_string()), Style::Rounded]
            }
            SymbolKind::Interface => {
                vec![
                    Style::CssClass("interface".to_string()),
                    Style::Rounded,
                    Style::Border(0),
                ]
            }
            _ => vec![],
        }
    }
}
