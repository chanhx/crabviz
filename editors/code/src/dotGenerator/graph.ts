
export interface GenerateSVG {
  generateSvg(
    tables: TableNode[],
    // nodes: Node[],
    edges: Edge[],
    subgraphs: Subgraph[]
  ): string;
}

export class Edge {
  from: [number, number, number];
  to: [number, number, number];
  classes: CssClass[];

  constructor(from: [number, number, number], to: [number, number, number], classes: CssClass[]) {
    this.from = from;
    this.to = to;
    this.classes = classes;
  }

  hashCode(): number {
    let hash = 0;
    hash = this.from.reduce((acc, val) => acc + val, hash);
    hash = this.to.reduce((acc, val) => acc + val, hash);
    return hash;
  }

  equals(other: Edge): boolean {
    return this.from.every((val, index) => val === other.from[index]) &&
           this.to.every((val, index) => val === other.to[index]);
  }
}

export class Cell {
  rangeStart: [number, number];
  rangeEnd: [number, number];
  title: string;
  style: Style;
  children: Cell[];
  isCollapsed: boolean;

  constructor(rangeStart: [number, number], rangeEnd: [number, number], title: string, style: Style, children: Cell[]) {
    this.rangeStart = rangeStart;
    this.rangeEnd = rangeEnd;
    this.title = title;
    this.style = style;
    this.children = children;
    this.isCollapsed = false;
  }

  highlight(cells: Set<[number, number]>) {
    if (cells.has(this.rangeStart)) {
      this.style.classes.push(CssClass.Highlight);
    }
    this.children.forEach(child => child.highlight(cells));
  }
}

export class TableNode {
  id: number;
  title: string;
  sections: Cell[];

  constructor(id: number, title: string, sections: Cell[]) {
    this.id = id;
    this.title = title;
    this.sections = sections;
  }

  highlightCells(cells: Set<[number, number]>) {
    this.sections.forEach(section => section.highlight(cells));
  }
}

export class Subgraph {
  title: string;
  nodes: string[];
  subgraphs: Subgraph[];

  constructor(title: string, nodes: string[], subgraphs: Subgraph[]) {
    this.title = title;
    this.nodes = nodes;
    this.subgraphs = subgraphs;
  }
}

export class Style {
  rounded: boolean;
  border?: number;
  icon?: string;
  classes: CssClass[];

  constructor(rounded: boolean, border?: number, icon?: string, classes: CssClass[] = []) {
    this.rounded = rounded;
    this.border = border;
    this.icon = icon;
    this.classes = classes;
  }
}

export enum CssClass {
  Module = "module",
  Interface = "interface",
  Function = "function",
  Method = "method",
  Constructor = "constructor",
  Property = "property",
  Type = "type",
  Impl = "impl",
  Clickable = "clickable",
  Highlight = "highlight",
  Cell = "cell"
}

export function cssClassToString(cssClass: CssClass): string {
  return cssClass;
}
