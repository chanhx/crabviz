mod go;
mod rust;

use {
    self::{go::Go, rust::Rust},
    crate::{
        generator::{FileOutline, PathMap, Relations},
        graph::{Cell, CellStyle, Edge, EdgeStyle, TableNode},
        lsp_types::{DocumentSymbol, SymbolKind},
    },
    std::path::Path,
};

pub(crate) trait Language {
    fn all_functions<'a, 'b>(&'a self, outline: &'b FileOutline) -> Vec<&'b DocumentSymbol> {
        outline
            .symbols
            .iter()
            .filter(|symbol| match symbol.kind {
                SymbolKind::Function | SymbolKind::Method => true,
                _ => false,
            })
            .collect()
    }

    fn all_interfaces<'a, 'b>(&'a self, outline: &'b FileOutline) -> Vec<&'b DocumentSymbol> {
        outline
            .symbols
            .iter()
            .filter(|symbol| symbol.kind == SymbolKind::Interface)
            .collect()
    }

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
            // .as_ref()
            // .unwrap_or(&vec![])
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
            SymbolKind::Function | SymbolKind::Method => {
                vec![CellStyle::CssClass("fn".to_string()), CellStyle::Rounded]
            }
            _ => vec![],
        }
    }

    fn calling_repr(&self, relations: Relations, map: &PathMap) -> Vec<Edge> {
        relations
            .iter()
            .flat_map(|rels| {
                let from_table_id = map.get(&rels.0.path.clone()).unwrap().to_string();
                let from_node_id = format!("{}_{}", rels.0.line, rels.0.character);

                rels.1
                    .iter()
                    .map(|location| Edge {
                        from_table_id: from_table_id.clone(),
                        from_node_id: from_node_id.clone(),
                        to_table_id: map.get(&location.path.clone()).unwrap().to_string(),
                        to_node_id: format!("{}_{}", location.line, location.character),
                        styles: vec![],
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn impl_repr(&self, relations: Relations, map: &PathMap) -> Vec<Edge> {
        relations
            .iter()
            .flat_map(|rels| {
                let to_table_id = map.get(&rels.0.path.clone()).unwrap().to_string();
                let to_node_id = format!("{}_{}", rels.0.line, rels.0.character);

                rels.1
                    .iter()
                    .map(|location| Edge {
                        from_table_id: map.get(&location.path.clone()).unwrap().to_string(),
                        from_node_id: format!("{}_{}", location.line, location.character),
                        to_table_id: to_table_id.clone(),
                        to_node_id: to_node_id.clone(),
                        styles: vec![EdgeStyle::CssClass("impl".to_string())],
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    // fn handle_unrecognized_functions(&self, funcs: Vec<&DocumentSymbol>);
}

pub struct DefaultLang;

impl Language for DefaultLang {}

pub(crate) fn language_handler(lang: &str) -> Box<dyn Language + Sync + Send> {
    match lang {
        "rust" => Box::new(Rust {}),
        "go" => Box::new(Go {}),
        _ => Box::new(DefaultLang {}),
    }
}