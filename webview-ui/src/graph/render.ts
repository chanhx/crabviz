import { instance as vizInstance, Graph } from "@viz-js/viz";
import { SymbolKind } from "../lsp";

const viz = vizInstance();

const namespaceURI = "http://www.w3.org/2000/svg";

export async function renderSVG(
  graph: Graph,
  focus: string | null
): Promise<SVGSVGElement> {
  const svg = await viz.then((viz) => viz.renderSVGElement(graph));

  svg.classList.add("callgraph");

  svg.querySelectorAll("title").forEach((el) => el.remove());

  svg.querySelectorAll<SVGAElement>("a").forEach((a) => {
    const href = a.href.baseVal;
    const g = a.parentNode! as SVGElement;

    a.replaceWith(...a.childNodes);
    g.id = g.id!.replace(/^a_/, "");

    const kind = parseInt(href);
    if (isNaN(kind)) {
      g.classList.add("title");
      g.closest(".node")?.setAttribute("data-path", href);

      return;
    }

    g.setAttribute("data-kind", `${kind}`);
    g.classList.add("cell");

    switch (kind) {
      case SymbolKind.MODULE:
        g.classList.add("module");
        break;
      case SymbolKind.FUNCTION:
        g.classList.add("function");
        break;
      case SymbolKind.METHOD:
        g.classList.add("method");
        break;
      case SymbolKind.CONSTRUCTOR:
        g.classList.add("constructor");
        break;
      case SymbolKind.INTERFACE:
        g.classList.add("interface");
        break;
      case SymbolKind.FIELD:
        g.classList.add("field");
        break;
      case SymbolKind.PROPERTY:
        g.classList.add("property");
        break;
      case SymbolKind.CLASS:
        g.classList.add("class");
        break;
      case SymbolKind.ENUM:
        g.classList.add("enum");
        break;
      case SymbolKind.STRUCT:
        g.classList.add("struct");
        break;
      default:
        break;
    }
  });

  svg
    .querySelectorAll<SVGPolygonElement>("g.node polygon")
    .forEach((polygon) => {
      polygon.parentNode!.replaceChild(polygon2rect(polygon), polygon);
    });

  svg
    .querySelectorAll<SVGPolygonElement>(
      "g.cluster > polygon:not(:first-of-type)"
    )
    .forEach((polygon) => {
      const rect = polygon2rect(polygon);
      rect.classList.add("cluster-label");

      polygon.parentNode!.replaceChild(rect, polygon);
    });

  svg.querySelectorAll<SVGGElement>("g.edge").forEach((edge) => {
    const [fromCell, toCell] = edge.id.split("-");

    edge.setAttribute("data-from", fromCell);
    edge.setAttribute("data-to", toCell);

    edge.querySelectorAll("path").forEach((path) => {
      const newPath = path.cloneNode() as SVGElement;
      newPath.classList.add("hover-path");
      newPath.removeAttribute("stroke-dasharray");
      path.parentNode!.appendChild(newPath);
    });
  });

  const fadedLayer = document.createElementNS(namespaceURI, "g");
  fadedLayer.id = "faded-group";
  svg.getElementById("graph0")!.appendChild(fadedLayer);

  const defs = document.createElementNS(namespaceURI, "defs");
  defs.innerHTML = `<filter id="shadow"><feDropShadow dx="0" dy="0" stdDeviation="4" flood-opacity="0.5"></filter>
    <linearGradient id="highlightGradient">
      <stop offset="0%" stop-color="var(--edge-incoming-color)" />
      <stop offset="100%" stop-color="var(--edge-outgoing-color)" />
    </linearGradient>`
  svg.appendChild(defs);

  if (focus) {
    svg.getElementById(focus)?.classList.add("highlight");
  }

  return svg;
}

function polygon2rect(polygon: SVGPolygonElement): SVGRectElement {
  const p0 = polygon.points[0];
  const p2 = polygon.points[2];

  const rect = document.createElementNS(namespaceURI, "rect");
  rect.setAttribute("x", Math.min(p0.x, p2.x).toString());
  rect.setAttribute("y", Math.min(p0.y, p2.y).toString());
  rect.setAttribute("width", Math.abs(p0.x - p2.x).toString());
  rect.setAttribute("height", Math.abs(p0.y - p2.y).toString());

  return rect;
}
