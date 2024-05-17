function exportSVG(uri) {
  const svg = document.querySelector("svg").cloneNode(true);

  svg.appendChild(document.getElementById("crabviz_style").cloneNode(true));
  svg.insertAdjacentHTML(
    "beforeend",
    "<style>:is(.cell, .edge) { pointer-events: none; }</style>"
  );

  acquireVsCodeApi().postMessage({
    command: 'saveSVG',
    uri: uri,
    svg: svg.outerHTML.replaceAll("&nbsp;", "&#160;")
  });
}

window.addEventListener('message', (e) => {
  const message = e.data;

  switch (message.command) {
    case 'exportSVG':
        exportSVG(message.uri);
        break;
  }
});
