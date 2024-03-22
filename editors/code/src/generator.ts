import * as vscode from 'vscode';
import { instance as vizInstance } from '@viz-js/viz';

import { retryCommand } from './utils/command';
import { GraphGenerator } from '../crabviz';
import { Ignore } from 'ignore';
import * as path from "path";

const FUNC_KINDS: readonly vscode.SymbolKind[] = [vscode.SymbolKind.Function, vscode.SymbolKind.Method, vscode.SymbolKind.Constructor];

const viz = vizInstance();
const renderOptions = {format: "svg"};

const isWindows = process.platform === 'win32';

export class Generator {
  private root: string;
  private inner: GraphGenerator;

  public constructor(root: vscode.Uri, lang: string) {
    this.root = normalizedPath(root.path);
    this.inner = new GraphGenerator(this.root, lang);
  }

  public async generateCallGraph(
    files: vscode.Uri[],
    progress: vscode.Progress<{ message?: string; increment?: number }>,
    token: vscode.CancellationToken,
  ): Promise<string> {
    files.sort((f1, f2) => f2.path.split('/').length - f1.path.split('/').length);

    const funcMap = new Map<string, Set<string>>(files.map(f => [normalizedPath(f.path), new Set()]));

    let finishedCount = 0;
    progress.report({ message: `${finishedCount} / ${files.length}` });

    for await (const file of files) {
      if (token.isCancellationRequested) {
        return "";
      }

      // retry several times if the LSP server is not ready
      let symbols = await retryCommand<vscode.DocumentSymbol[]>(5, 600, 'vscode.executeDocumentSymbolProvider', file);
      if (symbols === undefined) {
        vscode.window.showErrorMessage(`Document symbol information not available for '${file.fsPath}'`);
        continue;
      }

      const filePath = normalizedPath(file.path);

      if (!this.inner.add_file(filePath, symbols)) {
        finishedCount += 1;
        progress.report({ message: `${finishedCount} / ${files.length}`, increment: 100 / files.length });
        continue;
      }

      while (symbols.length > 0) {
        for await (const symbol of symbols) {
          if (token.isCancellationRequested) {
            return "";
          }

          const symbolStart = symbol.selectionRange.start;

          if (FUNC_KINDS.includes(symbol.kind) && !hasFunc(funcMap, filePath, symbolStart)) {
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
                this.inner.add_interface_implementations(filePath, symbol.selectionRange.start, locations);
              })
              .then(undefined, err => {
                console.log(err);
              });
          }
        }

        symbols = symbols.flatMap(symbol => symbol.children);
      }

