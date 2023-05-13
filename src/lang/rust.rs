use {
    super::Language,
    crate::lsp_types::{DocumentSymbol, SymbolKind},
    // lsp_types::{DocumentSymbol, SymbolKind},
    crate::{
        generator::FileOutline,
        graph::{CellStyle, TableStyle},
    },
};

pub(crate) struct Rust;

impl Language for Rust {
    fn all_functions<'a, 'b>(&'a self, outline: &'b FileOutline) -> Vec<&'b DocumentSymbol> {
        let (func_groups, funcs): (Vec<_>, Vec<_>) = outline
            .symbols
            .iter()
            .filter(|symbol| match symbol.kind {
                SymbolKind::Function | SymbolKind::Method => true,
                SymbolKind::Object | SymbolKind::Interface if !symbol.children.is_empty() => true,
                _ => false,
            })
            .partition(|symbol| !symbol.children.is_empty());

        func_groups
            .iter()
            .flat_map(|impl_block| {
                impl_block
                    .children
                    // .as_ref()
                    // .unwrap()
                    .iter()
                    .filter(|symbol| match symbol.kind {
                        SymbolKind::Function | SymbolKind::Method => true,
                        _ => false,
                    })
            })
            .chain(funcs)
            .collect()
    }

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
