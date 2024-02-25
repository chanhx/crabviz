/**
 * Apply function to each selected child
 *
 * @param {Element} parent
 * @param {string} selectors
 * @param {Function} fn
 */
const forEachSelectedChild = (parent, selectors, fn) => {
  parent.querySelectorAll(selectors).forEach(fn);
};

const GraphElemType = {
  NODE: 0,
  CELL: 1,
  EDGE: 2,
};

class CallGraph {
  /**
   * The SVG element
   * @type {SVGSVGElement}
   */
  svg;

  /**
   * @type {NodeListOf<SVGGElement>}
   */
  edges;

  /**
   * @type {NodeListOf<SVGGElement>}
   */
  nodes;

  /**
   * @type {boolean}
   */
  focusMode;

  /**
   * Focus element
   *
   * @type {SVGGElement}
   */
  focus;

  /**
   * Incoming edges in focus mode
   *
   * @type {NodeListOf<SVGGElement>}
   */
  incomings;

  /**
   * Outgoing edges in focus mode
   *
   * @type {NodeListOf<SVGGElement>}
   */
  outgoings;

  /**
   * @constructor
   * @param {SVGSVGElement} svg
   * @param {boolean} focusMode
   */
  constructor(svg, focusMode) {
    this.svg = svg;
    this.edges = svg.querySelectorAll("g.edge");
    this.nodes = svg.querySelectorAll("g.node");
    this.focusMode = focusMode;
  }

  activate() {
    this.processSVG();
    this.addGraphicalObjects();
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

    if (this.focusMode) {
      this.focus = this.svg.querySelector(".highlight").id;
      this.incomings = new Map();
      this.outgoings = new Map();
    }

    this.edges.forEach((edge) => {
      let [from, to] = edge.id.split(" -> ");

      edge.setAttribute("edge-from", from);
      edge.setAttribute("edge-to", to);

      if (this.focus) {
        if (this.incomings.has(to)) {
          this.incomings.get(to).push(edge);
        } else {
          this.incomings.set(to, [edge]);
        }

        if (this.outgoings.has(from)) {
          this.outgoings.get(from).push(edge);
        } else {
          this.outgoings.set(from, [edge]);
        }
      }
    });

    forEachSelectedChild(this.svg, "title", (el) => el.remove());
  }

  addGraphicalObjects() {
    let defs = document.createElementNS("http://www.w3.org/2000/svg", "defs");
    defs.innerHTML = '<filter id="shadow"><feDropShadow dx="0" dy="0" stdDeviation="4" flood-opacity="0.5"></filter>';

    this.svg.appendChild(defs);
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

  /**
   * Deselect all elements
   */
  reset() {
    this.nodes.forEach(node => {
      node.classList.remove("selected");
      forEachSelectedChild(node, "g.cell.selected", (elem) => {
        elem.classList.remove("selected");
      });
    });
    this.edges.forEach(edge => edge.classList.remove("fade", "incoming", "outgoing", "selected"));
  };

  /**
   * @param {SVGGElement} edge
   */
  onSelectEdge(edge) {
    this.edges.forEach(e => {
      if (e !== edge) {
        e.classList.add("fade");
      }
    });
  };

  /**
   * @param {SVGGElement} cell
   */
  onSelectCell(cell) {
    if (!cell.classList.contains("clickable")) {
      return;
    }

    const id = cell.id;

    if (this.focus) {
      this.highlightEdgeInFocusMode(id);
    } else {
      this.edges.forEach(edge => {
        let fade = true;

        if (edge.matches(`[edge-from="${id}"]`)) {
          edge.classList.add("outgoing");
          fade = false;
        }
        if (edge.matches(`[edge-to="${id}"]`)) {
          edge.classList.add("incoming");
          fade = false;
        }

        if (fade) {
          edge.classList.add("fade");
        }
      });
    }

    cell.classList.add("selected");
  };

  /**
   * @param {SVGGElement} node
   */
  onSelectNode(node) {
    const id = node.id;

    this.edges.forEach(edge => {
      let fade = true;

      if (edge.matches(`[edge-from^="${id}:"]`)) {
        edge.classList.add("outgoing");
        fade = false;
      }
      if (edge.matches(`[edge-to^="${id}:"]`)) {
        edge.classList.add("incoming");
        fade = false;
      }

      if (fade) {
        edge.classList.add("fade");
      }
    });

    node.classList.add("selected");
  }

  /**
   * @param {SVGGElement} elem
   * @returns {SVGGElement}
   */
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

  // TODO: fix highlight color problem in recursive calls
  // consider a recursive call like this:
  // a -> b -> c -> a
  // focus: a
  // at present, when b or c is selected, the edges are not highlighted in right color to show that they are in recursion.

  /**
   * @param {string} cellId
   */
  highlightEdgeInFocusMode(cellId) {
    let incomings = new Set();
    let outgoings = new Set();
    let visited1 = new Set([cellId, this.focus]);
    let visited2 = new Set([cellId, this.focus]);

    let newIncomings = this.incomings.get(cellId) ?? [];
    let newOutgoings = this.outgoings.get(cellId) ?? [];

    while (newIncomings.length > 0) {
      newIncomings = newIncomings
        .flatMap(e => {
          incomings.add(e);

          let id = e.getAttribute("edge-from");
          if (visited1.has(id)) {
            return [];
          }

          visited1.add(id);
          return this.incomings.get(id) ?? [];
        });
    }
    while (newOutgoings.length > 0) {
      newOutgoings = newOutgoings
        .flatMap(e => {
          outgoings.add(e);

          let id = e.getAttribute("edge-to");
          if (visited2.has(id)) {
            return [];
          }

          visited2.add(id);
          return this.outgoings.get(id) ?? [];
        });
    }

    this.edges.forEach(edge => {
      let fade = true;

      if (incomings.has(edge)) {
        edge.classList.add("incoming");
        fade = false;
      }
      if (outgoings.has(edge)) {
        edge.classList.add("outgoing");
        fade = false;
      }

      if (fade) {
        edge.classList.add("fade");
      }
    });
  }
}
