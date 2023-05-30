use {
    crate::graph::{Cell, Edge, EdgeStyle, Style, Subgraph, TableNode},
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
        fontname = "Arial"
    ];
    node [
        fontsize = "16"
        fontname = "Arial"
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
        let classes = cell
            .styles
            .iter()
            .filter_map(|s| match s {
                Style::CssClass(cls) => Some(cls.clone()),
                _ => None,
            })
            .chain(iter::once("cell".to_string()))
            .collect::<Vec<_>>();

        let styles = cell
            .styles
            .iter()
            .filter(|s| !matches!(s, Style::CssClass(_)))
            .map(|s| match s {
                Style::Border(w) => format!(r#"BORDER="{}""#, w),
                Style::Rounded => r#"STYLE="ROUNDED""#.to_string(),
                _ => "".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ");

        let (cell_styles, table_styles) = if cell.children.is_empty() {
            (styles, "".to_string())
        } else {
            (r#"BORDER="0""#.to_string(), styles)
        };

        if cell.children.is_empty() {
            format!(
                r#"     <TR><TD PORT="{port}" ID="{id}" {cell_styles} {href}>{name}</TD></TR>"#,
                port = cell.port,
                id = cell.id,
                href = if cell.children.len() > 0 {
                    "".to_string()
                } else {
                    Dot::css_classes_href(&classes)
                },
                name = escape_html(&cell.title),
            )
        } else {
            let dot_cell = format!(
                r#"     <TR><TD PORT="{port}" {cell_styles} {href}>{name}</TD></TR>"#,
                port = cell.port,
                href = if cell.children.len() > 0 {
                    "".to_string()
                } else {
                    Dot::css_classes_href(&classes)
                },
                name = escape_html(&cell.title),
            );

            format!(
                r#"
            <TR><TD BORDER="0" CELLPADDING="0">
            <TABLE ID="{id}" CELLSPACING="4" CELLPADDING="4" CELLBORDER="1" {table_styles} BGCOLOR="green" {href}>
            {}
            </TABLE>
            </TD></TR>
            "#,
                iter::once(dot_cell)
                    .chain(cell.children.iter().map(|item| Dot::process_cell(item)))
                    .collect::<Vec<_>>()
                    .join("\n"),
                id = cell.id,
                href = Dot::css_classes_href(&classes),
            )
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
