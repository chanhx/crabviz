import * as vscode from 'vscode';

import { graphviz } from '@hpcc-js/wasm';
import { retryCommand } from './utils/command';
import { convertSymbol } from './utils/lspTypesConversion';
import { set_panic_hook, GraphGenerator  } from '../crabviz';

await import('../crabviz');

set_panic_hook();

const FUNC_KINDS: readonly vscode.SymbolKind[] = [vscode.SymbolKind.Function, vscode.SymbolKind.Method, vscode.SymbolKind.Constructor];

export class Generator {
  private inner: GraphGenerator;

  public constructor(root: vscode.Uri) {
    this.inner = new GraphGenerator(root.path);
  }

  public async generateCallGraph(
    selectedFiles: vscode.Uri[],
    includePattern: vscode.RelativePattern,
    exclude: string
  ): Promise<string> {
    const files = await vscode.workspace.findFiles(includePattern, exclude);
    const allFiles = new Set(files.concat(selectedFiles));

    const sortedFiles = Array.from(allFiles);
    sortedFiles.sort((f1, f2) => f2.path.split('/').length - f1.path.split('/').length);

    const funcMap = new Map<string, Set<string>>(sortedFiles.map(f => [f.path, new Set()]));

    for await (const file of sortedFiles) {
      // retry several times if the LSP server is not ready
      let symbols = await retryCommand<vscode.DocumentSymbol[]>(5, 600, 'vscode.executeDocumentSymbolProvider', file);
      if (symbols === undefined) {
        vscode.window.showErrorMessage(`Document symbol information not available for '${file.fsPath}'`);
        continue;
      }

      let lspSymbols = symbols.map(convertSymbol);
      this.inner.add_file(file.path, lspSymbols);

      while (symbols.length > 0) {
        for await (const symbol of symbols) {
          const symbolStart = symbol.selectionRange.start;

          if (FUNC_KINDS.includes(symbol.kind) && !hasFunc(funcMap, file.path, symbolStart)) {
            let items: vscode.CallHierarchyItem[];
            try {
              items = await vscode.commands.executeCommand<vscode.CallHierarchyItem[]>('vscode.prepareCallHierarchy', file, symbolStart);
            } catch (e) {
              vscode.window.showErrorMessage(`${e}\n${file}\n${symbol.name}`);
              continue;
            }

            for await (const item of items) {
              await this.resolveCalls(item, funcMap);
            }
          } else if (symbol.kind === vscode.SymbolKind.Interface) {
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
                this.inner.add_interface_implementations(file.path, symbol.selectionRange.start, locations);
              })
              .then(undefined, err => {
                console.log(err);
              });
          }
        }

        symbols = symbols.flatMap(symbol => symbol.children);
      }
    }

    const dot = this.inner.generate_dot_source();

    return graphviz.dot(dot);
  }

  async resolveCalls(item: vscode.CallHierarchyItem, funcMap: Map<string, Set<string>>) {
    await vscode.commands.executeCommand<vscode.CallHierarchyIncomingCall[]>('vscode.provideIncomingCalls', item)
      .then(calls => {
        const symbolStart = item.selectionRange.start;
        this.inner.add_incoming_calls(item.uri.path, symbolStart, calls);

        funcMap.get(item.uri.path)?.add(`${symbolStart}`);

        calls
          .filter(call => {
            const funcs = funcMap.get(call.from.uri.path);
            return funcs !== undefined && !funcs.has(`${call.from.selectionRange.start}`);
          })
          .forEach(async call => await this.resolveCalls(call.from, funcMap));
      })
      .then(undefined, err => {
        console.error(err);
      });
  }
}

function hasFunc(funcMap: Map<string, Set<string>>, filePath: string, position: vscode.Position): boolean {
  return funcMap.get(filePath)?.has(`${position}`) ?? false;
}
