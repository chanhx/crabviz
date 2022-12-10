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

pub struct Cell {
    pub id: String,
    pub port: String,
    pub title: String,
    pub styles: Vec<CellStyle>,
    pub children: Vec<Cell>,
}

pub struct TableNode {
    pub id: String,
    pub title: String,
    pub sections: Vec<Cell>,
}

pub struct Subgraph {
    pub title: String,
    pub nodes: Vec<String>,
    pub subgraphs: Vec<Subgraph>,
}

pub enum TableStyle {
    Border(u8),
    CssClass(String),
}

pub enum CellStyle {
    Border(u8),
    CssClass(String),
    Rounded,
    Table(Vec<TableStyle>),
}

#[derive(Debug, Clone)]
pub enum EdgeStyle {
    CssClass(String),
}
