mod go;
mod rust;

use {
    self::{go::Go, rust::Rust},
    crate::{
        generator::FileOutline,
        graph::{Cell, CssClass, Style, TableNode},
        lsp_types::{DocumentSymbol, SymbolKind},
    },
};

pub(crate) trait Language {
    fn should_filter_out_file(&self, _file: &str) -> bool {
        false
    }

    fn file_repr(&self, file: &FileOutline) -> TableNode {
        let sections = file
            .symbols
            .iter()
            .filter(|symbol| self.filter_symbol(symbol))
            .map(|symbol| self.symbol_repr(file.id, symbol))
            .collect();

        TableNode {
            id: file.id,
            title: file.path.file_name().unwrap().to_str().unwrap().to_string(),
            sections,
        }
    }

    fn symbol_repr(&self, file_id: u32, symbol: &DocumentSymbol) -> Cell {
        let styles = self.symbol_style(symbol);

        let children = symbol
            .children
            .iter()
            .filter(|symbol| self.filter_symbol(symbol))
            .map(|symbol| self.symbol_repr(file_id, symbol))
            .collect();

        let range = symbol.selection_range;

        Cell {
            range_start: (range.start.line, range.start.character),
            range_end: (range.end.line, range.end.character),
            styles,
            title: symbol.name.clone(),
            children,
        }
    }

    fn filter_symbol(&self, symbol: &DocumentSymbol) -> bool {
        match symbol.kind {
            SymbolKind::Constant
            | SymbolKind::Variable
            | SymbolKind::Field
            | SymbolKind::EnumMember => false,
            _ => true,
        }
    }

    fn symbol_style(&self, symbol: &DocumentSymbol) -> Vec<Style> {
        match symbol.kind {
            SymbolKind::Module => vec![Style::CssClass(CssClass::Module), Style::Rounded],
            SymbolKind::Function => vec![
                Style::CssClass(CssClass::Function),
                Style::CssClass(CssClass::Callable),
                Style::Rounded,
            ],
            SymbolKind::Method => vec![
                Style::CssClass(CssClass::Method),
                Style::CssClass(CssClass::Callable),
                Style::Rounded,
            ],
            SymbolKind::Constructor => vec![
                Style::CssClass(CssClass::Constructor),
                Style::CssClass(CssClass::Callable),
                Style::Rounded,
            ],
            SymbolKind::Interface => vec![
                Style::CssClass(CssClass::Interface),
                Style::Border(0),
                Style::Rounded,
            ],
            SymbolKind::Enum => vec![Style::CssClass(CssClass::Type)],
            SymbolKind::Struct => vec![Style::CssClass(CssClass::Type)],
            SymbolKind::Class => vec![Style::CssClass(CssClass::Type)],
            _ => vec![],
        }
    }

    // fn handle_unrecognized_functions(&self, funcs: Vec<&DocumentSymbol>);
}

pub struct DefaultLang;

impl Language for DefaultLang {}

pub(crate) fn language_handler(lang: &str) -> Box<dyn Language + Sync + Send> {
    match lang {
        "Go" => Box::new(Go),
        "Rust" => Box::new(Rust),
        _ => Box::new(DefaultLang {}),
    }
}
