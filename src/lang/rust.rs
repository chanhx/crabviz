use {
    super::Language,
    crate::{
        graph::{CellStyle, TableStyle},
        lsp_types::{DocumentSymbol, SymbolKind},
    },
};

pub(crate) struct Rust;

impl Language for Rust {
    fn symbol_style(&self, symbol: &DocumentSymbol) -> Vec<CellStyle> {
        match symbol.kind {
            SymbolKind::Function | SymbolKind::Method => {
                vec![CellStyle::CssClass("fn".to_string()), CellStyle::Rounded]
            }
            SymbolKind::Interface => {
                let table_style = vec![TableStyle::CssClass("interface".to_string())];
                vec![CellStyle::Table(table_style), CellStyle::Border(0)]
            }
            SymbolKind::Object if symbol.name.starts_with("impl") => {
                let table_style = vec![
                    TableStyle::Border(0),
                    TableStyle::CssClass("method-block".to_string()),
                ];
                vec![CellStyle::Table(table_style), CellStyle::Border(0)]
            }
            SymbolKind::Module => {
                let table_style = vec![
                    TableStyle::Border(0),
                    TableStyle::CssClass("module".to_string()),
                ];
                vec![CellStyle::Table(table_style), CellStyle::Border(0)]
            }
            _ => vec![],
        }
    }
}
