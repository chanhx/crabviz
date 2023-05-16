mod go;
mod java;
mod rust;

use {
    self::{go::Go, java::Java, rust::Rust},
    crate::{
        generator::FileOutline,
        graph::{Cell, CellStyle, TableNode, TableStyle},
        lsp_types::{DocumentSymbol, SymbolKind},
    },
    std::path::Path,
};

pub(crate) trait Language {
    fn file_repr(&self, file: &FileOutline) -> TableNode {
        let sections = file
            .symbols
            .iter()
            .map(|symbol| self.symbol_repr(file.id, symbol, &file.path))
            .collect();

        TableNode {
            id: file.id.to_string(),
            title: file.path.file_name().unwrap().to_str().unwrap().to_string(),
            sections,
        }
    }

    fn symbol_repr(&self, file_id: u32, symbol: &DocumentSymbol, path: &Path) -> Cell {
        let styles = self.symbol_style(symbol);

        let children = symbol
            .children
            .iter()
            .map(|item| self.symbol_repr(file_id, item, path))
            .collect();

        let port = format!(
            "{}_{}",
            symbol.selection_range.start.line, symbol.selection_range.start.character
        )
        .to_string();

        Cell {
            port: port.clone(),
            id: format!("{}:{}", file_id, port),
            styles,
            title: symbol.name.clone(),
            children,
        }
    }

    fn symbol_style(&self, symbol: &DocumentSymbol) -> Vec<CellStyle> {
        match symbol.kind {
            SymbolKind::Function | SymbolKind::Method | SymbolKind::Constructor => {
                vec![CellStyle::CssClass("fn".to_string()), CellStyle::Rounded]
            }
            SymbolKind::Interface => {
                let table_style = vec![TableStyle::CssClass("interface".to_string())];
                vec![CellStyle::Table(table_style), CellStyle::Border(0)]
            }
            _ => vec![],
        }
    }

    // fn handle_unrecognized_functions(&self, funcs: Vec<&DocumentSymbol>);
}

pub struct DefaultLang;

impl Language for DefaultLang {}

pub(crate) fn language_handler(ext: &str) -> Box<dyn Language + Sync + Send> {
    match ext {
        "go" => Box::new(Go),
        "java" => Box::new(Java),
        "rs" => Box::new(Rust),
        _ => Box::new(DefaultLang {}),
    }
}
