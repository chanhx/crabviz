mod types;

#[cfg(feature = "wasm")]
mod wasm;
#[cfg(feature = "wasm")]
pub use wasm::{set_panic_hook, GraphGeneratorWasm};

#[cfg(test)]
mod tests;

pub(crate) use types::*;
use {
    crate::{
        graph::{dot::Dot, Cell, Edge, EdgeStyle, Subgraph},
        lang,
        lsp_types::{
            CallHierarchyIncomingCall, CallHierarchyOutgoingCall, DocumentSymbol, Location,
            Position,
        },
    },
    std::{
        collections::{hash_map::Entry, BTreeMap, HashMap, HashSet},
        path::{Path, PathBuf},
    },
};

pub struct GraphGenerator {
    // TODO: use a trie map to store files
    root: String,
    files: HashMap<String, FileOutline>,
    next_file_id: u32,

    incoming_calls: HashMap<SymbolLocation, Vec<CallHierarchyIncomingCall>>,
    outgoing_calls: HashMap<SymbolLocation, Vec<CallHierarchyOutgoingCall>>,
    interfaces: HashMap<SymbolLocation, Vec<SymbolLocation>>,

    highlights: HashMap<u32, HashSet<(u32, u32)>>,
}

impl GraphGenerator {
    pub fn new(root: String) -> Self {
        Self {
            root,
            files: HashMap::new(),
            next_file_id: 1,
            incoming_calls: HashMap::new(),
            outgoing_calls: HashMap::new(),
            interfaces: HashMap::new(),
            highlights: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, file_path: String, symbols: Vec<DocumentSymbol>) {
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
                let entry_file = entry.get_mut();
                *entry_file = file;
            }
        }
    }

    // TODO: graph database
    pub fn add_incoming_calls(
        &mut self,
        file_path: String,
        position: Position,
        calls: Vec<CallHierarchyIncomingCall>,
    ) {
        let location = SymbolLocation::new(file_path, &position);
        self.incoming_calls.insert(location, calls);
    }

    pub fn add_outgoing_calls(
        &mut self,
        file_path: String,
        position: Position,
        calls: Vec<CallHierarchyOutgoingCall>,
    ) {
        let location = SymbolLocation::new(file_path, &position);
        self.outgoing_calls.insert(location, calls);
    }

    pub fn highlight(&mut self, file_path: String, position: Position) {
        let file_id = match self.files.get(&file_path) {
            None => return,
            Some(file) => file.id,
        };

        let cell_pos = (position.line, position.character);

        match self.highlights.entry(file_id) {
            Entry::Vacant(entry) => {
                let mut set = HashSet::new();
                set.insert(cell_pos);

                entry.insert(set);
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(cell_pos);
            }
        }
    }

    pub fn add_interface_implementations(
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

        // TODO: it's better to construct tables before fetching call hierarchy, so that we can skip the filtered out symbols.
        let tables = files
            .values()
            .map(|f| {
                let mut table = lang.file_repr(f);
                if let Some(cells) = self.highlights.get(&f.id) {
                    table.highlight_cells(cells);
                }

                table
            })
            .collect::<Vec<_>>();

        let incoming_calls = self
            .incoming_calls
            .iter()
            .filter(|(callee, _)| files.contains_key(&callee.path))
            .flat_map(|(callee, calls)| {
                let to = (
                    files.get(&callee.path).unwrap().id,
                    callee.line,
                    callee.character,
                );

                calls.into_iter().filter_map(move |call| {
                    Some(Edge {
                        from: (
                            files.get(call.from.uri.path())?.id,
                            call.from.selection_range.start.line,
                            call.from.selection_range.start.character,
                        ),
                        to,
                        styles: vec![],
                    })
                })
            });

        let outgoing_calls = self
            .outgoing_calls
            .iter()
            .filter(|(caller, _)| files.contains_key(&caller.path))
            .flat_map(|(caller, calls)| {
                let from = (
                    files.get(&caller.path).unwrap().id,
                    caller.line,
                    caller.character,
                );

                calls.into_iter().filter_map(move |call| {
                    Some(Edge {
                        from,
                        to: (
                            files.get(call.to.uri.path())?.id,
                            call.to.selection_range.start.line,
                            call.to.selection_range.start.character,
                        ),
                        styles: vec![],
                    })
                })
            });

        let implementations = self
            .interfaces
            .iter()
            .flat_map(|(interface, implementations)| {
                let to = (
                    files.get(&interface.path).unwrap().id,
                    interface.line,
                    interface.character,
                );

                implementations.into_iter().filter_map(move |location| {
                    Some(Edge {
                        from: (
                            files.get(&location.path)?.id,
                            location.line,
                            location.character,
                        ),
                        to,
                        styles: vec![EdgeStyle::CssClass("impl".to_string())],
                    })
                })
            });

        let mut cell_ids = HashSet::new();
        tables
            .iter()
            .flat_map(|tbl| tbl.sections.iter().map(|cell| (tbl.id, cell)))
            .for_each(|(tid, cell)| self.collect_cell_ids(tid, cell, &mut cell_ids));

        let edges = incoming_calls
            .chain(outgoing_calls)
            .chain(implementations)
            .filter(|edge| {
                // some cells may have been filtered out, so we need to check the `from_id`

                cell_ids.contains(&edge.from) && cell_ids.contains(&edge.to)
            })
            .collect::<HashSet<_>>();

        let subgraphs = self.subgraphs(files.iter().map(|(_, f)| f));

        Dot::generate_dot_source(&tables, edges.into_iter(), &subgraphs)
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

    fn collect_cell_ids(&self, table_id: u32, cell: &Cell, ids: &mut HashSet<(u32, u32, u32)>) {
        ids.insert((table_id, cell.range_start.0, cell.range_start.1));
        cell.children
            .iter()
            .for_each(|child| self.collect_cell_ids(table_id, child, ids));
    }
}
