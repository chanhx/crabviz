pub struct Edge {
    pub from_table_id: String,
    pub from_node_id: String,
    pub to_table_id: String,
    pub to_node_id: String,
    pub style: Option<String>,
}

pub struct Node {
    pub id: String,
    pub port: String,
    pub title: String,
    pub classes: Option<Vec<String>>,
    pub children: Option<Vec<Node>>,
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
