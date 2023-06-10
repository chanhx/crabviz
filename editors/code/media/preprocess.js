const forEachNode = (parent, selector, fn) => {
  parent.querySelectorAll(selector).forEach(fn);
};

const addClass = (parent, selector, ...classes) => {
  forEachNode(parent, selector, (elem) => {
    elem.classList.add(...classes);
  });
};
const removeClass = (parent, selector, ...classes) => {
  forEachNode(parent, selector, (elem) => {
    elem.classList.remove(...classes);
  });
};

const getParent = (elem, className) => {
  while (elem && elem.tagName !== "svg") {
    if (elem.classList.contains(className)) {
      return elem;
    }
    elem = elem.parentNode;
  }

  return null;
};

const isCell = (elem) => {
  return getParent(elem, "cell") !== null;
};
const isEdge = (elem) => {
  return getParent(elem, "edge") !== null;
};
const isNode = (elem) => {
  return getParent(elem, "node") !== null;
};

const preprocessSVG = (svg) => {
  forEachNode(svg, "g.edge path", (path) => {
    let newPath = path.cloneNode();
    newPath.classList.add("hover-path");
    newPath.removeAttribute("stroke-dasharray");
    path.parentNode.appendChild(newPath);
  });

  forEachNode(svg, "a", (a) => {
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

  forEachNode(svg, "g.edge", (edge) => {
    let [from, to] = edge.id.split(" -> ");

    edge.setAttribute("edge-from", from);
    edge.setAttribute("edge-to", to);
  });

  forEachNode(svg, "title", (el) => el.remove());
};

const onSelectEdge = (svg, target) => {
  let edge = getParent(target, "edge");
  let id = edge.id;

  let selectedEdgeID = svg.state.selectedEdgeID;
  if (selectedEdgeID === id) {
    svg.state.selectedEdgeID = null;
  } else {
    edge.classList.add("selected");
    addClass(svg, "g.edge:not(.selected)", "fade");

    svg.state.selectedEdgeID = id;
  }
};

const onSelectCell = (svg, target) => {
  let cell = getParent(target, "cell");
  if (!cell.classList.contains("fn")) {
    return;
  }

  let id = cell.id;

  let selectedCellID = svg.state.selectedCellID;
  if (selectedCellID === id) {
    svg.state.selectedCellID = null;
  } else {
    addClass(svg, `g.edge[edge-from="${id}"]`, "incoming");
    addClass(svg, `g.edge[edge-to="${id}"]`, "outgoing");
    addClass(svg, "g.edge:not(.incoming):not(.outgoing)", "fade");

    cell.classList.add("selected");
    svg.state.selectedCellID = id;
  }
};

const onSelectNode = (svg, target) => {
  let node = getParent(target, "node");
  let id = node.id;

  let selectedNodeID = svg.state.selectedNodeID;
  if (selectedNodeID === id) {
    svg.state.selectedNodeID = null;
  } else {
    addClass(svg, `g.edge[edge-from^="${id}:"]`, "incoming");
    addClass(svg, `g.edge[edge-to^="${id}:"]`, "outgoing");
    addClass(svg, "g.edge:not(.incoming):not(.outgoing)", "fade");

    node.classList.add("selected");

    svg.state.selectedNodeID = id;
  }
};

const reset = (svg) => {
  removeClass(svg, "g.cell.selected", "selected");
  removeClass(svg, "g.edge", "fade", "incoming", "outgoing", "selected");
  removeClass(svg, "g.node", "selected");
};

const addListeners = (svg) => {
  const delta = 6;
  let startX;
  let startY;

  svg.addEventListener('mousedown', (event) => {
    startX = event.pageX;
    startY = event.pageY;
  });

  svg.addEventListener("mouseup", (event) => {
    const diffX = Math.abs(event.pageX - startX);
    const diffY = Math.abs(event.pageY - startY);

    if (diffX > delta || diffY > delta) {
      // a mouse drag event
      return;
    }

    reset(svg);

    let target = event.target;

    if (isEdge(target)) {
      onSelectEdge(svg, target);
    } else if (isCell(target)) {
      onSelectCell(svg, target);
    } else if (isNode(target)) {
      onSelectNode(svg, target);
    }
  });
};

let svg = document.querySelector("svg");
svg.state = {
  selectedCellID: null,
  selectedEdgeID: null,
  selectedNodeID: null,
};

preprocessSVG(svg);
addListeners(svg);

svgPanZoom(svg, {
  "dblClickZoomEnabled": false,
});
