import * as vscode from 'vscode';

import { initSync, set_panic_hook } from '../crabviz';
import { CallGraphPanel } from './webview';
import { CommandManager } from './command-manager';

export async function activate(context: vscode.ExtensionContext) {
	await vscode.workspace.fs.readFile(
		vscode.Uri.joinPath(context.extensionUri, 'crabviz/index_bg.wasm')
	).then(bits => {
		initSync(bits);
		set_panic_hook();
	});

	let manager = new CommandManager(context);

	context.subscriptions.push(
		vscode.commands.registerCommand('crabviz.generateCallGraph', manager.generateCallGraph.bind(manager)),
		vscode.commands.registerTextEditorCommand('crabviz.generateFuncCallGraph', manager.generateFuncCallGraph.bind(manager)),
		vscode.commands.registerCommand('crabviz.save', () => {
			CallGraphPanel.currentPanel?.save();
		}),
	);
}

// This method is called when your extension is deactivated
export function deactivate() {}
