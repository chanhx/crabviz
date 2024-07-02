import { provideVSCodeDesignSystem, vsCodeButton, Button, vsCodeTextField } from "@vscode/webview-ui-toolkit";

provideVSCodeDesignSystem().register(vsCodeTextField(), vsCodeButton());


const vscode = acquireVsCodeApi();

window.addEventListener('load', main);
window.addEventListener('message', (e) => {
  const message = e.data;

  switch (message.command) {
    case 'exportSVG':
      exportSVG(message.uri);
      break;
  }
});

function main() {
  const saveButton = document.getElementById('crabviz_save_button') as Button;
  saveButton?.addEventListener('click', () => {
    vscode.postMessage({
      command: 'save',
    });
  });
}

function exportSVG(uri: any) {
  const svg = <SVGSVGElement>document.querySelector('svg')!.cloneNode(true);
  const viewport = svg.querySelector(':scope > g')!;
  const graph = viewport.querySelector(':scope > g')!;

  svg.appendChild(graph);
  svg.removeChild(viewport);

  svg.appendChild(document.getElementById('crabviz_style')!.cloneNode(true));
  svg.insertAdjacentHTML(
    "beforeend",
    "<style>:is(.cell, .edge) { pointer-events: none; }</style>"
  );

  vscode.postMessage({
    command: 'saveSVG',
    uri: uri,
    svg: svg.outerHTML.replaceAll("&nbsp;", "&#160;")
  });
}


