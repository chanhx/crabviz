use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
    iter,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::Result;

use crate::file_structure::{File, FilePosition, RItem, RItemType};

const MIN_WIDTH: u32 = 230;

fn ritem_cell(item: &RItem) -> String {
    let class = match item.ty {
        RItemType::Func => ".fn",
        _ => "",
    };

    format!(
        r#"<TR><TD PORT="{port}" ID="{id}" HREF="remove_me_url.cell{class}">{name}</TD></TR>"#,
        port = item.pos.offset,
        id = item.pos,
        class = class,
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
        <TABLE BORDER="0" CELLSPACING="0" CELLPADDING="4" CELLBORDER="1">
        {}
        </TABLE>
        </TD></TR>
        "#,
        cells,
    )
}

fn file_node(m: &File) -> String {
    let cells = m
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
    "{id}" [id="{id}", label=<
        <TABLE BORDER="0" CELLBORDER="0">
        {header}
        {cells}
        <TR><TD BORDER="0"></TD></TR>
        </TABLE>
    >]
        "#,
        id = m.file_id,
        header = node_header,
        cells = cells,
    )
}

pub(crate) type CallMap = HashMap<FilePosition, Vec<FilePosition>>;

fn call_edges(calls: &CallMap) -> String {
    let calls = calls
        .iter()
        .flat_map(|(k, v)| {
            v.iter()
                .map(|pos| {
                    let mut attrs = vec![format!(r#"id="{} -> {}""#, pos, k)];
                    let pt = if k.file_id == pos.file_id {
                        attrs.push(r#"class="modify-me""#.to_string());
                        ":w"
                    } else {
                        ""
                    };

                    format!(
                        "{}{pt} -> {}{pt} [{attrs}]",
                        pos,
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

fn subgraph(files: &Vec<File>) -> String {
    let mut dirs = BTreeMap::<&Path, Vec<u32>>::default();
    for f in files {
        let parent = f.path.parent().unwrap();
        if let Some(v) = dirs.get_mut(parent) {
            (*v).push(f.file_id);
        } else {
            dirs.insert(parent, vec![f.file_id]);
        }
    }

    fn subgraph_recursive(parent: &Path, dirs: &BTreeMap<&Path, Vec<u32>>) -> String {
        dirs.iter()
            .filter(|(dir, _)| dir.parent().unwrap() == parent)
            .map(|(dir, v)| {
                format!(
                    r#"
    subgraph cluster_{name} {{
        label = "{name}";

        {nodes}

        {subgraph}
    }}
                    "#,
                    name = dir.file_name().unwrap().to_str().unwrap(),
                    nodes = v
                        .iter()
                        .map(|id| format!("{}", id))
                        .collect::<Vec<_>>()
                        .join(" "),
                    subgraph = subgraph_recursive(dir, dirs),
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    subgraph_recursive(dirs.keys().next().unwrap(), &dirs)
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

    {}
}}
        "#,
        files
            .iter()
            .map(|m| file_node(m))
            .collect::<Vec<_>>()
            .join("\n"),
        subgraph(files),
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
