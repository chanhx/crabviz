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
        let children = symbol
            .children
            .iter()
            .filter(|s| symbol.kind == SymbolKind::Interface || self.filter_symbol(s))
            .map(|symbol| self.symbol_repr(file_id, symbol))
            .collect();

        let range = symbol.selection_range;

        Cell {
            range_start: (range.start.line, range.start.character),
            range_end: (range.end.line, range.end.character),
            style: self.symbol_style(symbol),
            title: symbol.name.clone(),
            children,
        }
    }

    fn filter_symbol(&self, symbol: &DocumentSymbol) -> bool {
        match symbol.kind {
            SymbolKind::Constant
            | SymbolKind::Variable
            | SymbolKind::Field
            | SymbolKind::Property
            | SymbolKind::EnumMember => false,
            _ => true,
        }
    }

    fn symbol_style(&self, symbol: &DocumentSymbol) -> Style {
        match symbol.kind {
            SymbolKind::Module => Style {
                rounded: true,
                classes: CssClass::Cell | CssClass::Module,
                ..Default::default()
            },
            SymbolKind::Function => Style {
                rounded: true,
                classes: CssClass::Cell | CssClass::Function | CssClass::Clickable,
                ..Default::default()
            },
            SymbolKind::Method => Style {
                rounded: true,
                classes: CssClass::Cell | CssClass::Method | CssClass::Clickable,
                ..Default::default()
            },
            SymbolKind::Constructor => Style {
                rounded: true,
                classes: CssClass::Cell | CssClass::Constructor | CssClass::Clickable,
                ..Default::default()
            },
            SymbolKind::Interface => Style {
                border: Some(0),
                rounded: true,
                classes: CssClass::Cell | CssClass::Interface | CssClass::Clickable,
                ..Default::default()
            },
            SymbolKind::Enum => Style {
                icon: Some('E'),
                classes: CssClass::Cell | CssClass::Type,
                ..Default::default()
            },
            SymbolKind::Struct => Style {
                icon: Some('S'),
                classes: CssClass::Cell | CssClass::Type,
                ..Default::default()
            },
            SymbolKind::Class => Style {
                icon: Some('C'),
                classes: CssClass::Cell | CssClass::Type,
                ..Default::default()
            },
            SymbolKind::Property => Style {
                icon: Some('p'),
                classes: CssClass::Cell | CssClass::Property,
                ..Default::default()
            },
            _ => Style {
                rounded: true,
                classes: CssClass::Cell.into(),
                ..Default::default()
            },
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
