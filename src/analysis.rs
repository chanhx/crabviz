use anyhow::Result;

use ra_ide::{Analysis, AnalysisHost, FileId, StructureNode, StructureNodeKind, SymbolKind};
use ra_ide_db::{base_db::SourceDatabaseExt, symbol_index::SymbolsDatabase};
use ra_project_model::CargoConfig;
use ra_rust_analyzer::cli::load_cargo::{load_workspace_at, LoadCargoConfig};
use ra_vfs::Vfs;

use std::path::{Path, PathBuf};

use crate::file_structure::{File, FilePosition, RItem, RItemType};

pub(crate) struct Analyzer {
    host: AnalysisHost,
    analysis: Analysis,
    vfs: Vfs,
}

impl Analyzer {
    pub(crate) fn new(root: &Path) -> Self {
        let (host, analysis, vfs) = Analyzer::load_workspace(root).unwrap();
        Analyzer {
            host,
            analysis,
            vfs,
        }
    }

    fn load_workspace(root: &Path) -> Result<(AnalysisHost, Analysis, Vfs)> {
        let load_cargo_config = LoadCargoConfig {
            load_out_dirs_from_check: true,
            with_proc_macro: false,
            prefill_caches: false,
        };
        let cargo_config = CargoConfig::default();

        let (host, vfs, _proc_macro) =
            load_workspace_at(root, &cargo_config, &load_cargo_config, &|_| ())?;

        // let db = host.raw_database();
        let analysis = host.analysis();

        Ok((host, analysis, vfs))
    }

    pub(crate) fn analyze(&self, root: &Path) -> Result<Vec<File>> {
        let db = self.host.raw_database();

        let mut file_ids = Vec::new();
        for r in db.local_roots().iter() {
            for file_id in db.source_root(*r).iter() {
                let path = self.vfs.file_path(file_id).to_string();

                let path = PathBuf::from(path);
                if !path.as_path().starts_with(root) {
                    break;
                }

                file_ids.push((path, file_id));
            }
        }

        let files = file_ids
            .into_iter()
            .map(|(path, file_id)| {
                let mut file = File::new(path, file_id.0);
                file.items = self.file_ritems(file_id).unwrap();

                file
            })
            .collect();

        Ok(files)
    }

    fn file_ritems(&self, file_id: FileId) -> Result<Vec<RItem>> {
        struct WrappedNode {
            node: StructureNode,
            children: Option<Vec<WrappedNode>>,
        }

        let mut parents = self
            .analysis
            .file_structure(file_id)?
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

        let ritems = nodes
            .into_iter()
            .filter_map(|w| {
                structure_node_to_ritem(
                    w.node,
                    w.children
                        .and_then(|children| Some(children.into_iter().map(|c| c.node).collect())),
                    file_id,
                )
            })
            .rev()
            .collect::<Vec<_>>();

        Ok(ritems)
    }

    pub(crate) fn find_callers(&self, fn_pos: &FilePosition) -> Result<Vec<FilePosition>> {
        let position = ra_ide::FilePosition {
            file_id: ra_ide::FileId(fn_pos.file_id),
            offset: ra_ide::TextSize::from(fn_pos.offset),
        };

        let callers_pos = self
            .host
            .analysis()
            .incoming_calls(position)?
            .unwrap_or(Vec::new())
            .iter()
            .filter_map(|item| match item.target.focus_range {
                Some(caller_pos) => Some(FilePosition {
                    file_id: item.target.file_id.0,
                    offset: u32::from(caller_pos.start()),
                }),
                None => None,
            })
            .collect();

        Ok(callers_pos)
    }
}

fn structure_node_to_ritem(
    node: StructureNode,
    children: Option<Vec<StructureNode>>,
    file_id: ra_ide::FileId,
) -> Option<RItem> {
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
            offset: u32::from(node.navigation_range.start()),
        };

        let children = children.and_then(|children| {
            let children = children
                .into_iter()
                .filter_map(|node| structure_node_to_ritem(node, None, file_id))
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
