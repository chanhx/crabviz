use {
    crate::app::{GenerateSVG, Node, References, Subgraph, TableNode},
    anyhow::Result,
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
            .spawn()?;

        let cmd_stdin = cmd.stdin.as_mut().unwrap();
        cmd_stdin.write_all(graph.as_bytes())?;
        drop(cmd_stdin);

        Ok(cmd.wait_with_output()?)
    }

    pub(crate) fn export_svg(&self, graph: String) -> Result<String> {
        let output = self.export(graph, "svg")?;
        let output = String::from_utf8(output.stdout)?;

        Ok(output)
    }
}

impl GenerateSVG for Dot {
    fn gen_svg(
        &self,
        tables: &Vec<TableNode>,
        // nodes: Vec<Node>,
        refs: &References,
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

fn edges(calls: &References) -> String {
    let calls = calls
        .iter()
        .flat_map(|(k, v)| {
            v.iter()
                .map(|pos| {
                    let dest = pos.dest;
                    let mut attrs = vec![format!(r#"id="{} -> {}""#, dest, k)];
                    let pt = if k.file_id == dest.file_id {
                        attrs.push(r#"class="modify-me""#.to_string());
                        ":w"
                    } else {
                        ""
                    };

                    format!(
                        "{}{pt} -> {}{pt} [{attrs}];",
                        dest,
                        k,
                        pt = pt,
                        attrs = attrs.join(", "),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    calls
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
