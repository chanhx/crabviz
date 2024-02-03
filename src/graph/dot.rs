use {
    super::CssClass,
    crate::graph::{Cell, Edge, Subgraph, TableNode},
    std::iter,
};

pub(crate) fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("\"", "&quot;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

pub(crate) struct Dot;

impl Dot {
    pub fn generate_dot_source<T, E>(
        tables: T,
        // nodes: &[Node],
        edges: E,
        subgraphs: &[Subgraph],
    ) -> String
    where
        T: Iterator<Item = TableNode>,
        E: Iterator<Item = Edge>,
    {
        let tables = tables
            .map(|table| {
                format!(
                    r#"
    "{id}" [id="{id}", label=<
        <TABLE BORDER="0" CELLBORDER="1" CELLSPACING="8" CELLPADDING="4">
        <TR><TD WIDTH="230" BORDER="0" CELLPADDING="6" HREF="remove_me_url.title">{title}</TD></TR>
        {sections}
        <TR><TD BORDER="0"></TD></TR>
        </TABLE>
    >];
                    "#,
                    id = table.id,
                    title = table.title,
                    sections = table
                        .sections
                        .iter()
                        .map(|node| Dot::process_cell(table.id, node))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"
digraph {{
    graph [
        rankdir = "LR"
        ranksep = 2.0
        fontname = "Arial"
    ];
    node [
        fontsize = "16"
        fontname = "Arial"
        shape = "plaintext"
        style = "rounded, filled"
    ];
    edge [
        label = " "
    ];

    {}

    {}

    {}
}}
            "#,
            tables,
            Dot::clusters(subgraphs),
            Dot::process_edges(edges),
        )
    }

    fn process_cell(table_id: u32, cell: &Cell) -> String {
        let styles = [
            cell.style
                .border
                .map_or(String::new(), |b| format!(r#"BORDER="{}""#, b)),
            cell.style
                .rounded
                .then_some(r#"STYLE="ROUNDED""#.to_string())
                .unwrap_or(String::new()),
        ]
        .join(" ");

        let port = format!("{}_{}", cell.range_start.0, cell.range_start.1);

        if cell.children.is_empty() {
            format!(
                r#"     <TR><TD PORT="{port}" ID="{table_id}:{port}" {styles} {href}>{i}{name}</TD></TR>"#,
                href = Dot::css_classes_href(&cell.style.classes),
                i = cell
                    .style
                    .icon
                    .map(|c| format!("<B>{}</B>  ", c))
                    .unwrap_or(String::new()),
                name = escape_html(&cell.title),
            )
        } else {
            let (cell_styles, table_styles) = (r#"BORDER="0""#.to_string(), styles);

            let dot_cell = format!(
                r#"     <TR><TD PORT="{port}" {cell_styles} {href}>{name}</TD></TR>"#,
                href = String::new(),
                name = escape_html(&cell.title),
            );

            format!(
                r#"
            <TR><TD BORDER="0" CELLPADDING="0">
            <TABLE ID="{table_id}:{port}" CELLSPACING="4" CELLPADDING="4" CELLBORDER="1" {table_styles} BGCOLOR="green" {href}>
            {}
            </TABLE>
            </TD></TR>
            "#,
                iter::once(dot_cell)
                    .chain(
                        cell.children
                            .iter()
                            .map(|item| Dot::process_cell(table_id, item))
                    )
                    .collect::<Vec<_>>()
                    .join("\n"),
                href = Dot::css_classes_href(&cell.style.classes),
            )
        }
    }

    fn process_edges<E>(edges: E) -> String
    where
        E: Iterator<Item = Edge>,
    {
        edges
            .map(|edge| {
                let from = format!(r#"{}:"{}_{}""#, edge.from.0, edge.from.1, edge.from.2);
                let to = format!(r#"{}:"{}_{}""#, edge.to.0, edge.to.1, edge.to.2);

                let attrs = iter::once(format!(
                    r#"id="{}:{}_{} -> {}:{}_{}""#,
                    edge.from.0, edge.from.1, edge.from.2, edge.to.0, edge.to.1, edge.to.2,
                ))
                .chain(iter::once(Dot::css_classes_href(&edge.styles)))
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();

                format!("{} -> {} [{attrs}];", from, to, attrs = attrs.join(", "),)
            })
            .collect::<Vec<_>>()
            .join("\n    ")
    }

    fn clusters(subgraphs: &[Subgraph]) -> String {
        subgraphs
            .iter()
            .map(|subgraph| {
                format!(
                    r#"
        subgraph "cluster_{name}" {{
            label = "{name}";

            {nodes}

            {subgraph}
        }};
                    "#,
                    name = subgraph.title,
                    nodes = subgraph.nodes.join(" "),
                    subgraph = Dot::clusters(&subgraph.subgraphs),
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn css_classes_href(classes: &[CssClass]) -> String {
        if classes.is_empty() {
            "".to_string()
        } else {
            format!(
                r#"href="remove_me_url.{}""#,
                classes
                    .iter()
                    .map(|c| c.to_str())
                    .collect::<Vec<_>>()
                    .join(".")
            )
        }
    }
}
