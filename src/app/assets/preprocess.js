const forEachNode = (parent, selector, fn) => {
  parent.querySelectorAll(selector).forEach(fn);
};

const getParent = (elem, className) => {
  while (elem && elem.tagName !== 'svg') {
    if (elem.classList.contains(className)) return elem;
    elem = elem.parentNode;
  }

  return null;
}

const isCell = (elem) => {
  return getParent(elem, "cell") != null;
}
const isEdge = (elem) => {
  return getParent(elem, "edge") != null;
}
const isNode = (elem) => {
  return getParent(elem, "node") != null;
}

const preprocessSVG = (svg) => {
  forEachNode(svg, "g.edge.modify-me path", (path) => {
    let [m, c1, c2] = path.getPathData();
    let [mX, mY] = m.values;

    c1.values[0] = (c1.values[0] - mX) / 4 + mX;
    c1.values[2] = (c1.values[2] - mX) / 4 + mX;
    c1.values[4] = (c1.values[4] - mX) / 4 + mX;

    c2.values[0] = (c2.values[0] - mX) / 4 + mX;
    c2.values[2] = (c2.values[2] - mX) / 4 + mX;

    c1.values[1] = (c1.values[1] + mY) / 2;
    c1.values[3] = (c1.values[3] + mY) / 2;

    c2.values[1] = (c2.values[1] + c2.values[5]) / 2;
    c2.values[3] = (c2.values[3] + c2.values[5]) / 2;

    path.setPathData([m, c1, c2]);

    path.parentNode.classList.remove("modify-me");
  });

  forEachNode(svg, "g.edge path", (path) => {
    let newPath = path.cloneNode();
    newPath.classList.add("hover-path");
    newPath.removeAttribute("stroke-dasharray");
    path.parentNode.appendChild(newPath);
  });

  forEachNode(svg, 'a', (a) => {
    let urlComps = a.href.baseVal.split(".");
    if (urlComps[0] != "remove_me_url") {
      return;
    }

    let docFrag = document.createDocumentFragment();
    docFrag.append(...a.childNodes);

    let g = a.parentNode;
    g.replaceChild(docFrag, a);
    g.id = g.id.replace(/^a_/, '');

    if (urlComps.length > 1) {
      g.classList.add(...urlComps.slice(1));
    }
  });

  forEachNode(svg, "g.edge", (edge) => {
    let [from, to] = edge.id.split(' -> ');

    edge.setAttribute('edge-from', from);
    edge.setAttribute('edge-to', to);
  });

  forEachNode(svg, "title", (el) => el.remove());
};

const onSelectEdge = (svg, target) => {
  let edge = getParent(target, "edge");
  let id = edge.id;

  let selectedEdgeID = svg.state.selectedEdgeID;
  if (selectedEdgeID == id) {
    svg.state.selectedEdgeID = null;
  } else {
    edge.classList.add("selected");
    svg.state.selectedEdgeID = id;
  }
}

const onSelectCell = (svg, target) => {
  let cell = getParent(target, "cell");
  if (!cell.classList.contains("fn")) {
    return;
  }

  let id = cell.id;

  let selectedCellID = svg.state.selectedCellID;
  if (selectedCellID == id) {
    svg.state.selectedCellID = null;
  } else {
    forEachNode(svg, `g.edge[edge-from="${id}"]`, (elem) => {
      elem.classList.add("incoming");
    });
    forEachNode(svg, `g.edge[edge-to="${id}"]`, (elem) => {
      elem.classList.add("outgoing");
    });
    forEachNode(svg, "g.edge:not(.incoming):not(.outgoing)", (elem) => {
      elem.classList.add("fade");
    });

    cell.classList.add("selected");
    svg.state.selectedCellID = id;
  }
}

const onSelectNode = (svg, target) => {
  let node = getParent(target, "node");
  let id = node.id;

  let selectedNodeID = svg.state.selectedNodeID;
  if (selectedNodeID == id) {
    svg.state.selectedNodeID = null;
  } else {
    forEachNode(svg, `g.edge[edge-from^="${id}"]`, (elem) => {
      elem.classList.add("incoming");
    });
    forEachNode(svg, `g.edge[edge-to^="${id}"]`, (elem) => {
      elem.classList.add("outgoing");
    });
    forEachNode(svg, "g.edge:not(.incoming):not(.outgoing)", (elem) => {
      elem.classList.add("fade");
    });

    node.classList.add("selected");

    svg.state.selectedNodeID = id;
  }
}

const reset = (svg) => {
  forEachNode(svg, "g.cell.selected", (elem) => {
    elem.classList.remove("selected");
  });

  forEachNode(svg, "g.edge", (elem) => {
    elem.classList.remove("fade", "incoming", "outgoing", "selected");
  });

  forEachNode(svg, "g.node", (elem) => {
    elem.classList.remove("fade", "selected");
  })
}

const addListeners = (svg) => {
  svg.addEventListener("mouseup", (event) => {
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
}

preprocessSVG(svg);
addListeners(svg);
