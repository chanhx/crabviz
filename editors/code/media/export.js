const vscode = acquireVsCodeApi();

function exportSVG() {
  const original = document.getElementsByTagName("svg")[0];
  const cloned = original.cloneNode(true);

  addInlineStyle(original, cloned);

  cloned.removeAttribute('width');
  cloned.removeAttribute("height");

  vscode.postMessage({
    command: 'saveImage',
    svg: cloned.outerHTML
  });
}

function addInlineStyle(original, cloned) {
  const style = getComputedStyle(original);

  for (const prop of style) {
    cloned.style[prop] = style.getPropertyValue(prop);
  }

  // recursively add inline styles

  const children = original.children;
  if (children <= 0) {
    return;
  }
  const clonedChildren = cloned.children;

  for (let i = 0, len = children.length; i < len; ++i) {
    const elem = children[i];
    const clonedElem = clonedChildren[i];
    addInlineStyle(elem, clonedElem);
  }
}

window.addEventListener('message', (e) => {
  const message = e.data;

  switch (message.command) {
    case 'export':
        exportSVG();
        break;
  }
});
