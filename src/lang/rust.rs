use {
    super::{Entry, Language},
    crate::{
        analysis::FileOutline,
        config::CONFIG,
        graph::{CellStyle, TableStyle},
    },
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
        let (func_groups, funcs): (Vec<_>, Vec<_>) = outline
            .symbols
            .iter()
            .filter(|symbol| match symbol.kind {
                SymbolKind::FUNCTION | SymbolKind::METHOD => true,
                SymbolKind::OBJECT | SymbolKind::INTERFACE if symbol.children.is_some() => true,
                _ => false,
            })
            .partition(|symbol| symbol.children.is_some());

        func_groups
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

    fn symbol_style(&self, symbol: &DocumentSymbol) -> Vec<CellStyle> {
        match symbol.kind {
            SymbolKind::FUNCTION | SymbolKind::METHOD => {
                vec![CellStyle::CssClass("fn".to_string()), CellStyle::Rounded]
            }
            SymbolKind::INTERFACE => {
                let table_style = vec![TableStyle::CssClass("interface".to_string())];
                vec![CellStyle::Table(table_style), CellStyle::Border(0)]
            }
            SymbolKind::OBJECT if symbol.name.starts_with("impl") => {
                let table_style = vec![
                    TableStyle::Border(0),
                    TableStyle::CssClass("rust-impl".to_string()),
                ];
                vec![CellStyle::Table(table_style), CellStyle::Border(0)]
            }
            SymbolKind::MODULE => {
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
