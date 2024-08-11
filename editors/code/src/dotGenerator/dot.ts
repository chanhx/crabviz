
import { Cell, CssClass, Edge, Subgraph, TableNode } from './graph';

function escapeHtml(s: string): string {
  return s.replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

const EMPTY_STRING = "";

//      .filter(node => !node.isCollapsed)
export class Dot {
  static generateDotSourceString(tables: TableNode[], edges: Edge[], subgraphs: Subgraph[]): string {
    const tablesStr = tables.map(table => {
        return `
"${table.id}" [id="${table.id}", label=<
    <TABLE BORDER="0" CELLBORDER="1" CELLSPACING="8" CELLPADDING="4">
    <TR><TD WIDTH="230" BORDER="0" CELLPADDING="6" HREF="remove_me_url.title">${table.title}</TD></TR>
    ${table.sections

      .map(node => Dot.processCell(table.id, node)).join("\n")}
    <TR><TD CELLSPACING="0" HEIGHT="1" WIDTH="1" FIXEDSIZE="TRUE" STYLE="invis"></TD></TR>
    </TABLE>
>];
`     ;}).join("\n");

    return `
digraph {
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

  ${tablesStr}

  ${Dot.clusters(subgraphs)}

  ${Dot.processEdges(edges)}
}
    `;
  }

  static processCell(tableId: number, cell: Cell): string {
    const styles = [
      cell.style.border ? `BORDER="${cell.style.border}"` : "",
      cell.style.rounded ? `STYLE="ROUNDED"` : ""
    ].join(" ");

    const title = `${cell.style.icon ? `<B>${cell.style.icon}</B>  ` : EMPTY_STRING}${escapeHtml(cell.title)}`;
    const port = `${cell.rangeStart[0]}_${cell.rangeStart[1]}`;

    // A cell is either a single table row (when it has no children), or a table row containing an embedded table (when it has children)
    //          .filter(item => !item.isCollapsed)
    if (cell.children.length === 0) {
      return `     <TR><TD PORT="${port}" ID="${tableId}:${port}" ${styles} ${Dot.cssClassesHref(cell.style.classes)}>${title}</TD></TR>`;
    } else {
      const cellStyles = `BORDER="0"`;
      const tableStyles = styles;

      const dotCell = `     <TR><TD PORT="${port}" ${cellStyles} ${EMPTY_STRING}>${title}</TD></TR>`;

      return `
        <TR><TD BORDER="0" CELLPADDING="0">
        <TABLE ID="${tableId}:${port}" CELLSPACING="8" CELLPADDING="4" CELLBORDER="1" ${tableStyles} BGCOLOR="green" ${Dot.cssClassesHref(cell.style.classes)}>
        ${[dotCell, ...cell.children

          .map(item => Dot.processCell(tableId, item))].join("\n")}
        </TABLE>
        </TD></TR>
      `;
    }
  }

  static processEdges(edges: Edge[]): string {
    return edges
      .map(edge => {
        const from = `${edge.from[0]}:"${edge.from[1]}_${edge.from[2]}"`;
        const to = `${edge.to[0]}:"${edge.to[1]}_${edge.to[2]}"`;

        const attrs = [
          `id="${edge.from[0]}:${edge.from[1]}_${edge.from[2]} -> ${edge.to[0]}:${edge.to[1]}_${edge.to[2]}"`,
          Dot.cssClasses(edge.classes)
        ].filter(s => s !== "").join(", ");

        return `${from} -> ${to} [${attrs}];`;
      })
      .join("\n    ");
  }

  static clusters(subgraphs: Subgraph[]): string {
    return subgraphs
      .map(subgraph => {
        return `
        subgraph "cluster_${subgraph.title}" {
          label = "${subgraph.title}";

          ${subgraph.nodes.join(" ")}

          ${Dot.clusters(subgraph.subgraphs)}
        };
      `;
      })
      .join("\n");
  }

  static cssClasses(classes: CssClass[]): string {
    if (classes.length == 0) {
      return "";
    } else {
      return `class="${Array.from(classes).map(c => c.toString()).join(" ")}"`;
    }
  }

  static cssClassesHref(classes: CssClass[]): string {
    if (classes.length == 0) {
      return "";
    } else {
      return `href="remove_me_url.${Array.from(classes).map(c => c.toString()).join(".")}"`;
    }
  }
}