      finishedCount += 1;
      progress.report({ message: `${finishedCount} / ${files.length}`, increment: 100 / files.length });
    }

    const dot = this.inner.generate_dot_source();

    return await viz.then(viz => viz.renderString(dot, renderOptions));
  }

  async generateFuncCallGraph(uri: vscode.Uri, anchor: vscode.Position, ig: Ignore): Promise<string | null> {
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
      files.set(item.uri.path, new VisitedFile(item.uri));

      await this.resolveIncomingCalls(item, files, ig);
      await this.resolveOutgoingCalls(item, files, ig);
    }

    for await (const file of files.values()) {
      if (file.skip) { continue; }

      let symbols = await retryCommand<vscode.DocumentSymbol[]>(5, 600, 'vscode.executeDocumentSymbolProvider', file.uri);
      if (symbols === undefined) {
        // vscode.window.showErrorMessage(`Document symbol information not available for '${file.uri.fsPath}'`);
        continue;
      }

      const funcs = file.sortedFuncs().filter(rng => !rng.isEmpty);
      symbols = this.filterSymbols(symbols, funcs);

      this.inner.add_file(normalizedPath(file.uri.path), symbols);
    }

    for await (const item of items) {
      this.inner.highlight(item.uri.path, item.selectionRange.start);
    }

    const dot = this.inner.generate_dot_source();

    return await viz.then(viz => viz.renderString(dot, renderOptions));
  }

  filterSymbols(symbols: vscode.DocumentSymbol[], funcs: vscode.Range[], ctx = { i: 0 }): vscode.DocumentSymbol[] {
    return symbols
      .sort((s1, s2) => s1.selectionRange.start.compareTo(s2.selectionRange.start))
      .filter(symbol => {
        const keep = ctx.i < funcs.length && symbol.range.contains(funcs[ctx.i]);
        if (!keep) {
          return keep;
        }

        if (symbol.selectionRange.isEqual(funcs[ctx.i])) {
          ctx.i += 1;
          if (ctx.i === funcs.length || !symbol.range.contains(funcs[ctx.i])) {
            symbol.children = [];
            return keep;
          }
        }

        if (symbol.children.length > 0) {
          symbol.children = this.filterSymbols(symbol.children, funcs, ctx);
        }

        return keep;
      });
  }

  async resolveCallsInFiles(item: vscode.CallHierarchyItem, funcMap: Map<string, Set<string>>) {
    await vscode.commands.executeCommand<vscode.CallHierarchyIncomingCall[]>('vscode.provideIncomingCalls', item)
      .then(async calls => {
        const symbolStart = item.selectionRange.start;
        this.inner.add_incoming_calls(item.uri.path, symbolStart, calls);

        funcMap.get(item.uri.path)?.add(keyFromPosition(symbolStart));

        calls = calls
          .filter(call => {
            const funcs = funcMap.get(call.from.uri.path);
            return funcs !== undefined && !funcs.has(keyFromPosition(call.from.selectionRange.start));
          });

        for await (const call of calls) {
          await this.resolveCallsInFiles(call.from, funcMap);
        }
      })
      .then(undefined, err => {
        console.error(err);
      });
  }

  async resolveIncomingCalls(item: vscode.CallHierarchyItem, funcMap: Map<string, VisitedFile>, ig: Ignore) {
    await vscode.commands.executeCommand<vscode.CallHierarchyIncomingCall[]>('vscode.provideIncomingCalls', item)
      .then(async calls => {
        this.inner.add_incoming_calls(item.uri.path, item.selectionRange.start, calls);
        funcMap.get(item.uri.path)!.visitFunc(item.selectionRange, FuncCallDirection.Incoming);

        calls = calls
          .filter(call => {
            const uri = call.from.uri;

            let file = funcMap.get(uri.path);
            if (!file) {
              file = new VisitedFile(uri);
              file.skip = ig.ignores(path.relative(this.root, uri.path)) || this.inner.should_filter_out_file(uri.path);
              funcMap.set(uri.path, file);
            }

            return !file.skip && !file.hasVisitedFunc(call.from.selectionRange.start, FuncCallDirection.Incoming);
          });

        for await (const call of calls) {
          await this.resolveIncomingCalls(call.from, funcMap, ig);
        }
      })
      .then(undefined, err => {
        console.error(err);
      });
  }

  async resolveOutgoingCalls(item: vscode.CallHierarchyItem, funcMap: Map<string, VisitedFile>, ig: Ignore) {
    await vscode.commands.executeCommand<vscode.CallHierarchyOutgoingCall[]>('vscode.provideOutgoingCalls', item)
      .then(async calls => {
        this.inner.add_outgoing_calls(item.uri.path, item.selectionRange.start, calls);
        funcMap.get(item.uri.path)!.visitFunc(item.selectionRange, FuncCallDirection.Outgoing);

        calls = calls
          .filter(call => {
            if (!call.to.uri.path.startsWith(this.root)) {
              return false;
            }

            const uri = call.to.uri;

            let file = funcMap.get(uri.path);
            if (!file) {
              file = new VisitedFile(uri);
              file.skip = ig.ignores(path.relative(this.root, uri.path)) || this.inner.should_filter_out_file(uri.path);
              funcMap.set(uri.path, file);
            }

            return !file.skip && !file.hasVisitedFunc(call.to.selectionRange.start, FuncCallDirection.Outgoing);
          });

        for await (const call of calls) {
          await this.resolveOutgoingCalls(call.to, funcMap, ig);
        }
      })
      .then(undefined, err => {
        console.error(err);
      });
  }
}

enum FuncCallDirection {
  Incoming = 1 << 1,
  Outgoing = 1 << 2,
  Bidirection = Incoming | Outgoing,
}

class VisitedFile {
  uri: vscode.Uri;
  skip: boolean;
  private funcs: Map<string, [vscode.Range, FuncCallDirection]>;

  constructor(uri: vscode.Uri) {
    this.uri = uri;
    this.skip = false;
    this.funcs = new Map();
  }

  visitFunc(rng: vscode.Range, direction: FuncCallDirection) {
    let key = keyFromPosition(rng.start);
    let val = this.funcs.get(key);

    if (!val) {
      this.funcs.set(key, [rng, direction]);
    } else {
      val[1] |= direction;
    }
  }

  hasVisitedFunc(pos: vscode.Position, direction: FuncCallDirection): boolean {
    return ((this.funcs.get(keyFromPosition(pos))?.[1] ?? 0) & direction) === direction;
  }

  sortedFuncs(): vscode.Range[] {
    const funcs = Array.from(this.funcs.values());
    return funcs
            .sort((p1, p2) => p1[0].start.compareTo(p2[0].start))
            .map(tuple => tuple[0]);
  }
};

function hasFunc(funcMap: Map<string, Set<string>>, filePath: string, position: vscode.Position): boolean {
  return funcMap.get(filePath)?.has(keyFromPosition(position)) ?? false;
}

function keyFromPosition(pos: vscode.Position): string {
  return `${pos.line} ${pos.character}`;
}

// In Windows, the drive letter cases are not consistent in paths returned from APIs and commands.
// According to the docs, we should use `fsPath` rather than `path` for consistency, but there would be some other issues (in rust part) if so.
// So here we normalize `path` to upper-case drive letters.
function normalizedPath(path: string): string {
  return isWindows ? path.replace(/^\/\w+(?=:)/, drive => drive.toUpperCase()) : path;
}
