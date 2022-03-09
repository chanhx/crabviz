mod file_structure;
mod ra;

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use ra_ide::{FileId, StructureNode, StructureNodeKind, SymbolKind};

pub(crate) use file_structure::{File, FilePosition, RItem, RItemType};

pub(crate) struct Analyzer(ra::Analyzer);

impl Analyzer {
    pub fn new(path: &Path) -> Self {
        Analyzer(ra::Analyzer::new(path))
    }

    pub fn files_id(&self, path: String) -> HashSet<FileId> {
        self.0.files_id(path)
    }

    pub fn file_structure(&self, file_id: FileId) -> File {
        let nodes = self.0.file_structure(file_id);
        let items = file_ritems(file_id, nodes);

        File {
            path: self.0.file_path(file_id),
            file_id: file_id.0,
            items,
        }
    }

    pub fn file_references(&self, file: &File) -> HashMap<FilePosition, Vec<FilePosition>> {
        let mut items = file.items.iter().collect::<Vec<_>>();
        {
            // collect all items in file by breadth-first traversing

            let mut s = 0;
            loop {
                let mut new_items = items
                    .iter()
                    .skip(s)
                    .filter_map(|item| item.children.as_ref())
                    .flat_map(|children| children)
                    .collect::<Vec<_>>();

                if new_items.len() <= 0 {
                    break;
                }

                s = items.len();
                items.append(&mut new_items);
            }
        }

        items
            .iter()
            .filter(|item| matches!(item.ty, RItemType::Func))
            .map(|item| {
                let pos = ra_ide::FilePosition {
                    file_id: FileId(item.pos.file_id),
                    offset: item.pos.offset.into(),
                };
                let refs = self
                    .0
                    .find_references(pos)
                    .iter()
                    .filter_map(|item| match item.target.focus_range {
                        Some(ref_pos) => Some(FilePosition {
                            file_id: item.target.file_id.0,
                            offset: u32::from(ref_pos.start()),
                        }),
                        None => None,
                    })
                    .collect();

                (item.pos, refs)
            })
            .collect()
    }
}

struct WrappedNode {
    node: StructureNode,
    children: Option<Vec<WrappedNode>>,
}

fn file_ritems(file_id: FileId, nodes: Vec<StructureNode>) -> Vec<RItem> {
    let mut parents = nodes
        .into_iter()
        .map(|node| WrappedNode {
            node,
            children: None,
        })
        .collect::<Vec<_>>();

    let mut nodes = Vec::new();
    while let Some(mut w) = parents.pop() {
        if let Some(children) = &mut w.children {
            children.reverse();
        }
        let parent = match w.node.parent {
            None => &mut nodes,
            Some(i) => parents[i].children.get_or_insert_with(Vec::new),
        };
        parent.push(w);
    }

    nodes
        .into_iter()
        .filter_map(|w| node_to_ritem(file_id, w))
        .rev()
        .collect::<Vec<_>>()
}

fn node_to_ritem(file_id: ra_ide::FileId, wrapped_node: WrappedNode) -> Option<RItem> {
    let node = wrapped_node.node;
    let children = wrapped_node.children;

    match node.kind {
        StructureNodeKind::SymbolKind(kind) => match kind {
            SymbolKind::Struct => Some(RItemType::Struct),
            SymbolKind::Function => Some(RItemType::Func),
            SymbolKind::Enum => Some(RItemType::Enum),
            SymbolKind::Union => Some(RItemType::Union),
            SymbolKind::Trait => Some(RItemType::Trait),
            SymbolKind::Macro => Some(RItemType::Macro),
            SymbolKind::Impl => Some(RItemType::Impl),
            // SymbolKind::Module => ...
            _ => None,
        },
        _ => None,
    }
    .and_then(|ty| {
        let pos = FilePosition {
            file_id: file_id.0,
            offset: node.navigation_range.start().into(),
        };

        let children = children.and_then(|children| {
            let children = children
                .into_iter()
                .filter_map(|node| node_to_ritem(file_id, node))
                .collect::<Vec<_>>();

            if children.len() > 0 {
                Some(children)
            } else {
                None
            }
        });

        Some(RItem {
            ident: node.label,
            ty,
            pos,
            children,
        })
    })
}
