import createPanZoom from "panzoom";

export class CallGraph {
  readonly svg: SVGSVGElement;
  readonly nodes: NodeListOf<SVGGElement>;
  readonly edges: NodeListOf<SVGGElement>;
  readonly clusters: NodeListOf<SVGGElement>;
  readonly width: number;
  readonly height: number;
  selectedElem: SVGElement | null = null;
  panZoomState?: ReturnType<typeof this.createPanZoomState>;

  focus: string | null;
  incomings?: Map<string, SVGGElement[]>;
  outgoings?: Map<string, SVGGElement[]>;

  public constructor(
    svg: SVGSVGElement,
    focus: string | null,
    onSelectElem?: (elem: SVGElement | null) => void
  ) {
    this.svg = svg;
    this.nodes = svg.querySelectorAll("g.node");
    this.edges = svg.querySelectorAll("g.edge");
    this.clusters = svg.querySelectorAll("g.cluster");
    this.width = this.svg.viewBox.baseVal.width;
    this.height = this.svg.viewBox.baseVal.height;
    this.setUpInteraction(onSelectElem ?? this.onSelectElem);

    this.focus = focus;
    if (focus) {
      const incomings = new Map();
      const outgoings = new Map();

      this.edges.forEach((edge) => {
        const fromCell = edge.dataset.from,
          toCell = edge.dataset.to;

        incomings.get(toCell)?.push(edge) ?? incomings.set(toCell, [edge]);
        outgoings.get(fromCell)?.push(edge) ?? outgoings.set(fromCell, [edge]);
      });

      this.incomings = incomings;
      this.outgoings = outgoings;
    }
  }

  setUpInteraction(onSelectElem: (elem: SVGElement | null) => void) {
    const svg = this.svg;
    let clickPoint = [0, 0];

    svg.onmousedown = function (e) {
      clickPoint = [e.pageX, e.pageY];
    };
    svg.onmouseup = function (e) {
      const delta = 6;
      const [x, y] = clickPoint;
      const diffX = Math.abs(e.pageX - x);
      const diffY = Math.abs(e.pageY - y);

      if (diffX > delta || diffY > delta) {
        // a mouse drag event
        return;
      }

      for (
        let elem = e.target;
        elem && elem instanceof SVGElement && elem !== svg;
        elem = elem.parentNode
      ) {
        const classes = elem.classList;
        if (
          classes.contains("node") ||
          classes.contains("cell") ||
          classes.contains("edge") ||
          classes.contains("cluster-label")
        ) {
          onSelectElem(elem);
          return;
        }
      }
      onSelectElem(null);
    };
  }

  setUpPanZoom() {
    this.panZoomState = this.createPanZoomState();
  }

  createPanZoomState() {
    const g0 = this.svg.querySelector<SVGElement>("#graph0")!;
    const pz = createPanZoom(g0, {
      smoothScroll: false,
      autocenter: true,
    });

    const { x, y, scale } = structuredClone(pz.getTransform());
    const cRect = g0.getBoundingClientRect();
    const cx = cRect.x + cRect.width / 2;
    const cy = cRect.y + cRect.height / 2;
    const sRect = this.svg.getBoundingClientRect();

    return {
      pz,
      scale,
      x,
      y,
      cx,
      cy,
      sRect,
    };
  }

  public resetStyles() {
    const g0 = this.svg.getElementById("graph0")!;
    const fadedGroup = this.svg.getElementById("faded-group")!;

    this.selectedElem?.closest(".selected")?.classList.remove("selected");
    g0.querySelectorAll(":scope > .edge").forEach((edge) =>
      edge.classList.remove("incoming", "outgoing")
    );

    const firstNode = g0.querySelector(".node");
    fadedGroup.querySelectorAll(".cluster").forEach((cluster) => {
      g0.insertBefore(cluster, firstNode);
    });
    fadedGroup.querySelectorAll(".node").forEach((node) => {
      g0.insertBefore(node, firstNode);
    });

    // faded edges
    g0.append(...fadedGroup.childNodes);
  }

  public smoothZoom(scale: number) {
    if (!this.panZoomState) {
      return;
    }

    const { pz, cx, cy } = this.panZoomState;
    pz.smoothZoom(cx, cy, scale);
  }

  public resetPanZoom() {
    if (!this.panZoomState) {
      return;
    }

    const { pz, x, y, scale } = this.panZoomState;
    pz.moveTo(x, y);
    pz.zoomAbs(x, y, scale);
  }

  public onSelectElem = (elem: SVGElement | null) => {
    this.resetStyles();
    this.selectedElem = elem;

    if (!elem) {
      return;
    }

    const { pz, scale, cx, cy, sRect } = this.panZoomState!;
    const eRect = elem.getBoundingClientRect();
    if (eRect.width > sRect.width || eRect.height > sRect.height) {
      pz.smoothZoomAbs(cx, cy, scale);
      // no `zoomend` event and no callback available, have to use `setTimeout` here
      setTimeout(() => pz.centerOn(elem), 1000);
    } else if (
      eRect.left < sRect.left ||
      eRect.top < sRect.top ||
      eRect.right > sRect.right ||
      eRect.bottom > sRect.bottom
    ) {
      pz.centerOn(elem);
    }

    const classes = elem.classList;
    if (classes.contains("node")) {
      this.onSelectNode(elem);
    } else if (classes.contains("cell")) {
      this.onSelectCell(elem);
    } else if (classes.contains("edge")) {
      this.onSelectEdge(elem);
    } else if (classes.contains("cluster-label")) {
      this.onSelctCluster(elem);
    }
  };

