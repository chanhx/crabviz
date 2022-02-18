use std::path::PathBuf;

pub struct RItem {
    pub ident: String,
    pub ty: RItemType,
    pub pos: FilePosition,
    pub children: Option<Vec<RItem>>,
    // pub visiblity: Visibility
}

impl RItem {
    pub fn id(&self) -> String {
        format!("{}_{}", self.pos.file_id, self.pos.offset)
    }
}

pub struct FilePosition {
    pub file_id: u32,
    pub offset: u32,
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

pub struct Module {
    pub path: PathBuf,
    pub file_id: u32,
    pub items: Vec<RItem>,
}

impl Module {
    pub fn new(path: PathBuf, file_id: u32) -> Self {
        Module {
            path,
            file_id,
            items: Vec::new(),
        }
    }
}
