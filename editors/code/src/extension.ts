import * as vscode from 'vscode';
import { extname, relative } from 'path';

import { initSync, set_panic_hook } from '../crabviz';

import { Generator } from './generator';
import { CallGraphPanel } from './webview';
import { groupFileExtensions } from './utils/languages';
import { ignoredExtensions, readIgnoreRules } from './utils/ignores';

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

		const ignores = await readIgnoreRules();
		const selectedFiles: vscode.Uri[] = [];

		const folder = vscode.workspace.workspaceFolders!
			.find(folder => contextSelection.path.startsWith(folder.uri.path))!;

		let extensions = new Set<string>();

		const scanDirectories = allSelections.map(selection => {
			const ext = extname(selection.path).substring(1);
			if (ext.length > 0) {
				selectedFiles.push(selection);
				extensions.add(ext);
				return undefined;
			} else {
				if (!selection.path.startsWith(folder.uri.path)) {
					vscode.window.showErrorMessage("Call graph across multiple workspace folders is not supported");
					return;
				}

				return relative(folder.uri.path, selection.path);
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

		const generator = new Generator(folder.uri, extensionsByLanguage[lang][0]);

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
			panel.showCallGraph(svg, false);
		});
	});

	context.subscriptions.push(disposable);

	context.subscriptions.push(vscode.commands.registerTextEditorCommand('crabviz.generateFuncCallGraph', editor => {
		const uri = editor.document.uri;
		const anchor = editor.selection.start;

		const folder = vscode.workspace.workspaceFolders!
			.find(folder => uri.path.startsWith(folder.uri.path))!;

		const ext = extname(uri.path).substring(1);

		const generator = new Generator(folder.uri, ext);

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

		const ext = extname(files[0].path).substring(1);
		if (ext.length > 0) {
			extensions.add(ext);
		} else {
			let relativePath = relative(folder.uri.path, files[0].path);
			hiddenFiles.push(relativePath);
		}
	}
}

// This method is called when your extension is deactivated
export function deactivate() {}
