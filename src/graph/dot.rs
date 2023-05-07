use {
    crate::graph::{Cell, CellStyle, Edge, EdgeStyle, Subgraph, TableNode, TableStyle},
    std::iter,
};

pub(crate) fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("\"", "&quot;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

pub(crate) struct Dot;

// use graphviz_sys as gv;
// use std::ffi::CString;
// impl GenerateSVG for Dot {
//     fn generate_svg(
//         &self,
//         tables: &[TableNode],
//         // nodes: &[Node],
//         edges: &[Edge],
//         subgraphs: &[Subgraph],
//     ) -> String {
//         let dot_source = Dot::generate_dot_source(tables, edges, subgraphs);

//         unsafe {
//             let dot_source = CString::new(dot_source).unwrap();
//             let graph = gv::agmemread(dot_source.as_ptr());

//             let gvc = gv::gvContext();
//             let input_format = CString::new("dot").unwrap();
//             let output_format = CString::new("svg").unwrap();

//             gv::gvLayout(gvc, graph, input_format.as_ptr());

//             let mut data = std::ptr::null_mut();
//             let data: *mut *mut i8 = &mut data;
//             let mut data_size: u32 = 0;
//             gv::gvRenderData(
//                 gvc,
//                 graph,
//                 output_format.as_ptr(),
//                 data,
//                 &mut data_size as _,
//             );

//             gv::gvFreeLayout(gvc, graph);
//             gv::agclose(graph);
//             gv::gvFreeContext(gvc);

//             String::from_raw_parts(*data as _, data_size as _, data_size as _)
//         }
//     }
// }

impl Dot {
    pub fn generate_dot_source(
        tables: &[TableNode],
        // nodes: &[Node],
        edges: &[Edge],
        subgraphs: &[Subgraph],
    ) -> String {
        let tables = tables
            .iter()
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
                        .map(|node| Dot::process_cell(node))
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
    ];
    node [
        fontsize = "16"
        fontname = "helvetica, open-sans"
        shape = "plaintext"
        style = "rounded, filled"
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

    fn process_cell(cell: &Cell) -> String {
        let children = cell
            .children
            .iter()
            .map(|item| Dot::process_cell(item))
            .collect::<Vec<_>>();

        let classes = cell
            .styles
            .iter()
            .filter_map(|s| match s {
                CellStyle::CssClass(cls) => Some(cls.clone()),
                _ => None,
            })
            .chain(iter::once("cell".to_string()))
            .collect::<Vec<_>>();

        let mut table_styles = None;

        let styles = cell
            .styles
            .iter()
            .filter(|s| !matches!(s, CellStyle::CssClass(_)))
            .map(|s| match s {
                CellStyle::Border(w) => format!(r#"BORDER="{}""#, w),
                CellStyle::Rounded => r#"STYLE="ROUNDED""#.to_string(),
                CellStyle::Table(styles) => {
                    table_styles = Some(styles);
                    "".to_string()
                }
                _ => "".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ");

        let cell = format!(
            r#"     <TR><TD PORT="{port}" ID="{id}" {styles} {href}>{name}</TD></TR>"#,
            port = cell.port,
            id = cell.id,
            href = Dot::css_classes_href(&classes),
            name = escape_html(&cell.title),
        );

        match table_styles {
            Some(styles) => {
                let classes = styles
                    .iter()
                    .filter_map(|s| match s {
                        TableStyle::CssClass(cls) => Some(cls.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                format!(
                    r#"
                <TR><TD BORDER="0" CELLPADDING="0">
                <TABLE CELLSPACING="4" CELLPADDING="4" BORDER="0" CELLBORDER="1" STYLE="ROUNDED" BGCOLOR="green" {href}>
                {}
                </TABLE>
                </TD></TR>
                "#,
                    iter::once(cell)
                        .chain(children.into_iter())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    href = Dot::css_classes_href(&classes),
                )
            }

            None => cell,
        }
    }

    fn process_edges(edges: &[Edge]) -> String {
        edges
            .iter()
            .map(|edge| {
                let from = format!(r#"{}:"{}""#, edge.from_table_id, edge.from_node_id);
                let to = format!(r#"{}:"{}""#, edge.to_table_id, edge.to_node_id);

                let classes = edge
                    .styles
                    .iter()
                    .filter_map(|s| match s {
                        EdgeStyle::CssClass(cls) => Some(cls.clone()),
                    })
                    .collect::<Vec<_>>();

                let mut attrs = iter::once(format!(
                    r#"id="{}:{} -> {}:{}""#,
                    edge.from_table_id, edge.from_node_id, edge.to_table_id, edge.to_node_id
                ))
                .chain(iter::once(Dot::css_classes_href(&classes)))
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();

                if edge.from_table_id == edge.to_table_id {
                    attrs.push(r#"label=" ""#.to_string());
                };

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

    fn css_classes_href(classes: &[String]) -> String {
        if classes.is_empty() {
            "".to_string()
        } else {
            format!(r#"href="remove_me_url.{}""#, classes.join("."))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_svg() {
        let edges = vec![Edge {
            from_table_id: "abc".to_string(),
            from_node_id: "1".to_string(),
            to_table_id: "def".to_string(),
            to_node_id: "2".to_string(),
            styles: vec![],
        }];

        let svg = Dot::generate_dot_source(&[], &edges, &[]);

        println!("{}", svg);
    }
}
