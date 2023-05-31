mod types;
mod wasm;

#[cfg(test)]
mod tests;

pub(crate) use types::*;
pub use wasm::GraphGeneratorWasm;
use {
    crate::{
        graph::{dot::Dot, Cell, Edge, EdgeStyle, Subgraph},
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
    root: String,
    files: HashMap<String, FileOutline>,
    next_file_id: PathId,

    outgoing_calls: HashMap<SymbolLocation, Vec<CallHierarchyOutgoingCall>>,
    interfaces: HashMap<SymbolLocation, Vec<SymbolLocation>>,
}

impl GraphGenerator {
    fn new(root: String) -> Self {
        Self {
            root,
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

        let calls = self.outgoing_calls.iter().flat_map(|(caller, callee)| {
            let from_table_id = files.get(&caller.path).unwrap().id.to_string();
            let from_node_id = format!("{}_{}", caller.line, caller.character);

            callee.into_iter().filter_map(move |call| {
                let to_table_id = files.get(call.to.uri.path())?.id.to_string();
                Some(Edge {
                    from_table_id: from_table_id.clone(),
                    from_node_id: from_node_id.clone(),
                    to_table_id,
                    to_node_id: format!(
                        "{}_{}",
                        call.to.selection_range.start.line, call.to.selection_range.start.character
                    ),
                    styles: vec![],
                })
            })
        });

        let implementations = self
            .interfaces
            .iter()
            .flat_map(|(interface, implementations)| {
                let to_table_id = files.get(&interface.path).unwrap().id.to_string();
                let to_node_id = format!("{}_{}", interface.line, interface.character);

                implementations.into_iter().filter_map(move |location| {
                    let from_table_id = files.get(&location.path)?.id.to_string();
                    Some(Edge {
                        from_table_id,
                        from_node_id: format!("{}_{}", location.line, location.character),
                        to_table_id: to_table_id.clone(),
                        to_node_id: to_node_id.clone(),
                        styles: vec![EdgeStyle::CssClass("impl".to_string())],
                    })
                })
            });

        let mut cell_ids = HashSet::new();
        tables
            .iter()
            .flat_map(|tbl| tbl.sections.iter())
            .for_each(|cell| self.collect_cell_ids(cell, &mut cell_ids));

        let mut edges = HashMap::new();
        calls
            .chain(implementations)
            .filter(|edge| {
                let id = format!("{}:{}", edge.to_table_id, edge.to_node_id);
                cell_ids.contains(&id)
            })
            .for_each(|edge| {
                let key = format!(
                    "{}:{}-{}:{}",
                    edge.from_table_id, edge.from_node_id, edge.to_table_id, edge.to_node_id
                );
                edges.entry(key).or_insert(edge);
            });

        let subgraphs = self.subgraphs(files.iter().map(|(_, f)| f));

        Dot::generate_dot_source(&tables, edges.into_values(), &subgraphs)
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

        let mut subgraphs: Vec<Subgraph> = vec![];

        dirs.iter().for_each(|(dir, files)| {
            let nodes = files
                .iter()
                .map(|path| {
                    self.files
                        .get(path.to_str().unwrap())
                        .unwrap()
                        .id
                        .to_string()
                })
                .collect::<Vec<_>>();

            let dir = dir.strip_prefix(&self.root).unwrap_or(dir);
            self.add_subgraph(dir, nodes, &mut subgraphs);
        });

        subgraphs
    }

    fn add_subgraph<'a, 'b, 'c>(
        &'a self,
        dir: &'b Path,
        nodes: Vec<String>,
        subgraphs: &'c mut Vec<Subgraph>,
    ) {
        let ancestor = subgraphs.iter_mut().find(|g| dir.starts_with(&g.title));

        match ancestor {
            None => subgraphs.push(Subgraph {
                title: dir.to_str().unwrap().into(),
                nodes,
                subgraphs: vec![],
            }),
            Some(ancestor) => {
                let dir = dir.strip_prefix(&ancestor.title).unwrap();
                self.add_subgraph(dir, nodes, &mut ancestor.subgraphs);
            }
        }
    }

    fn collect_cell_ids(&self, cell: &Cell, ids: &mut HashSet<String>) {
        ids.insert(cell.id.clone());
        cell.children
            .iter()
            .for_each(|c| self.collect_cell_ids(c, ids));
    }
}
