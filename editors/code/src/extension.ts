// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import * as lsp_types from 'vscode-languageserver-types';
import * as fs from 'fs';
// const crabviz = await import('../../../pkg/crabviz');
import { graphviz } from "@hpcc-js/wasm";

// wasmFolder("https://cdn.jsdelivr.net/npm/@hpcc-js/wasm/dist");

// const indexPage = require("../content/index.html").default;

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export async function activate(context: vscode.ExtensionContext) {

	// Use the console to output diagnostic information (console.log) and errors (console.error)
	// This line of code will only be executed once when your extension is activated
	console.log('Congratulations, your extension "crabviz" is now active!');


	// The command has been defined in the package.json file
	// Now provide the implementation of the command with registerCommand
	// The commandId parameter must match the command field in package.json
	let disposable = vscode.commands.registerCommand('crabviz.helloWorld', () => {
		// The code you place here will be executed every time your command is executed
		// Display a message box to the user

		// let ext = vscode.window.activeTextEditor?.document.fileName
		generateCallGraph(context);
		vscode.window.showInformationMessage('Hello World from crabviz!');
	});

	context.subscriptions.push(disposable);
}

async function generateCallGraph(context: vscode.ExtensionContext) {
	const crabviz = await import('../../../pkg');
	crabviz.set_panic_hook();
	let generator = new crabviz.GraphGenerator();

	const files = await vscode.workspace.findFiles("**/*.rs", "**/target/**");

	for await (const file of files) {
		let symbols = await vscode.commands.executeCommand<vscode.DocumentSymbol[]>('vscode.executeDocumentSymbolProvider', file);
		if (!symbols) {
			continue;
		}

		let lspSymbols = symbols.map(convertSymbol);
		generator.add_file(file.path, lspSymbols);

		while (symbols.length > 0) {
			for await (const symbol of symbols) {
				if (![vscode.SymbolKind.Function, vscode.SymbolKind.Method, vscode.SymbolKind.Interface].includes(symbol.kind)) {
					continue;
				}

				const items = await vscode.commands.executeCommand<vscode.CallHierarchyItem[]>('vscode.prepareCallHierarchy', file, symbol.selectionRange.start);

				for await (const item of items) {
					await vscode.commands.executeCommand<vscode.CallHierarchyOutgoingCall[]>('vscode.provideOutgoingCalls', item)
						.then(outgoings => {
							generator.add_outgoing_calls(file.path, item.selectionRange.start, outgoings);
						})
						.then(undefined, err => {
							console.error(err);
						});
				}

				if (symbol.kind === vscode.SymbolKind.Interface) {
					await vscode.commands.executeCommand<vscode.LocationLink[]>('vscode.executeImplementationProvider', file, symbol.selectionRange.start)
						.then(links => {
							generator.add_interface_implementations(file.path, symbol.selectionRange.start, links);
						})
						.then(undefined, err => {
							console.log(err);
						});
				}
			}

			symbols = symbols.flatMap(symbol => symbol.children);
		}
	}

	const panel = vscode.window.createWebviewPanel('crabviz', 'crabviz', vscode.ViewColumn.One, {
		localResourceRoots: [
			vscode.Uri.joinPath(context.extensionUri, '..', 'common', 'media')
		],
		enableScripts: true
	});
	const dot = generator.generate_dot_source();

	await activateWebviewPanel(context, panel, dot);
}

async function activateWebviewPanel(context: vscode.ExtensionContext, panel: vscode.WebviewPanel, dot: string) {
	const svg = await graphviz.dot(dot);
	panel.webview.html = getWebviewContent(context, panel.webview, svg);
}

function getWebviewContent(context: vscode.ExtensionContext, webview: vscode.Webview, svg: String) {
	const resourceUri = vscode.Uri.joinPath(context.extensionUri, '..', 'common', 'media');
	const stylesPath = vscode.Uri.joinPath(resourceUri, 'styles.css');
	const preprocessJsPath = vscode.Uri.joinPath(resourceUri, 'preprocess.js');

	const stylesUri = webview.asWebviewUri(stylesPath);
	const preprossessJsUri = webview.asWebviewUri(preprocessJsPath);

	const nonce = getNonce();

  return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
		<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource}; img-src ${webview.cspSource}; script-src 'nonce-${nonce}';">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
		<link rel="stylesheet" href="${stylesUri}">
    <title>crabviz</title>
</head>
<body>
<body>
    ${svg}
		<script nonce="${nonce}" src="${preprossessJsUri}"></script>
</body>
</html>`;
}

function getNonce() {
	let text = '';
	const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
	for (let i = 0; i < 32; i++) {
		text += possible.charAt(Math.floor(Math.random() * possible.length));
	}
	return text;
}

function convertSymbol(symbol: vscode.DocumentSymbol): lsp_types.DocumentSymbol {
	return lsp_types.DocumentSymbol.create(
		symbol.name,
		symbol.detail,
		convertSymbolKind(symbol.kind),
		symbol.range,
		symbol.selectionRange,
		symbol.children.map(convertSymbol),
	);
}

function convertSymbolKind(kind: vscode.SymbolKind): lsp_types.SymbolKind {
	switch (kind) {
		case vscode.SymbolKind.File:
			return lsp_types.SymbolKind.File;
		case vscode.SymbolKind.Module:
			return lsp_types.SymbolKind.Module;
		case vscode.SymbolKind.Namespace:
			return lsp_types.SymbolKind.Namespace;
		case vscode.SymbolKind.Package:
			return lsp_types.SymbolKind.Package;
		case vscode.SymbolKind.Class:
			return lsp_types.SymbolKind.Class;
		case vscode.SymbolKind.Method:
			return lsp_types.SymbolKind.Method;
		case vscode.SymbolKind.Property:
			return lsp_types.SymbolKind.Property;
		case vscode.SymbolKind.Field:
			return lsp_types.SymbolKind.Field;
		case vscode.SymbolKind.Constructor:
			return lsp_types.SymbolKind.Constructor;
		case vscode.SymbolKind.Enum:
			return lsp_types.SymbolKind.Enum;
		case vscode.SymbolKind.Interface:
			return lsp_types.SymbolKind.Interface;
		case vscode.SymbolKind.Function:
			return lsp_types.SymbolKind.Function;
		case vscode.SymbolKind.Variable:
			return lsp_types.SymbolKind.Variable;
		case vscode.SymbolKind.Constant:
			return lsp_types.SymbolKind.Constant;
		case vscode.SymbolKind.String:
			return lsp_types.SymbolKind.String;
		case vscode.SymbolKind.Number:
			return lsp_types.SymbolKind.Number;
		case vscode.SymbolKind.Boolean:
			return lsp_types.SymbolKind.Boolean;
		case vscode.SymbolKind.Array:
			return lsp_types.SymbolKind.Array;
		case vscode.SymbolKind.Object:
			return lsp_types.SymbolKind.Object;
		case vscode.SymbolKind.Key:
			return lsp_types.SymbolKind.Key;
		case vscode.SymbolKind.Null:
			return lsp_types.SymbolKind.Null;
		case vscode.SymbolKind.EnumMember:
			return lsp_types.SymbolKind.EnumMember;
		case vscode.SymbolKind.Struct:
			return lsp_types.SymbolKind.Struct;
		case vscode.SymbolKind.Event:
			return lsp_types.SymbolKind.Event;
		case vscode.SymbolKind.Operator:
			return lsp_types.SymbolKind.Operator;
		case vscode.SymbolKind.TypeParameter:
			return lsp_types.SymbolKind.TypeParameter;
	}
}

// This method is called when your extension is deactivated
export function deactivate() {}
