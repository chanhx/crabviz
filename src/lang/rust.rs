use {
    super::{Entry, Language},
    crate::{analysis::FileOutline, config::CONFIG},
    lsp_types::{DocumentSymbol, SymbolKind},
    std::{
        path::Path,
        process::{Child, Command, Stdio},
    },
};

pub(crate) struct Rust {}

impl Language for Rust {
    fn start_language_server(&self) -> Child {
        let server_path = CONFIG
            .servers
            .get("rust")
            .expect("The path to rust language server is not set");

        let server_path = shellexpand::full(server_path)
            .map(|path| std::path::Path::new(path.as_ref()).canonicalize().unwrap())
            .unwrap();

        Command::new(server_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to start the language server")
    }

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
