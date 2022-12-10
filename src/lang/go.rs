use {
    super::{Entry, Language},
    crate::graph::{CellStyle, TableStyle},
    lsp_types::{DocumentSymbol, SymbolKind},
    std::{
        path::Path,
        process::{Child, Command, Stdio},
    },
};

pub(crate) struct Go {}

impl Language for Go {
    fn start_language_server(&self) -> Child {
        Command::new("gopls")
            .arg("serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to start the language server")
    }

    fn entry(&self, base: &Path) -> Entry {
        Entry::new(base, vec!["go".to_string()], &[".git"])
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
            _ => vec![],
        }
    }
}
