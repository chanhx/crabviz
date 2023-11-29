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
    pub from: (u32, u32, u32),
    pub to: (u32, u32, u32),
    pub styles: Vec<EdgeStyle>,
}

impl Hash for Edge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
    }
}

impl Eq for Edge {}

#[derive(Debug)]
pub struct Cell {
    pub range_start: (u32, u32),
    pub range_end: (u32, u32),
    pub title: String,
    pub styles: Vec<Style>,
    pub children: Vec<Cell>,
}

impl Cell {
    pub fn highlight(&mut self, cells: &HashSet<(u32, u32)>) {
        if cells.contains(&self.range_start) {
            self.styles.push(Style::CssClass(String::from("highlight")));
        }
        self.children.iter_mut().for_each(|c| c.highlight(cells));
    }
}

#[derive(Debug)]
pub struct TableNode {
    pub id: u32,
    pub title: String,
    pub sections: Vec<Cell>,
}

impl TableNode {
    pub fn highlight_cells(&mut self, cells: &HashSet<(u32, u32)>) {
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
