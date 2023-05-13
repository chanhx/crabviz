use {
    super::Language,
    crate::{
        graph::{CellStyle, TableStyle},
        lsp_types::{DocumentSymbol, SymbolKind},
    },
};

pub(crate) struct Java;

impl Language for Java {
    fn symbol_style(&self, symbol: &DocumentSymbol) -> Vec<CellStyle> {
        match symbol.kind {
            // different style for constructor?
            SymbolKind::Function | SymbolKind::Method | SymbolKind::Constructor => {
                vec![CellStyle::CssClass("fn".to_string()), CellStyle::Rounded]
            }
            SymbolKind::Interface => {
                let table_style = vec![TableStyle::CssClass("interface".to_string())];
                vec![CellStyle::Table(table_style), CellStyle::Border(0)]
            }
            SymbolKind::Class => {
                let table_style = vec![
                    TableStyle::Border(0),
                    TableStyle::CssClass("method-block".to_string()),
                ];
                vec![CellStyle::Table(table_style), CellStyle::Border(0)]
            }
            _ => vec![],
        }
    }
}
