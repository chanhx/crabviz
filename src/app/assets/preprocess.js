const forEachNode = (parent, selector, fn) => {
  let nodes = parent.querySelectorAll(selector);
  for (let node of nodes) {
    fn(node);
  }
};

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

  forEachNode(svg, "title", (el) => el.remove());
};

let svg = document.querySelector("svg");
preprocessSVG(svg);
