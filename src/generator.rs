mod types;
mod wasm;

#[cfg(test)]
mod tests;

pub(crate) use types::*;
pub use wasm::GraphGeneratorWasm;
use {
    crate::{
        graph::{dot::Dot, Edge, EdgeStyle, Subgraph},
        lang,
        lsp_types::{CallHierarchyOutgoingCall, DocumentSymbol, Location, Position},
    },
    std::{
        collections::{hash_map::Entry, BTreeMap, HashMap, HashSet},
        path::{Path, PathBuf},
    },
};

struct GraphGenerator {
    // TODO: use a trie map to store files
    files: HashMap<String, FileOutline>,
    next_file_id: PathId,

    outgoing_calls: HashMap<SymbolLocation, Vec<CallHierarchyOutgoingCall>>,
    interfaces: HashMap<SymbolLocation, Vec<SymbolLocation>>,
}

impl GraphGenerator {
    fn new() -> Self {
        Self {
            files: HashMap::new(),
            next_file_id: 1,
            outgoing_calls: HashMap::new(),
            interfaces: HashMap::new(),
        }
    }

    fn add_file(&mut self, file_path: String, symbols: Vec<DocumentSymbol>) {
        let path = PathBuf::from(&file_path);
        let file = FileOutline {
            id: self.next_file_id,
            path,
            symbols,
        };

        match self.files.entry(file_path) {
            Entry::Vacant(entry) => {
                entry.insert(file);
                self.next_file_id += 1;
            }
            Entry::Occupied(mut entry) => {
                entry.insert(file);
            }
        }
    }

    // TODO: graph database
    fn add_outgoing_calls(
        &mut self,
        file_path: String,
        position: Position,
        calls: Vec<CallHierarchyOutgoingCall>,
    ) {
        let location = SymbolLocation::new(file_path, &position);
        self.outgoing_calls.insert(location, calls);
    }

    fn add_interface_implementations(
        &mut self,
        file_path: String,
        position: Position,
        locations: Vec<Location>,
    ) {
        let location = SymbolLocation::new(file_path, &position);
        let implementations = locations
            .into_iter()
            .map(|location| {
                SymbolLocation::new(location.uri.path().to_string(), &location.range.start)
            })
            .collect();
        self.interfaces.insert(location, implementations);
    }

    fn subgraphs<'a, I>(&'a self, files: I) -> Vec<Subgraph>
    where
        I: Iterator<Item = &'a FileOutline>,
    {
        let mut dirs = BTreeMap::new();
        for f in files {
            let parent = f.path.parent().unwrap();
            dirs.entry(parent)
                .or_insert(Vec::new())
                .push(f.path.clone());
        }

        fn subgraph_recursive(
            parent: &Path,
            dirs: &BTreeMap<&Path, Vec<PathBuf>>,
            map: &HashMap<String, FileOutline>,
        ) -> Vec<Subgraph> {
            dirs.iter()
                .filter(|(dir, _)| dir.parent().is_some_and(|p| p == parent))
                .map(|(dir, v)| Subgraph {
                    title: dir.file_name().unwrap().to_str().unwrap().into(),
                    nodes: v
                        .iter()
                        .map(|path| map.get(path.to_str().unwrap()).unwrap().id.to_string())
                        .collect::<Vec<_>>(),
                    subgraphs: subgraph_recursive(dir, dirs, map),
                })
                .collect()
        }

        subgraph_recursive(dirs.keys().next().unwrap(), &dirs, &self.files)
    }

    pub fn generate_dot_source(&self) -> String {
        let files = &self.files;

        let ext = files
            .iter()
            .next()
            .and_then(|(_, f)| f.path.extension().and_then(|ext| ext.to_str()))
            .unwrap_or("");
        let lang = lang::language_handler(ext);

        let tables = files
            .values()
            .map(|f| lang.file_repr(f))
            .collect::<Vec<_>>();

        let paths = files
            .iter()
            .map(|(_, outline)| &outline.path)
            .collect::<HashSet<_>>();

        let calls = self
            .outgoing_calls
            .iter()
            .flat_map(|(caller, callee)| {
                let from_table_id = files.get(&caller.path).unwrap().id.to_string();
                let from_node_id = format!("{}_{}", caller.line, caller.character);

                callee
                    .into_iter()
                    .filter(|callee| {
                        let path = callee.to.uri.path();
                        let path = PathBuf::from(path);

                        paths.contains(&path)
                    })
                    .map(|call| Edge {
                        from_table_id: from_table_id.clone(),
                        from_node_id: from_node_id.clone(),
                        to_table_id: files.get(call.to.uri.path()).unwrap().id.to_string(),
                        to_node_id: format!(
                            "{}_{}",
                            call.to.selection_range.start.line,
                            call.to.selection_range.start.character
                        ),
                        styles: vec![],
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let implementations = self
            .interfaces
            .iter()
            .flat_map(|(interface, implementations)| {
                let to_table_id = files.get(&interface.path).unwrap().id.to_string();
                let to_node_id = format!("{}_{}", interface.line, interface.character);

                implementations
                    .into_iter()
                    .filter(|location| {
                        let path = &location.path;
                        let path = PathBuf::from(path);

                        paths.contains(&path)
                    })
                    .map(|location| Edge {
                        from_table_id: files.get(&location.path).unwrap().id.to_string(),
                        from_node_id: format!("{}_{}", location.line, location.character),
                        to_table_id: to_table_id.clone(),
                        to_node_id: to_node_id.clone(),
                        styles: vec![EdgeStyle::CssClass("impl".to_string())],
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let edges = [calls, implementations].concat();

        let subgraphs = self.subgraphs(files.iter().map(|(_, f)| f));

        Dot::generate_dot_source(&tables, &edges, &subgraphs)
    }
}
