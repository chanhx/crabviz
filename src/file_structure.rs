use std::{fmt, hash::Hash, path::PathBuf};

pub struct RItem {
    pub ident: String,
    pub ty: RItemType,
    pub pos: FilePosition,
    pub children: Option<Vec<RItem>>,
    // pub visiblity: Visibility
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FilePosition {
    pub file_id: u32,
    pub offset: u32,
}

impl fmt::Display for FilePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.file_id, self.offset)
    }
}

// pub enum Visibility {
//     Private,
//     VisibleToCrate,
//     VisibleToPath,
//     VisibleToParent,
//     Public,
// }

// pub struct FuncInfo {
//     pub name: String,
//     pub signature: String,
//     pub is_unsafe: bool,
//     pub is_async: bool,
// }

pub enum RItemType {
    Macro,
    // TypeAlias { original_type: String },
    Enum,
    Struct,
    Union,
    Trait,
    Impl,
    Func,
}

pub struct File {
    pub path: PathBuf,
    pub file_id: u32,
    pub items: Vec<RItem>,
}

impl File {
    pub fn new(path: PathBuf, file_id: u32) -> Self {
        File {
            path,
            file_id,
            items: Vec::new(),
        }
    }
}
