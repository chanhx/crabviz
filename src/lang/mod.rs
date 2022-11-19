mod rust;

use {
    crate::app::FileOutline,
    lsp_types::{DocumentSymbol, SymbolKind},
    std::{
        path::{Path, PathBuf},
        process::Child,
    },
};

pub(crate) use rust::Rust;

pub(crate) trait Language {
    fn start_language_server(&self) -> Child;
    fn entry(&self, base: &Path) -> Entry;

    fn all_functions<'a, 'b>(&'a self, outline: &'b FileOutline) -> Vec<&'b DocumentSymbol> {
        outline
            .symbols
            .iter()
            .filter(|symbol| match symbol.kind {
                SymbolKind::FUNCTION | SymbolKind::METHOD => true,
                _ => false,
            })
            .collect()
    }

    fn all_interfaces<'a, 'b>(&'a self, outline: &'b FileOutline) -> Vec<&'b DocumentSymbol> {
        outline
            .symbols
            .iter()
            .filter(|symbol| symbol.kind == SymbolKind::INTERFACE)
            .collect()
    }

    // fn handle_unrecognized_functions(&self, funcs: Vec<&DocumentSymbol>);
}

pub(crate) fn language_handler(lang: &str) -> Box<dyn Language> {
    Box::new(match lang {
        "rust" => Rust {},
        _ => unimplemented!(),
    })
}

pub struct Entry {
    pub extensions: Vec<String>,
    pub include: Vec<PathBuf>,
    pub exclude: Vec<PathBuf>,
}

impl Entry {
    fn new(base: &Path, extensions: Vec<String>, exclude: &[&str]) -> Self {
        let base = base.to_path_buf();
        let exclude = exclude.iter().map(|it| base.join(it)).collect();

        Self {
            extensions,
            include: vec![base],
            exclude,
        }
    }
}
