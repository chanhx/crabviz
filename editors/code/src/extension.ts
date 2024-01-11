import * as vscode from 'vscode';
import { extname, relative } from 'path';

import { initSync, set_panic_hook } from '../crabviz';

import { Generator } from './generator';
import { CallGraphPanel } from './webview';
import { groupFileExtensions } from './utils/languages';
import { collectFileExtensions, ignoreManager } from './utils/workspace';

export async function activate(context: vscode.ExtensionContext) {
	const bits = await vscode.workspace.fs.readFile(
		vscode.Uri.joinPath(context.extensionUri, './crabviz/index_bg.wasm')
	);

	initSync(bits);

	set_panic_hook();

	let disposable = vscode.commands.registerCommand('crabviz.generateCallGraph', async (contextSelection: vscode.Uri, allSelections: vscode.Uri[]) => {
		let cancelled = false;

		// selecting no file is actually selecting the entire workspace
		if (allSelections.length === 0) {
			allSelections.push(contextSelection);
		}

		const root = vscode.workspace.workspaceFolders!
			.find(folder => contextSelection.path.startsWith(folder.uri.path))!;

		const ig = await ignoreManager(root);

		// separate directories and files in selections, and collect selected files' extension

		let selectedDirectories: vscode.Uri[] = [];
		let selectedFiles: vscode.Uri[] = [];
		let extensions = new Set<string>();

		for await (const uri of allSelections) {
			if (!uri.path.startsWith(root.uri.path)) {
				vscode.window.showErrorMessage("Can not generate call graph across multiple workspace folders");
				return;
			}

			let fileType = (await vscode.workspace.fs.stat(uri)).type;

			if ((fileType & vscode.FileType.Directory) === vscode.FileType.Directory) {
				selectedDirectories.push(uri);
			} else if ((fileType & vscode.FileType.File) === vscode.FileType.File) {
				selectedFiles.push(uri);

				const ext = extname(uri.path).substring(1);
				extensions.add(ext);
			}
		}

		// collect file extensions in selected directories

		if (selectedDirectories.length > 0) {
			extensions = await vscode.window.withProgress({
				location: vscode.ProgressLocation.Notification,
				title: "Detecting project languages",
				cancellable: true
			}, (_, token) => {
				token.onCancellationRequested(() => {
					cancelled = true;
				});
				return collectFileExtensions(root.uri.path, selectedDirectories, ig, token);
			}).then(newExtensions => {
				extensions.forEach(ext => {
					newExtensions.add(ext);
				});

				return newExtensions;
			});

			if (cancelled) {
				return;
			}
		}

		// detect programming languages from file extensions

		const extensionsByLanguage = groupFileExtensions(extensions);

		const selections = Object.keys(extensionsByLanguage).map(lang => ({ label: lang }));
		let lang: string;
		if (selections.length > 1) {
			const selectedItem = await vscode.window.showQuickPick(selections, {
				title: "Pick a language to generate call graph",
			});

			if (!selectedItem) {
				return;
			}
			lang = selectedItem.label;
		} else if (selections.length === 1) {
			lang = selections[0].label;
		} else {
			return;
		}

		vscode.window.withProgress({
			location: vscode.ProgressLocation.Window,
			title: "Crabviz: Generating call graph",
		}, _ => {
			const generator = new Generator(root.uri, extensionsByLanguage[lang][0]);
			const fileExtensions = new Set(extensionsByLanguage[lang]);

			return generator.generateCallGraph(selectedFiles, selectedDirectories, fileExtensions, ig);
		})
		.then(svg => {
			const panel = new CallGraphPanel(context.extensionUri);
			panel.showCallGraph(svg, false);
		});
	});

	context.subscriptions.push(disposable);

	context.subscriptions.push(vscode.commands.registerTextEditorCommand('crabviz.generateFuncCallGraph', editor => {
		const uri = editor.document.uri;
		const anchor = editor.selection.start;

		const root = vscode.workspace.workspaceFolders!
			.find(folder => uri.path.startsWith(folder.uri.path))!;

		const ext = extname(uri.path).substring(1);

		const generator = new Generator(root.uri, ext);

		vscode.window.withProgress({
			location: vscode.ProgressLocation.Window,
			title: "Crabviz: Generating call graph",
		}, _ => {
			return generator.generateFuncCallGraph(uri, anchor);
		})
		.then(svg => {
			if (!svg) {
				vscode.window.showErrorMessage('No results');
				return;
			}

			const panel = new CallGraphPanel(context.extensionUri);
			panel.showCallGraph(svg, true);
		});
	}));

	context.subscriptions.push(vscode.commands.registerCommand('crabviz.exportCallGraph', () => {
		CallGraphPanel.currentPanel?.exportSVG();
	}));
}

// This method is called when your extension is deactivated
export function deactivate() {}
