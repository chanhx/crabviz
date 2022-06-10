use {
    crate::{
        analysis::{File, FilePosition, RItem, RItemType},
        app::Path,
    },
    std::collections::{BTreeMap, HashMap},
};

pub struct Reference {
    pub dest: FilePosition,
    pub style: Option<String>,
}
pub type References = HashMap<FilePosition, Vec<Reference>>;

pub struct Node {
    pub id: String,
    pub port: u32,
    pub title: String,
    pub classes: Option<Vec<String>>,
    pub children: Option<Box<Vec<Node>>>,
}

pub struct TableNode {
    pub id: String,
    pub title: String,
    pub sections: Vec<Node>,
}

pub struct Subgraph {
    pub title: String,
    pub nodes: Vec<String>,
    pub subgraphs: Box<Vec<Subgraph>>,
}

pub trait GenerateSVG {
    fn gen_svg(
        &self,
        tables: &Vec<TableNode>,
        // nodes: Vec<Node>,
        refs: &References,
        subgraphs: &Vec<Subgraph>,
    ) -> String;
}

fn ritem_to_node(item: &RItem) -> Node {
    let classes = match item.ty {
        RItemType::Func => Some(vec![".fn".into()]),
        _ => None,
    };

    let children = item
        .children
        .as_ref()
        .map(|children| Box::new(children.iter().map(|item| ritem_to_node(item)).collect()));

    Node {
        port: item.pos.offset,
        id: item.pos.to_string(),
        classes,
        title: item.ident.clone(),
        children,
    }
}

fn file_to_table_node(file: &File) -> TableNode {
    let sections = file.items.iter().map(|item| ritem_to_node(item)).collect();

    TableNode {
        id: file.file_id.to_string(),
        title: file.path.file_name().unwrap().to_str().unwrap().to_string(),
        sections,
    }
}

fn subgraphs(files: &Vec<File>) -> Vec<Subgraph> {
    let mut dirs = BTreeMap::new();
    for f in files {
        let parent = f.path.parent().unwrap();
        dirs.entry(parent).or_insert(Vec::new()).push(f.file_id);
    }

    fn subgraph_recursive(parent: &Path, dirs: &BTreeMap<&Path, Vec<u32>>) -> Vec<Subgraph> {
        dirs.iter()
            .filter(|(dir, _)| dir.parent().unwrap() == parent)
            .map(|(dir, v)| Subgraph {
                title: dir.file_name().unwrap().to_str().unwrap().into(),
                nodes: v.iter().map(|id| format!("{}", id)).collect::<Vec<_>>(),
                subgraphs: Box::new(subgraph_recursive(dir, dirs)),
            })
            .collect::<Vec<_>>()
    }

    subgraph_recursive(dirs.keys().next().unwrap(), &dirs)
}

pub(super) fn gen_svg<T: GenerateSVG>(
    generator: &T,
    files: &Vec<File>,
    refs: References,
) -> String {
    let tables = files.iter().map(|f| file_to_table_node(f)).collect();

    generator.gen_svg(&tables, &refs, &subgraphs(files))
}
