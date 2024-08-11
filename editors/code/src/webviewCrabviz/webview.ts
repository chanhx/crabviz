import * as vscode from 'vscode';
import { retryCommand } from '../utils/command';
import { SymbolsByFileId } from '../utils/symbol-lookup';
import { ViewColumn } from 'vscode';

export class CallGraphPanel {
	public static readonly viewType = 'codevisual.callgraph';

	public static currentPanel: CallGraphPanel | null = null;
	private static num = 1;

	private readonly _panel: vscode.WebviewPanel;
	private readonly _extensionUri: vscode.Uri;
	private _disposables: vscode.Disposable[] = [];

	private symbolLookup: SymbolsByFileId = {};

	public constructor(extensionUri: vscode.Uri) {
		this._extensionUri = extensionUri;

		const panel = vscode.window.createWebviewPanel(CallGraphPanel.viewType, `Visualize #${CallGraphPanel.num}`, vscode.ViewColumn.One, {
			localResourceRoots: [
				vscode.Uri.joinPath(this._extensionUri, 'contentCrab')
			],
			enableScripts: true
		});

		panel.iconPath = vscode.Uri.joinPath(this._extensionUri, 'contentCrab', 'icon.svg');

		this._panel = panel;

		this._panel.webview.onDidReceiveMessage(
			async (message) => {
				console.log('Message received from panel:', message);

				switch (message.command) {
					case 'saveSVG':
						this.saveSVG(message.svg);
						break;

					case 'selectCell':
						await this.editorNavigateToSymbolId(message.symbol);
						break;
				}
			},
			null,
			this._disposables
		);

		this._panel.onDidChangeViewState(
			e => {
				// console.log('Panel onDidChangeViewState');
				
				if (panel.active) {
					CallGraphPanel.currentPanel = this;
				} else if (CallGraphPanel.currentPanel !== this) {
					return;
				} else {
					CallGraphPanel.currentPanel = null;
				}
			},
			null,
			this._disposables
		);

		this._panel.onDidDispose(() => {
			// console.log('Panel onDidDispose');

			this.dispose()
		}, null, this._disposables);


		// Detect changes in active panel
		const statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left);

		vscode.window.onDidChangeActiveTextEditor(editor => { 
			if (editor) {
				statusBarItem.text = `Active: $(file) ${editor.document.fileName}`;
				statusBarItem.show();
			} else {
				statusBarItem.hide();
			}
		});
			
