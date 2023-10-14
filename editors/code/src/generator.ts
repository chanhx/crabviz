import * as vscode from 'vscode';

import { graphviz } from '@hpcc-js/wasm';
import { retryCommand } from './utils/command';
import { convertSymbol } from './utils/lspTypesConversion';
import { set_panic_hook, GraphGenerator  } from '../crabviz';

await import('../crabviz');

set_panic_hook();

const FUNC_KINDS: readonly vscode.SymbolKind[] = [vscode.SymbolKind.Function, vscode.SymbolKind.Method, vscode.SymbolKind.Constructor];

export class Generator {
  private root: string;
  private inner: GraphGenerator;

  public constructor(root: vscode.Uri) {
    this.root = root.path;
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
              await this.resolveCallsInFiles(item, funcMap);
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

  async generateFuncCallGraph(uri: vscode.Uri, anchor: vscode.Position): Promise<string | null> {
    const files = new Map<string, VisitedFile>();

    let items: vscode.CallHierarchyItem[];
    try {
      items = await vscode.commands.executeCommand<vscode.CallHierarchyItem[]>('vscode.prepareCallHierarchy', uri, anchor);
    } catch (e) {
      vscode.window.showErrorMessage(`${e}`);
      return null;
    }

    if (items.length <= 0) {
      return null;
    }

    for await (const item of items) {
      await this.resolveIncomingCalls(item, files);
      await this.resolveOutgoingCalls(item, files);
    }

    for await (const file of files.values()) {
      let symbols = await retryCommand<vscode.DocumentSymbol[]>(5, 600, 'vscode.executeDocumentSymbolProvider', file.uri);
      if (symbols === undefined) {
        // vscode.window.showErrorMessage(`Document symbol information not available for '${file.uri.fsPath}'`);
        continue;
      }

      const funcs = file.sortedFuncs();
      const lspSymbols = this.filterSymbols(symbols, funcs, 0, true).map(convertSymbol);

      this.inner.add_file(file.uri.path, lspSymbols);
    }

    const dot = this.inner.generate_dot_source();

    return graphviz.dot(dot);
  }

  filterSymbols(symbols: vscode.DocumentSymbol[], funcsPos: vscode.Position[], i: number, top: boolean): vscode.DocumentSymbol[] {
    let funcPos = funcsPos[i];

    return symbols.filter(symbol => {
      const keep =
        i < funcsPos.length &&
        (symbol.range.start.line < funcPos.line || (symbol.range.start.line === funcPos.line && symbol.range.start.character <= funcPos.character)) &&
        (symbol.range.end.line > funcPos.line || (symbol.range.end.line === funcPos.line && symbol.range.end.character >= funcPos.character));

      if (keep) {
        symbol.children = this.filterSymbols(symbol.children, funcsPos, i, false);

        if (top) {
          funcPos = funcsPos[++i];
        }
      }

      return keep;
    });
  }

  async resolveCallsInFiles(item: vscode.CallHierarchyItem, funcMap: Map<string, Set<string>>) {
    await vscode.commands.executeCommand<vscode.CallHierarchyIncomingCall[]>('vscode.provideIncomingCalls', item)
      .then(async calls => {
        const symbolStart = item.selectionRange.start;
        this.inner.add_incoming_calls(item.uri.path, symbolStart, calls);

        funcMap.get(item.uri.path)?.add(`${symbolStart}`);

        calls = calls
          .filter(call => {
            const funcs = funcMap.get(call.from.uri.path);
            return funcs !== undefined && !funcs.has(`${call.from.selectionRange.start}`);
          });

        for await (const call of calls) {
          await this.resolveCallsInFiles(call.from, funcMap);
        }
      })
      .then(undefined, err => {
        console.error(err);
      });
  }

  async resolveIncomingCalls(item: vscode.CallHierarchyItem, funcMap: Map<string, VisitedFile>) {
    await vscode.commands.executeCommand<vscode.CallHierarchyIncomingCall[]>('vscode.provideIncomingCalls', item)
      .then(async calls => {
        const symbolStart = item.selectionRange.start;
        this.inner.add_incoming_calls(item.uri.path, symbolStart, calls);

        let file = funcMap.get(item.uri.path);
        if (!file) {
          file = new VisitedFile(item.uri);
          funcMap.set(item.uri.path, file);
        }
        file.addFunc(symbolStart);

        if (file.incoming) {
          return;
        }
        file.incoming = true;

        calls = calls
          .filter(call =>
            !funcMap.get(call.from.uri.path)?.hasFunc(call.from.selectionRange.start) ?? true
          );

        for await (const call of calls) {
          await this.resolveIncomingCalls(call.from, funcMap);
        }
      })
      .then(undefined, err => {
        console.error(err);
      });
  }

  async resolveOutgoingCalls(item: vscode.CallHierarchyItem, funcMap: Map<string, VisitedFile>) {
    await vscode.commands.executeCommand<vscode.CallHierarchyOutgoingCall[]>('vscode.provideOutgoingCalls', item)
      .then(async calls => {
        const symbolStart = item.selectionRange.start;
        this.inner.add_outgoing_calls(item.uri.path, symbolStart, calls);

        let file = funcMap.get(item.uri.path);
        if (!file) {
          file = new VisitedFile(item.uri);
          funcMap.set(item.uri.path, file);
        }
        file.addFunc(symbolStart);

        if (file.outgoing) {
          return;
        }
        file.outgoing = true;

        calls = calls
          .filter(call =>
            call.to.uri.path.startsWith(this.root) &&
            (!funcMap.get(call.to.uri.path)?.hasFunc(call.to.selectionRange.start) ?? true)
          );

        for await (const call of calls) {
          await this.resolveOutgoingCalls(call.to, funcMap);
        }
      })
      .then(undefined, err => {
        console.error(err);
      });
  }
}

class VisitedFile {
  uri: vscode.Uri;
  incoming: boolean;
  outgoing: boolean;
  private funcs: Map<string, vscode.Position>;

  constructor(uri: vscode.Uri) {
    this.uri = uri;
    this.incoming = false;
    this.outgoing = false;
    this.funcs = new Map();
  }

  addFunc(pos: vscode.Position) {
    this.funcs.set(`${pos}`, pos);
  }

  hasFunc(pos: vscode.Position): boolean {
    return this.funcs.has(`${pos}`);
  }

  sortedFuncs(): vscode.Position[] {
    const funcs = Array.from(this.funcs.values());
    return funcs.sort((p1, p2) => {
      const lineDiff = p1.line - p2.line;
      if (lineDiff !== 0) {
        return lineDiff;
      } else {
        return p1.character - p2.character;
      }
    });
  }
};

function hasFunc(funcMap: Map<string, Set<string>>, filePath: string, position: vscode.Position): boolean {
  return funcMap.get(filePath)?.has(`${position}`) ?? false;
}
