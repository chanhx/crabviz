use {
    enumset::{EnumSet, EnumSetType},
    std::{
        collections::HashSet,
        hash::{Hash, Hasher},
    },
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
    pub classes: EnumSet<CssClass>,
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
    pub style: Style,
    pub children: Vec<Cell>,
}

impl Cell {
    pub fn highlight(&mut self, cells: &HashSet<(u32, u32)>) {
        if cells.contains(&self.range_start) {
            self.style.classes.insert(CssClass::Highlight);
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

#[derive(Debug, Default)]
pub struct Style {
    pub rounded: bool,
    pub border: Option<u8>,
    pub icon: Option<char>,
    pub classes: EnumSet<CssClass>,
}

#[derive(EnumSetType, Debug)]
pub enum CssClass {
    Module,

    Interface,

    Function,
    Method,
    Constructor,
    Property,

    Type,

    Impl,

    Clickable,
    Highlight,
    Cell,
}

impl CssClass {
    pub fn to_str(&self) -> &'static str {
        match self {
            CssClass::Module => "module",

            CssClass::Interface => "interface",
            CssClass::Type => "type",

            CssClass::Function => "function",
            CssClass::Method => "method",
            CssClass::Constructor => "constructor",
            CssClass::Property => "property",

            CssClass::Impl => "impl",

            CssClass::Clickable => "clickable",
            CssClass::Highlight => "highlight",
            CssClass::Cell => "cell",
        }
    }
}
