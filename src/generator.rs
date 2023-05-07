mod types;

pub(crate) use types::*;
use {
    crate::{
        graph::{dot::Dot, Edge, EdgeStyle, Subgraph},
        lang,
        lsp_types::{CallHierarchyOutgoingCall, DocumentSymbol, LocationLink, Position},
    },
    std::{
        cell::RefCell,
        collections::{hash_map::Entry, BTreeMap, HashMap, HashSet},
        path::{Path, PathBuf},
    },
    wasm_bindgen::prelude::*,
};

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: String);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[wasm_bindgen]
pub struct GraphGenerator {
    inner: RefCell<GraphGeneratorInner>,
}

struct GraphGeneratorInner {
    // TODO: use a trie map to store files
    files: HashMap<String, FileOutline>,
    next_file_id: PathId,

    outgoing_calls: HashMap<SymbolLocation, Vec<CallHierarchyOutgoingCall>>,
    interfaces: HashMap<SymbolLocation, Vec<SymbolLocation>>,
}

#[wasm_bindgen]
impl GraphGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(GraphGeneratorInner {
                files: HashMap::new(),
                next_file_id: 1,
                outgoing_calls: HashMap::new(),
                interfaces: HashMap::new(),
            }),
        }
    }

    #[wasm_bindgen(js_name = add_file)]
    pub fn add_file_wasm(&self, file_path: String, symbols: JsValue) {
        let symbols = serde_wasm_bindgen::from_value::<Vec<DocumentSymbol>>(symbols).unwrap();

        self.inner.borrow_mut().add_file(file_path, symbols);
    }

    #[wasm_bindgen(js_name = add_outgoing_calls)]
    pub fn add_outgoing_calls_wasm(&self, file_path: String, position: JsValue, calls: JsValue) {
        let position = serde_wasm_bindgen::from_value::<Position>(position).unwrap();
        let calls =
            serde_wasm_bindgen::from_value::<Vec<CallHierarchyOutgoingCall>>(calls).unwrap();

        self.inner
            .borrow_mut()
            .add_outgoing_calls(file_path, position, calls);
    }

    #[wasm_bindgen(js_name = add_interface_implementations)]
    pub fn add_interface_implementations_wasm(
        &self,
        file_path: String,
        position: JsValue,
        links: JsValue,
    ) {
        let position = serde_wasm_bindgen::from_value::<Position>(position).unwrap();
        let links = serde_wasm_bindgen::from_value::<Vec<LocationLink>>(links).unwrap();

        self.inner
            .borrow_mut()
            .add_interface_implementations(file_path, position, links);
    }

    pub fn generate_dot_source(&self) -> String {
        let lang = lang::language_handler("rust");

        let generator = self.inner.borrow();
        let files = &generator.files;

        let tables = files
            .values()
            .map(|f| lang.file_repr(f))
            .collect::<Vec<_>>();

        let paths = files
            .iter()
            .map(|(_, outline)| &outline.path)
            .collect::<HashSet<_>>();
        // let edges = vec![];

        let calls = generator
            .outgoing_calls
            .iter()
            .map(|(caller, callee)| {
                (
                    caller.clone(),
                    callee
                        .into_iter()
                        .filter(|callee| {
                            let path = callee.to.uri.path();
                            let path = PathBuf::from(path);

                            paths.contains(&path)
                        })
                        .map(|call| {
                            SymbolLocation::new(
                                call.to.uri.path().to_string(),
                                &call.to.selection_range.start,
                            )
                        })
                        .collect(),
                )
            })
            .collect::<Relations>();

        let implementations = generator
            .interfaces
            .iter()
            .map(|(interface, implementations)| {
                (
                    interface.clone(),
                    implementations
                        .iter()
                        .filter(|location| {
                            let path = &location.path;
                            let path = PathBuf::from(path);

                            paths.contains(&path)
                        })
                        .map(|location| location.clone())
                        .collect(),
                )
            })
            .collect::<Relations>();

        let calling_edges = calls
            .iter()
            .flat_map(|rels| {
                let from_table_id = files.get(&rels.0.path.clone()).unwrap().id;
                let from_node_id = format!("{}_{}", rels.0.line, rels.0.character);

                rels.1
                    .iter()
                    .map(|location| Edge {
                        from_table_id: from_table_id.to_string(),
                        from_node_id: from_node_id.clone(),
                        to_table_id: files.get(&location.path.clone()).unwrap().id.to_string(),
                        to_node_id: format!("{}_{}", location.line, location.character),
                        styles: vec![],
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let impl_edges = implementations
            .iter()
            .flat_map(|rels| {
                let to_table_id = files.get(&rels.0.path.clone()).unwrap().id.to_string();
                let to_node_id = format!("{}_{}", rels.0.line, rels.0.character);

                rels.1
                    .iter()
                    .map(|location| Edge {
                        from_table_id: files.get(&location.path.clone()).unwrap().id.to_string(),
                        from_node_id: format!("{}_{}", location.line, location.character),
                        to_table_id: to_table_id.clone(),
                        to_node_id: to_node_id.clone(),
                        styles: vec![EdgeStyle::CssClass("impl".to_string())],
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let edges = [calling_edges, impl_edges].concat();

        let subgraphs = generator.subgraphs(files.iter().map(|(_, f)| f));

        Dot::generate_dot_source(&tables, &edges, &subgraphs)
    }
}

impl GraphGeneratorInner {
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
        links: Vec<LocationLink>,
    ) {
        let location = SymbolLocation::new(file_path, &position);
        let implementations = links
            .into_iter()
            .map(|link| {
                SymbolLocation::new(
                    link.target_uri.path().to_string(),
                    &link.target_selection_range.start,
                )
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
                .filter(|(dir, _)| dir.parent().unwrap() == parent)
                .map(|(dir, v)| Subgraph {
                    title: dir.file_name().unwrap().to_str().unwrap().into(),
                    nodes: v
                        .iter()
                        .map(|path| map.get(path.to_str().unwrap()).unwrap().id.to_string())
                        .collect::<Vec<_>>(),
                    subgraphs: subgraph_recursive(dir, dirs, map),
                })
                .collect::<Vec<_>>()
        }

        subgraph_recursive(dirs.keys().next().unwrap(), &dirs, &self.files)
    }
}

pub struct PathMap {
    // analysis_root: PathBuf,
    // source: HashMap<PathBuf, u32>,
    // dependencies: HashMap<PathBuf, u32>,
    map: HashMap<String, PathId>,
    next_id: PathId,
}

impl PathMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            next_id: 1,
        }
    }

    fn insert(&mut self, path: String) -> PathId {
        match self.map.try_insert(path, self.next_id) {
            Ok(id) => {
                self.next_id += 1;
                id.to_owned()
            }
            Err(e) => e.value,
        }
    }

    pub fn get(&self, path: &str) -> Option<u32> {
        self.map.get(path).copied()
    }
}
