use {
    crate::{
        error::{self, Result},
        graph::{Cell, CellStyle, Edge, EdgeStyle, GenerateSVG, Subgraph, TableNode, TableStyle},
    },
    snafu::ResultExt,
    std::{
        io::Write,
        iter,
        process::{Command, Output, Stdio},
    },
};

pub(crate) fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("\"", "&quot;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

pub(crate) struct Dot {}

impl Dot {
    pub(crate) fn export(&self, graph: String, format: &str) -> Result<Output> {
        let mut cmd = Command::new("dot")
            .arg(format!("-T{}", format))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(Into::into)
            .context(error::GenerateSVGSnafu)?;

        let cmd_stdin = cmd.stdin.as_mut().unwrap();
        cmd_stdin
            .write_all(graph.as_bytes())
            .map_err(Into::into)
            .context(error::GenerateSVGSnafu)?;
        drop(cmd_stdin);

        Ok(cmd
            .wait_with_output()
            .map_err(Into::into)
            .context(error::GenerateSVGSnafu)?)
    }

    pub(crate) fn export_svg(&self, graph: String) -> Result<String> {
        let output = self.export(graph, "svg")?;
        let output = String::from_utf8(output.stdout)
            .map_err(Into::into)
            .context(error::GenerateSVGSnafu)?;

        Ok(output)
    }
}

impl GenerateSVG for Dot {
    fn generate_svg(
        &self,
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
                        .map(|node| process_cell(node))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let graph = format!(
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
            clusters(subgraphs),
            process_edges(edges),
        );

        // println!("{}", graph);

        self.export_svg(graph).unwrap()
    }
}

fn process_cell(cell: &Cell) -> String {
    let children = cell
        .children
        .iter()
        .map(|item| process_cell(item))
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
        href = css_classes_href(&classes),
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
                href = css_classes_href(&classes),
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
            .chain(iter::once(css_classes_href(&classes)))
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
    subgraph cluster_{name} {{
        label = "{name}";

        {nodes}

        {subgraph}
    }};
                "#,
                name = subgraph.title,
                nodes = subgraph.nodes.join(" "),
                subgraph = clusters(&subgraph.subgraphs),
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
