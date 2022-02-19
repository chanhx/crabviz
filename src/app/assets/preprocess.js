const forEachNode = (parent, selector, fn) => {
  let nodes = parent.querySelectorAll(selector);
  for (let node of nodes) {
    fn(node);
  }
};

const preprocessSVG = (svg) => {
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
