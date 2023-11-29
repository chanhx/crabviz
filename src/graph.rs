use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
};

pub mod dot;

pub trait GenerateSVG {
    fn generate_svg(
        &self,
        tables: &[TableNode],
        // nodes: &[Node],
        edges: &[Edge],
        subgraphs: &[Subgraph],
    ) -> String;
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from_table_id: String,
    pub from_node_id: String,
    pub to_table_id: String,
    pub to_node_id: String,
    pub styles: Vec<EdgeStyle>,
}

impl Hash for Edge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from_table_id.hash(state);
        self.from_node_id.hash(state);
        self.to_table_id.hash(state);
        self.to_node_id.hash(state);
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.from_table_id == other.from_table_id
            && self.from_node_id == other.from_node_id
            && self.to_table_id == other.to_table_id
            && self.to_node_id == other.to_node_id
    }
}

impl Eq for Edge {}

#[derive(Debug)]
pub struct Cell {
    pub id: String,
    pub port: String,
    pub title: String,
    pub styles: Vec<Style>,
    pub children: Vec<Cell>,
}

impl Cell {
    pub fn highlight(&mut self, cells: &HashSet<String>) {
        if cells.contains(&self.port) {
            self.styles.push(Style::CssClass(String::from("highlight")));
        }
        self.children.iter_mut().for_each(|c| c.highlight(cells));
    }
}

#[derive(Debug)]
pub struct TableNode {
    pub id: String,
    pub title: String,
    pub sections: Vec<Cell>,
}

impl TableNode {
    pub fn highlight_cells(&mut self, cells: &HashSet<String>) {
        self.sections.iter_mut().for_each(|c| c.highlight(cells));
    }
}

#[derive(Debug)]
pub struct Subgraph {
    pub title: String,
    pub nodes: Vec<String>,
    pub subgraphs: Vec<Subgraph>,
}

#[derive(Debug)]
pub enum Style {
    Border(u8),
    CssClass(String),
    Rounded,
}

#[derive(Debug, Clone)]
pub enum EdgeStyle {
    CssClass(String),
}
