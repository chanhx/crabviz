use {
    super::Language,
    crate::graph::{CellStyle, TableStyle},
    // lsp_types::{DocumentSymbol, SymbolKind},
    crate::lsp_types::{DocumentSymbol, SymbolKind},
};

pub(crate) struct Go;

impl Language for Go {
    fn symbol_style(&self, symbol: &DocumentSymbol) -> Vec<CellStyle> {
        match symbol.kind {
            SymbolKind::Function | SymbolKind::Method => {
                vec![CellStyle::CssClass("fn".to_string()), CellStyle::Rounded]
            }
            SymbolKind::Interface => {
                let table_style = vec![TableStyle::CssClass("interface".to_string())];
                vec![CellStyle::Table(table_style), CellStyle::Border(0)]
            }
            _ => vec![],
        }
    }
}
