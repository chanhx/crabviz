use {
    super::{Entry, Language},
    crate::app::FileOutline,
    lsp_types::{DocumentSymbol, SymbolKind},
    std::path::Path,
};

pub(crate) struct Rust {}

impl Language for Rust {
    fn entry(&self, base: &Path) -> Entry {
        Entry::new(base, vec!["rs".to_string()], &[".git", "target"])
    }

    fn all_functions<'a, 'b>(&'a self, outline: &'b FileOutline) -> Vec<&'b DocumentSymbol> {
        let (impls, funcs): (Vec<_>, Vec<_>) = outline
            .symbols
            .iter()
            .filter(|symbol| match symbol.kind {
                SymbolKind::FUNCTION | SymbolKind::METHOD => true,
                SymbolKind::OBJECT if symbol.children.is_some() => true,
                _ => false,
            })
            .partition(|symbol| symbol.kind == SymbolKind::OBJECT);

        impls
            .iter()
            .flat_map(|impl_block| {
                impl_block
                    .children
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter(|symbol| match symbol.kind {
                        SymbolKind::FUNCTION | SymbolKind::METHOD => true,
                        _ => false,
                    })
            })
            .chain(funcs)
            .collect()
    }
}
