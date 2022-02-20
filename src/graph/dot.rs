use std::{
    collections::HashMap,
    io::Write,
    iter,
    process::{Command, Stdio},
};

use anyhow::Result;

use crate::file_structure::{File, FilePosition, RItem};

const MIN_WIDTH: u32 = 230;

fn ritem_cell(item: &RItem) -> String {
    format!(
        r#"<TR><TD PORT="{port}">{name}</TD></TR>"#,
        port = item.pos.offset,
        name = item.ident,
    )
}

fn ritem_table(item: &RItem) -> String {
    static EMPTY: &Vec<RItem> = &vec![];
    let cells = iter::once(item)
        .chain(item.children.as_ref().unwrap_or(EMPTY).iter())
        .map(|m| ritem_cell(m))
        .collect::<Vec<_>>()
        .join("\n        ");

    format!(
        r#"
        <TR><TD>
        <TABLE BORDER="1" CELLBORDER="0" ROWS="*">
        {}
        </TABLE>
        </TD></TR>
        "#,
        cells,
    )
}

fn file_node(m: &File) -> String {
    let groups = m
        .items
        .iter()
        .map(|ritem| ritem_table(ritem))
        .collect::<Vec<_>>()
        .join("\n");

    let node_header = format!(
        r#"<TR><TD WIDTH="{width}" BORDER="0"><FONT POINT-SIZE="12">{title}</FONT></TD></TR>"#,
        width = MIN_WIDTH,
        title = m.path.file_name().unwrap().to_str().unwrap(),
    );

    format!(
        r#"
    "{}" [label=<
        <TABLE BORDER="0" CELLBORDER="0">
        {}
        {}
        <TR><TD BORDER="0"></TD></TR>
        </TABLE>
    >]
        "#,
        m.file_id, node_header, groups,
    )
}

pub(crate) type CallMap = HashMap<FilePosition, Vec<FilePosition>>;

fn call_edges(calls: &CallMap) -> String {
    let calls = calls
        .iter()
        .flat_map(|(k, v)| {
            v.iter()
                .map(|pos| {
                    let (pt, attr) = if k.file_id == pos.file_id {
                        (":w", r#"[class="modify-me"]"#)
                    } else {
                        ("", "")
                    };

                    format!("{}{pt} -> {}{pt} {attr}", pos, k, pt = pt, attr = attr,)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    calls
}

pub(super) fn files_graph(files: &Vec<File>, calls: &CallMap) -> String {
    format!(
        r#"
digraph graphviz {{
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
}}
        "#,
        files
            .iter()
            .map(|m| file_node(m))
            .collect::<Vec<_>>()
            .join("\n"),
        call_edges(calls),
    )
}

pub(super) fn render_svg(graph: String) -> Result<String> {
    let mut cmd = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let cmd_stdin = cmd.stdin.as_mut().unwrap();
    cmd_stdin.write_all(graph.as_bytes())?;
    drop(cmd_stdin);

    let output = cmd.wait_with_output()?;
    let output = String::from_utf8(output.stdout)?;

    Ok(output)
}
