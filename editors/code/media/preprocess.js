const forEachSelectedChild = (parent, selector, fn) => {
  parent.querySelectorAll(selector).forEach(fn);
};

const GraphElemType = {
  NODE: 0,
  CELL: 1,
  EDGE: 2,
};

class CallGraph {
  svg;
  edges;
  nodes;

  constructor(svg) {
    this.svg = svg;
    this.edges = svg.querySelectorAll("g.edge");
    this.nodes = svg.querySelectorAll("g.node");
  }

  activate() {
    this.processSVG();
    this.addListeners();
  }

  processSVG() {
    this.edges.forEach(edge => forEachSelectedChild(edge, "path", (path) => {
      let newPath = path.cloneNode();
      newPath.classList.add("hover-path");
      newPath.removeAttribute("stroke-dasharray");
      path.parentNode.appendChild(newPath);
    }));

    forEachSelectedChild(this.svg, "a", (a) => {
      let urlComps = a.href.baseVal.split(".");
      if (urlComps[0] !== "remove_me_url") {
        return;
      }

      let docFrag = document.createDocumentFragment();
      docFrag.append(...a.childNodes);

      let g = a.parentNode;
      g.replaceChild(docFrag, a);
      g.id = g.id.replace(/^a_/, "");

      if (urlComps.length > 1) {
        g.classList.add(...urlComps.slice(1));
      }
    });

    this.edges.forEach((edge) => {
      let [from, to] = edge.id.split(" -> ");

      edge.setAttribute("edge-from", from);
      edge.setAttribute("edge-to", to);
    });

    forEachSelectedChild(this.svg, "title", (el) => el.remove());
  }

  addListeners() {
    const delta = 6;
    let startX;
    let startY;

    this.svg.addEventListener('mousedown', (event) => {
      startX = event.pageX;
      startY = event.pageY;
    });

    this.svg.addEventListener("mouseup", (event) => {
      const diffX = Math.abs(event.pageX - startX);
      const diffY = Math.abs(event.pageY - startY);

      if (diffX > delta || diffY > delta) {
        // a mouse drag event
        return;
      }

      this.reset();

      const target = event.target;
      const elemTuple = this.findNearestGraphElem(target);

      if (elemTuple === null) {
        return;
      }

      const [elem, elemType] = elemTuple;

      switch (elemType) {
        case GraphElemType.NODE:
          this.onSelectNode(elem);
          break;
        case GraphElemType.CELL:
          this.onSelectCell(elem);
          break;
        case GraphElemType.EDGE:
          this.onSelectEdge(elem);
          break;
      }
    });
  }

  reset() {
    this.nodes.forEach(node => {
      node.classList.remove("selected");
      forEachSelectedChild(node, "g.cell.selected", (elem) => {
        elem.classList.remove("selected");
      });
    });
    this.edges.forEach(edge => edge.classList.remove("fade", "incoming", "outgoing", "selected"));
  };

  onSelectEdge(edge) {
    this.edges.forEach(e => {
        e.classList.add(e === edge ? "selected" : "fade");
    });
  };

  onSelectCell(cell) {
    if (!cell.classList.contains("fn")) {
      return;
    }

    const id = cell.id;

    this.edges.forEach(edge => {
      let fade = true;

      if (edge.matches(`[edge-from="${id}"]`)) {
        edge.classList.add("incoming");
        fade = false;
      }
      if (edge.matches(`[edge-to="${id}"]`)) {
        edge.classList.add("outgoing");
        fade = false;
      }

      if (fade) {
        edge.classList.add("fade");
      }
    });

    cell.classList.add("selected");
  };

  onSelectNode(node) {
    const id = node.id;

    this.edges.forEach(edge => {
      let fade = true;

      if (edge.matches(`[edge-from^="${id}:"]`)) {
        edge.classList.add("incoming");
        fade = false;
      }
      if (edge.matches(`[edge-to^="${id}:"]`)) {
        edge.classList.add("outgoing");
        fade = false;
      }

      if (fade) {
        edge.classList.add("fade");
      }
    });

    node.classList.add("selected");
  }

  findNearestGraphElem(elem) {
    while (elem && elem !== this.svg) {
      for (let i = 0; i < elem.classList.length; ++i) {
        let cls = elem.classList.item(i);

        if (cls === "node") {
          return [elem, GraphElemType.NODE];
        } else if (cls === "cell") {
          return [elem, GraphElemType.CELL];
        } else if (cls === "edge") {
          return [elem, GraphElemType.EDGE];
        }
      }

      elem = elem.parentNode;
    }

    return null;
  }
}

const graph = new CallGraph(document.querySelector("svg"));
graph.activate();

svgPanZoom(graph.svg, {
  "dblClickZoomEnabled": false,
});
