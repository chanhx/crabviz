use {
    crate::lsp_types::{CallHierarchyItem, DocumentSymbol, Position},
    std::{collections::HashMap, fmt::Display, hash::Hash, path::PathBuf},
};

pub(crate) struct FileOutline {
    pub id: u32,
    pub path: PathBuf,
    pub symbols: Vec<DocumentSymbol>,
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct SymbolLocation {
    pub path: String,
    pub line: u32,
    pub character: u32,
}

impl SymbolLocation {
    pub fn new(path: String, position: &Position) -> Self {
        Self {
            path,
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

pub(crate) trait LocationId {
    fn location_id(&self, files: &HashMap<String, FileOutline>) -> Option<(u32, u32, u32)>;
}

impl LocationId for SymbolLocation {
    fn location_id(&self, files: &HashMap<String, FileOutline>) -> Option<(u32, u32, u32)> {
        Some((files.get(&self.path)?.id, self.line, self.character))
    }
}

impl LocationId for CallHierarchyItem {
    fn location_id(&self, files: &HashMap<String, FileOutline>) -> Option<(u32, u32, u32)> {
        Some((
            files.get(&self.uri.path)?.id,
            self.selection_range.start.line,
            self.selection_range.start.character,
        ))
    }
}