  onSelectNode(node: SVGElement) {
    const id = node.id;

    this.highlightEdges((edge) => [
      edge.dataset.to!.startsWith(`${id}:`),
      edge.dataset.from!.startsWith(`${id}:`),
    ]);

    node.classList.add("selected");
    this.fadeOutNodes(new Set([id]));
  }

  onSelectCell(cell: SVGElement) {
    const id = cell.id;

    if (this.focus) {
      this.onSelectCellInFocusMode(id);
    } else {
      const cellIds = new Set([cell.id]);
      cell.querySelectorAll(".cell").forEach((c) => {
        cellIds.add(c.id);
      });

      this.highlightEdges((edge) => [
        cellIds.has(edge.dataset.to!),
        cellIds.has(edge.dataset.from!),
      ]);
    }

    cell.classList.add("selected");
    this.fadeOutNodes(new Set([this.getNodeId(cell.id)]));
  }

  onSelectCellInFocusMode(cellId: string) {
    const highlights = [new Set<SVGGElement>(), new Set<SVGGElement>()];
    const inout = [this.incomings!, this.outgoings!];

    for (let i = 0; i < inout.length; ++i) {
      const visited = new Set([cellId, this.focus!]);
      const map = inout[i];
      const highlightEdges = highlights[i];

      for (let newEdges = map.get(cellId) ?? []; newEdges.length > 0; ) {
        newEdges = newEdges.flatMap((edge) => {
          highlightEdges.add(edge);

          const id = i == 0 ? edge.dataset.from! : edge.dataset.to!;
          if (visited.has(id)) {
            return [];
          }

          visited.add(id);
          return map.get(id) ?? [];
        });
      }
    }

    this.highlightEdges((edge) => [
      highlights[0].has(edge),
      highlights[1].has(edge),
    ]);

    this.fadeOutNodes(new Set([this.getNodeId(cellId)]));
  }

  onSelectEdge(edge: SVGElement) {
    const g0 = this.svg.getElementById("graph0")!;
    const fadedGroup = this.svg.getElementById("faded-group")!;

    fadedGroup.append(...this.edges);
    g0.appendChild(edge);

    this.fadeOutNodes();
  }

  onSelctCluster(clusterLabel: SVGElement) {
    const cluster = clusterLabel.parentNode! as SVGGElement;
    const rect = cluster.getBoundingClientRect();

    cluster.classList.add("selected");

    const selected = new Set<string>();
    this.nodes.forEach((node) => {
      if (this.rectContains(rect, node.getBoundingClientRect())) {
        selected.add(node.id);
      }
    });

    this.highlightEdges((edge) => [
      selected.has(this.getNodeId(edge.dataset.to!)),
      selected.has(this.getNodeId(edge.dataset.from!)),
    ]);

    this.fadeOutNodes(selected);
  }

  highlightEdges(judge: (edge: SVGGElement) => [boolean, boolean]) {
    const fadedGroup = this.svg.getElementById("faded-group")!;

    for (const edge of this.edges) {
      const [incoming, outgoing] = judge(edge);
      if (incoming) {
        edge.classList.add("incoming");
      }
      if (outgoing) {
        edge.classList.add("outgoing");
      }

      if (!(incoming || outgoing)) {
        fadedGroup.appendChild(edge);
      }
    }
  }

  fadeOutNodes(kept?: Set<string>) {
    if (!kept) {
      kept = new Set();
    }
    const fadedGroup = this.svg.getElementById("faded-group")!;

    for (const edge of this.edges) {
      if (edge.parentElement === fadedGroup) {
        continue;
      }

      kept
        .add(this.getNodeId(edge.dataset.from!))
        .add(this.getNodeId(edge.dataset.to!));
    }

    const clusters = new Set(this.clusters);

    for (const node of this.nodes) {
      if (!kept.has(node.id)) {
        fadedGroup.appendChild(node);
        continue;
      }

      const rect = node.getBoundingClientRect();
      for (const cluster of clusters) {
        if (this.rectContains(cluster.getBoundingClientRect(), rect)) {
          clusters.delete(cluster);
        }
      }
    }

    fadedGroup.append(...clusters);
  }

  getNodeId(id: string): string {
    return id.substring(0, id.indexOf(":"));
  }

  rectContains(rect1: DOMRect, rect2: DOMRect): boolean {
    return (
      rect1.left < rect2.left &&
      rect1.right > rect2.right &&
      rect1.bottom > rect2.bottom &&
      rect1.top < rect2.top
    );
  }
}