		CallGraphPanel.num += 1;
	}

	public dispose() {
		if (CallGraphPanel.currentPanel === this) {
			CallGraphPanel.currentPanel = null;
		}

		while (this._disposables.length) {
			const x = this._disposables.pop();
			if (x) {
				x.dispose();
			}
		}
	}

	public showCallGraphCrabviz(svg: string, focusMode: boolean, symbolLookup: SymbolsByFileId) {
		const resourceUri = vscode.Uri.joinPath(this._extensionUri, 'contentCrab');

		this.symbolLookup = symbolLookup;

		const filePromises = ['variables.css', 'styles.css', 'graph.js', 'panzoom.min.js', 'export.js', 'vscode.js'].map(fileName =>
			vscode.workspace.fs.readFile(vscode.Uri.joinPath(resourceUri, fileName))
		);

		CallGraphPanel.currentPanel = this;

		const nonce = getNonce();

		Promise.all(filePromises).then(([cssVariables, cssStyles, ...scripts]) => {
			this._panel.webview.html = `<!DOCTYPE html>
			<html lang="en">
			<head>
					<meta charset="UTF-8">
					<meta http-equiv="Content-Security-Policy" content="script-src 'nonce-${nonce}';">
					<meta name="viewport" content="width=device-width, initial-scale=1.0">
					<style id="crabviz_style">
						${cssVariables.toString()}
						${cssStyles.toString()}
					</style>
					${scripts.map((s) => `<script nonce="${nonce}">${s.toString()}</script>`)}
					<title>Visualize</title>
			</head>
			<body data-vscode-context='{ "preventDefaultContextMenuItems": true }'>
					${svg}

					<script nonce="${nonce}">
						const graph = new CallGraph(document.querySelector("svg"), ${focusMode});
						graph.activate();

						panzoom(graph.svg, {
							minZoom: 1,
							smoothScroll: false,
							zoomDoubleClickSpeed: 1
						});
					</script>
			</body>
			</html>`;
		});
	}

	public exportSVG() {
		this._panel.webview.postMessage({ command: 'exportSVG' });
	}

	saveSVG(svg: string) {
		const writeData = Buffer.from(svg, 'utf8');

		vscode.window.showSaveDialog({
			saveLabel: "export",
			filters: { 'Images': ['svg'] },
		}).then((fileUri) => {
			if (fileUri) {
				try {
					vscode.workspace.fs.writeFile(fileUri, writeData)
						.then(() => {
							console.log("File Saved");
						}, (err : any) => {
							vscode.window.showErrorMessage(`Error on writing file: ${err}`);
						});
				} catch (err) {
					vscode.window.showErrorMessage(`Error on writing file: ${err}`);
				}
			}
		});
	}

	public async editorNavigateToSymbolName(symbolName: string) {
		// Navigate to the given symbol by searching the workspace for the symbol
		let symbolsFound = await retryCommand<vscode.SymbolInformation[]>(5, 600, 'vscode.executeWorkspaceSymbolProvider', symbolName);
		console.log('Symbols found matching symbol name:', symbolName, symbolsFound);

		if (symbolsFound && symbolsFound.length > 0) {
			const targetSymbol = symbolsFound[0];
			console.log('Symbol instance:', targetSymbol);

			// Open document containing symbol
			const location = new vscode.Location(targetSymbol.location.uri, targetSymbol.location.range);
			vscode.workspace.openTextDocument(location.uri).then(document => {
				console.log('Symbol document opened:', location.uri);
				
				// Show document and navigation to symbol range
				vscode.window.showTextDocument(document, {
					viewColumn: ViewColumn.Active,
					preserveFocus: false,
					preview: false,
					selection: targetSymbol.location.range
				});
			}, error => {
				vscode.window.showErrorMessage(`Error opening file: ${error}`);
			});
		} else {
			vscode.window.showErrorMessage(`Symbol not found in workspace: ${symbolName}`);
		}
	}

	public async editorNavigateToSymbolId(symbolId: string) {
		// Navigate to the given symbol reference using the symbol lookup
		// Parse the symbol ID to get the file ID and the symbol location within the file
		const partsSymbol = symbolId.split(':');
		if (partsSymbol.length == 2) {
			const fileId = partsSymbol[0];

			const partsLocation = partsSymbol[1].split('_');
			if (partsLocation.length == 2) {
				const line = parseInt(partsLocation[0]);
				const character = parseInt(partsLocation[1]);

				// Lookup the symbol
				const fileSymbols = this.symbolLookup[fileId];
				if (fileSymbols) {
					console.log('Found file in lookup:', fileId, fileSymbols.filePath);

					// Open file containing the symbol
					vscode.workspace.openTextDocument(fileSymbols.filePath).then(document => {
						console.log('File opened:', fileSymbols.filePath);
						
						// Find symbol in lookup
						const symbolFound = fileSymbols.symbols.find(s => 
							s.selectionRange.start.line == line &&
							s.selectionRange.start.character == character);

						if (symbolFound) {
							console.log('Found symbol in lookup:', symbolFound);
							const positionStart = new vscode.Position(symbolFound.selectionRange.start.line, symbolFound.selectionRange.start.character);
							const positionEnd = new vscode.Position(symbolFound.selectionRange.end.line, symbolFound.selectionRange.end.character);
							
							// Show the text file containing the symbol and select the symbol
							vscode.window.showTextDocument(document, {
								viewColumn: ViewColumn.Active,
								preserveFocus: false,
								preview: false,
								selection: new vscode.Range(positionStart, positionEnd)
							});
						}
					}, error => {
						vscode.window.showErrorMessage(`Error opening file: ${error}`);
					});
				} else {
					vscode.window.showErrorMessage(`Document symbol not found in the lookup table`);
				}
			}
		}
	}

	public editorFindWithPath(path: string): vscode.TextEditor | undefined {
		return vscode.window.visibleTextEditors.find(editor => editor.document.uri.fsPath === path);		
	}

	public editorRevealRange(editor: vscode.TextEditor, range: vscode.Range) {
		editor.revealRange(range, vscode.TextEditorRevealType.InCenter);	
	}
}

function getNonce() {
	let text = '';
	const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
	for (let i = 0; i < 32; i++) {
		text += possible.charAt(Math.floor(Math.random() * possible.length));
	}
	return text;
}
