import * as vscode from 'vscode';
import * as path from 'path';

import { Generator } from './generator';
import { CallGraphPanel } from './webview';
import { groupFileExtensions } from './utils/languages';
import { ignoredExtensions, readIgnoreRules } from './utils/ignores';

// wasmFolder("https://cdn.jsdelivr.net/npm/@hpcc-js/wasm/dist");

export async function activate(context: vscode.ExtensionContext) {
	let disposable = vscode.commands.registerCommand('crabviz.generateCallGraph', async (contextSelection: vscode.Uri, allSelections: vscode.Uri[]) => {
		let cancelled = false;

		// selecting no file is actually selecting the entire workspace
		if (allSelections.length === 0) {
			allSelections.push(contextSelection);
		}

		const ignores = await readIgnoreRules();
		const selectedFiles: vscode.Uri[] = [];

		const folder = vscode.workspace.workspaceFolders!
			.find(folder => contextSelection.path.startsWith(folder.uri.path))!;

		const generator = new Generator(folder.uri);

		let extensions = new Set<string>();

		const scanDirectories = allSelections.map(selection => {
			const ext = path.extname(selection.path).substring(1);
			if (ext.length > 0) {
				selectedFiles.push(selection);
				extensions.add(ext);
				return undefined;
			} else {
				if (!selection.path.startsWith(folder.uri.path)) {
					vscode.window.showErrorMessage("Call graph across multiple workspace folders is not supported");
					return;
				}

				return path.relative(folder.uri.path, selection.path);
			}
		})
		.filter((scanPath): scanPath is string => scanPath !== undefined);

		if (scanDirectories.length > 0) {
			await vscode.window.withProgress({
				location: vscode.ProgressLocation.Notification,
				title: "Detecting project languages",
				cancellable: true
			}, (_, token) => {
				token.onCancellationRequested(() => {
					cancelled = true;
				});
				return collectFileExtensions(folder, scanDirectories, extensions, ignores, token);
			});;

			if (cancelled) {
				return;
			}
		}

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
			// TODO: user input
			return;
		}

		vscode.window.withProgress({
			location: vscode.ProgressLocation.Window,
			title: "Crabviz: Generating call graph",
		}, _ => {
			let paths = scanDirectories.map(dir => extensionsByLanguage[lang].map(ext => dir.length > 0 ? `${dir}/**/*.${ext}` : `**/*.${ext}`).join(','));
			const include = new vscode.RelativePattern(folder, `{${paths.join(',')}}`);
			const exclude = `{${ignores.join(',')}}`;

			return generator.generateCallGraph(selectedFiles, include, exclude);
		})
		.then(svg => {
			const panel = new CallGraphPanel(context.extensionUri);
			panel.showCallGraph(svg);
		});
	});

	context.subscriptions.push(disposable);

	context.subscriptions.push(vscode.commands.registerCommand('crabviz.exportCallGraph', () => {
		CallGraphPanel.currentPanel?.exportSVG();
	}));
}

async function collectFileExtensions(
	folder: vscode.WorkspaceFolder,
	scanDirectories: string[],
	extensions: Set<string>,
	ignores: string[],
	token: vscode.CancellationToken
) {
	let files: vscode.Uri[];

	let paths = scanDirectories.map(dir => dir.length > 0 ? `${dir}/**/*.*` : `**/*.*`);
	const include = new vscode.RelativePattern(folder, `{${paths.join(',')}}`);
	let hiddenFiles: string[] = [];

	while (true) {
		let exclude = `{${Array.from(extensions).concat(ignoredExtensions).map(ext => `**/*.${ext}`).concat(ignores).concat(hiddenFiles).join(',')}}`;

		files = await vscode.workspace.findFiles(include, exclude, 1, token);
		if (files.length <= 0 || token.isCancellationRequested) {
			break;
		}

		const ext = path.extname(files[0].path).substring(1);
		if (ext.length > 0) {
			extensions.add(ext);
		} else {
			let relativePath = path.relative(folder.uri.path, files[0].path);
			hiddenFiles.push(relativePath);
		}
	}
}

// This method is called when your extension is deactivated
export function deactivate() {}
