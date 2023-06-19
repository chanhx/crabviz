import * as vscode from 'vscode';

// require('../crabviz/crabviz_bg.wasm');
// import * as crabviz from '../crabviz';

import { graphviz } from '@hpcc-js/wasm';
import { retryCommand } from './utils/command';
import { convertSymbol } from './utils/lspTypesConversion';

// crabviz.set_panic_hook();

export class Generator {
  private root: vscode.Uri;
  // private inner: crabviz.GraphGenerator;

  public constructor(root: vscode.Uri) {
    this.root = root;
    // inner = new crabviz.GraphGenerator(root.path);
  }

  public async generateCallGraph(
    selectedFiles: vscode.Uri[],
    includePattern: vscode.RelativePattern,
    exclude: string
  ): Promise<string> {
    // Help needed: import the module every time the method is called has some performance implications.
    // Any best practices for loading wasm modules in VS Code extension?
    const crabviz = await import('../crabviz');
    const inner = new crabviz.GraphGenerator(this.root.path);

    const files = await vscode.workspace.findFiles(includePattern, exclude);
    const allFiles = new Set(files.concat(selectedFiles));

    for await (const file of allFiles) {
      // retry several times if the LSP server is not ready
      let symbols = await retryCommand<vscode.DocumentSymbol[]>(5, 600, 'vscode.executeDocumentSymbolProvider', file);
      if (symbols === undefined) {
        vscode.window.showErrorMessage(`Document symbol information not available for '${file.fsPath}'`);
        continue;
      }

      let lspSymbols = symbols.map(convertSymbol);
      inner.add_file(file.path, lspSymbols);

      while (symbols.length > 0) {
        for await (const symbol of symbols) {
          if (![vscode.SymbolKind.Function, vscode.SymbolKind.Method, vscode.SymbolKind.Constructor, vscode.SymbolKind.Interface].includes(symbol.kind)) {
            continue;
          }

          let items: vscode.CallHierarchyItem[];
          try {
            items = await vscode.commands.executeCommand<vscode.CallHierarchyItem[]>('vscode.prepareCallHierarchy', file, symbol.selectionRange.start);
          } catch (e) {
            vscode.window.showErrorMessage(`${e}\n${file}\n${symbol.name}`);
            continue;
          }

          for await (const item of items) {
            await vscode.commands.executeCommand<vscode.CallHierarchyIncomingCall[]>('vscode.provideIncomingCalls', item)
              .then(calls => {
                inner.add_incoming_calls(file.path, item.selectionRange.start, calls);
              })
              .then(undefined, err => {
                console.error(err);
              });
          }

          if (symbol.kind === vscode.SymbolKind.Interface) {
            await vscode.commands.executeCommand<vscode.Location[] | vscode.LocationLink[]>('vscode.executeImplementationProvider', file, symbol.selectionRange.start)
              .then(result => {
                if (result.length <= 0) {
                  return;
                }

                let locations: vscode.Location[];
                if (!(result[0] instanceof vscode.Location)) {
                  locations = result.map(l => {
                    let link = l as vscode.LocationLink;
                    return new vscode.Location(link.targetUri, link.targetSelectionRange ?? link.targetRange);
                  });
                } else {
                  locations = result as vscode.Location[];
                }
                inner.add_interface_implementations(file.path, symbol.selectionRange.start, locations);
              })
              .then(undefined, err => {
                console.log(err);
              });
          }
        }

        symbols = symbols.flatMap(symbol => symbol.children);
      }
    }

    const dot = inner.generate_dot_source();

    return graphviz.dot(dot);
  }
}
