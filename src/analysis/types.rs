use {
    lsp_types::{DocumentSymbol, Position, Url},
    std::{collections::HashMap, fmt::Display, hash::Hash, path::PathBuf},
};

pub(crate) struct FileOutline {
    pub path: PathBuf,
    pub symbols: Vec<DocumentSymbol>,
}

pub type Relations = HashMap<SymbolLocation, Vec<(SymbolLocation, Option<String>)>>;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct SymbolLocation {
    pub path: String,
    pub line: u32,
    pub character: u32,
}

impl SymbolLocation {
    pub fn new(uri: &Url, position: &Position) -> Self {
        Self {
            path: uri.path().to_string().trim_end_matches('/').to_string(),
            line: position.line,
            character: position.character,
        }
    }
}

impl Display for SymbolLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#""{}":"{}_{}""#, self.path, self.line, self.character)
    }
}
