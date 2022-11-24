mod types;

pub(crate) use types::*;

use {
    super::analysis::{FileOutline, Relations},
    lsp_types::{DocumentSymbol, SymbolKind},
    std::{
        collections::{BTreeMap, HashMap},
        path::{Path, PathBuf},
    },
};

pub trait GenerateSVG {
    fn gen_svg(
        &self,
        tables: &Vec<TableNode>,
        // nodes: Vec<Node>,
        edges: &Vec<Edge>,
        subgraphs: &Vec<Subgraph>,
    ) -> String;
}

fn symbol_to_node(symbol: &DocumentSymbol, path: &Path, map: &PathMap) -> Node {
    let classes = match symbol.kind {
        SymbolKind::FUNCTION | SymbolKind::METHOD => Some(vec![".fn".into()]),
        _ => None,
    };

    let children = symbol.children.as_ref().map(|children| {
        children
            .iter()
            .map(|item| symbol_to_node(item, path, map))
            .collect()
    });

    let port = format!(
        "{}_{}",
        symbol.selection_range.start.line, symbol.selection_range.start.character
    )
    .to_string();

    Node {
        port: port.clone(),
        id: format!("{}:{}", map.get(path).unwrap().to_string(), port),
        classes,
        title: symbol.name.clone(),
        children,
    }
}

fn file_to_table_node(node: &FileOutline, map: &PathMap) -> TableNode {
    let sections = node
        .symbols
        .iter()
        .map(|symbol| symbol_to_node(symbol, &node.path, map))
        .collect();

    TableNode {
        id: map.get(&node.path).unwrap().to_string(),
        title: node.path.file_name().unwrap().to_str().unwrap().to_string(),
        sections,
    }
}

fn subgraphs(files: &Vec<FileOutline>, map: &PathMap) -> Vec<Subgraph> {
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
        map: &PathMap,
    ) -> Vec<Subgraph> {
        dirs.iter()
            .filter(|(dir, _)| dir.parent().unwrap() == parent)
            .map(|(dir, v)| Subgraph {
                title: dir.file_name().unwrap().to_str().unwrap().into(),
                nodes: v
                    .iter()
                    .map(|path| map.get(&path).unwrap().to_string())
                    .collect::<Vec<_>>(),
                subgraphs: Box::new(subgraph_recursive(dir, dirs, map)),
            })
            .collect::<Vec<_>>()
    }

    subgraph_recursive(dirs.keys().next().unwrap(), &dirs, map)
}

pub(super) fn gen_svg<T: GenerateSVG>(
    generator: &T,
    files: &Vec<FileOutline>,
    relations: Relations,
) -> String {
    let mut map = PathMap::new();
    files.iter().for_each(|f| map.insert(f.path.clone()));

    let tables = files.iter().map(|f| file_to_table_node(f, &map)).collect();
    let edges = relations
        .iter()
        .flat_map(|rels| {
            let from_table_id = map
                .get(&PathBuf::from(rels.0.path.clone()))
                .unwrap()
                .to_string();
            let from_node_id = format!("{}_{}", rels.0.line, rels.0.character);

            rels.1
                .iter()
                .map(|(location, opt)| Edge {
                    from_table_id: from_table_id.clone(),
                    from_node_id: from_node_id.clone(),
                    to_table_id: map
                        .get(&PathBuf::from(location.path.clone()))
                        .unwrap()
                        .to_string(),
                    to_node_id: format!("{}_{}", location.line, location.character),
                    style: opt.clone(),
                })
                .collect::<Vec<_>>()
        })
        .collect();

    generator.gen_svg(&tables, &edges, &subgraphs(files, &map))
}

struct PathMap {
    // analysis_root: PathBuf,
    // source: HashMap<PathBuf, u32>,
    // dependencies: HashMap<PathBuf, u32>,
    map: HashMap<PathBuf, u32>,
    next_id: u32,
}

impl PathMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            next_id: 1,
        }
    }

    fn insert(&mut self, path: PathBuf) {
        self.map.insert(path, self.next_id);
        self.next_id += 1;
    }

    fn get(&self, path: &Path) -> Option<u32> {
        self.map.get(path).copied()
    }
}
