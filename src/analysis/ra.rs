use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::Result;

use ra_ide::{Analysis, AnalysisHost, CallItem, FileId, FilePosition, StructureNode};
use ra_ide_db::{base_db::SourceDatabaseExt, symbol_index::SymbolsDatabase};
use ra_project_model::CargoConfig;
use ra_rust_analyzer::cli::load_cargo::{load_workspace_at, LoadCargoConfig};
use ra_vfs::{Vfs, VfsPath};

pub(crate) struct Analyzer {
    host: AnalysisHost,
    analysis: Analysis,
    vfs: Vfs,
}

impl Analyzer {
    pub(crate) fn new(path: &Path) -> Self {
        let (host, analysis, vfs) = Analyzer::load_workspace(&path).unwrap();
        Analyzer {
            host,
            analysis,
            vfs,
        }
    }

    fn load_workspace(path: &Path) -> Result<(AnalysisHost, Analysis, Vfs)> {
        let load_cargo_config = LoadCargoConfig {
            load_out_dirs_from_check: true,
            with_proc_macro: false,
            prefill_caches: false,
        };
        let cargo_config = CargoConfig::default();

        let (host, vfs, _proc_macro) =
            load_workspace_at(path, &cargo_config, &load_cargo_config, &|_| ())?;

        let analysis = host.analysis();

        Ok((host, analysis, vfs))
    }

    pub(super) fn files_id(&self, path: String) -> HashSet<FileId> {
        let db = self.host.raw_database();
        let vfs_path = VfsPath::new_real_path(path);

        db.local_roots()
            .iter()
            .flat_map(|&root_id| db.source_root(root_id).iter().collect::<Vec<_>>())
            .filter(|&file_id| self.vfs.file_path(file_id).starts_with(&vfs_path))
            .collect()
    }

    pub(super) fn file_path(&self, file_id: FileId) -> PathBuf {
        self.vfs.file_path(file_id).to_string().into()
    }

    pub(super) fn file_structure(&self, file_id: FileId) -> Vec<StructureNode> {
        self.analysis.file_structure(file_id).unwrap()
    }

    pub(super) fn find_references(&self, pos: FilePosition) -> Vec<CallItem> {
        self.analysis
            .incoming_calls(pos)
            .unwrap()
            .unwrap_or(Vec::new())
    }
}
