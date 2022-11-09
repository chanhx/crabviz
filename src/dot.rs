use snafu::ResultExt;

use {
    crate::{
        app::{Edge, GenerateSVG, Node, Subgraph, TableNode},
        error::{self, Result},
    },
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
            .context(error::RuntimeSnafu)?;

        let cmd_stdin = cmd.stdin.as_mut().unwrap();
        cmd_stdin
            .write_all(graph.as_bytes())
            .map_err(Into::into)
            .context(error::RuntimeSnafu)?;
        drop(cmd_stdin);

        Ok(cmd
            .wait_with_output()
            .map_err(Into::into)
            .context(error::RuntimeSnafu)?)
    }

    pub(crate) fn export_svg(&self, graph: String) -> Result<String> {
        let output = self.export(graph, "svg")?;
        let output = String::from_utf8(output.stdout)
            .map_err(Into::into)
            .context(error::RuntimeSnafu)?;

        Ok(output)
    }
}

impl GenerateSVG for Dot {
    fn gen_svg(
        &self,
        tables: &Vec<TableNode>,
        // nodes: Vec<Node>,
        refs: &Vec<Edge>,
        subgraphs: &Vec<Subgraph>,
    ) -> String {
        let tables = tables
            .iter()
            .map(|table| {
                format!(
                    r#"
    "{id}" [id="{id}", label=<
        <TABLE BORDER="0" CELLBORDER="0">
        <TR><TD WIDTH="230" BORDER="0"><FONT POINT-SIZE="12">{title}</FONT></TD></TR>
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
                        .map(|node| section(node))
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
            edges(refs),
        );

        // println!("{}", graph);

        self.export_svg(graph).unwrap()
    }
}

fn cell(node: &Node) -> String {
    let sub_cells = match node.children.as_ref() {
        Some(children) => children.iter().map(|item| cell(item)).collect::<Vec<_>>(),
        None => Vec::new(),
    };

    let classes = match &node.classes {
        Some(classes) => classes.join(""),
        _ => "".into(),
    };

    let cell = format!(
        r#"     <TR><TD PORT="{port}" ID="{id}" HREF="remove_me_url.cell{classes}">{name}</TD></TR>"#,
        port = node.port,
        id = node.id,
        name = escape_html(&node.title),
    );

    iter::once(cell)
        .chain(sub_cells.into_iter())
        .collect::<Vec<_>>()
        .join("\n")
}

fn section(node: &Node) -> String {
    format!(
        r#"
        <TR><TD>
        <TABLE BORDER="0" CELLSPACING="0" CELLPADDING="4" CELLBORDER="1">
        {cell}
        </TABLE>
        </TD></TR>
        "#,
        cell = cell(node),
    )
}

fn edges(edges: &Vec<Edge>) -> String {
    edges
        .iter()
        .map(|edge| {
            let from = format!(r#"{}:"{}""#, edge.from_table_id, edge.from_node_id);
            let to = format!(r#"{}:"{}""#, edge.to_table_id, edge.to_node_id);

            let mut attrs = vec![format!(
                r#"id="{}:{} -> {}:{}""#,
                edge.from_table_id, edge.from_node_id, edge.to_table_id, edge.to_node_id
            )];
            // let mut attrs = vec![];
            let pt = if edge.from_table_id == edge.to_table_id {
                attrs.push(r#"class="modify-me""#.to_string());
                ":w"
            } else {
                ""
            };

            format!(
                "{}{pt} -> {}{pt} [{attrs}];",
                from,
                to,
                pt = pt,
                attrs = attrs.join(", "),
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ")
}

fn clusters(subgraphs: &Vec<Subgraph>) -> String {
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
