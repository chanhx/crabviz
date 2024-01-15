import * as vscode from 'vscode';
import { extname } from 'path';
import { Ignore } from 'ignore';

import { collectFileExtensions, ignoreManager } from './utils/workspace';
import { groupFileExtensions } from './utils/languages';
import { Generator } from './generator';
import { CallGraphPanel } from './webview';

export class CommandManager {
  private context: vscode.ExtensionContext;

	// TODO: listen to .gitignore file modifications
	private ignores: Map<string, Ignore>;

  public constructor(context: vscode.ExtensionContext) {
    this.context = context;
		this.ignores = new Map();
  }

  public async generateCallGraph(contextSelection: vscode.Uri, allSelections: vscode.Uri[]) {
		let cancelled = false;

		// selecting no file is actually selecting the entire workspace
		if (allSelections.length === 0) {
			allSelections.push(contextSelection);
		}

		const root = vscode.workspace.workspaceFolders!
			.find(folder => contextSelection.path.startsWith(folder.uri.path))!;

		let ig: Ignore;
		if (this.ignores.has(root.uri.path)) {
			ig = this.ignores.get(root.uri.path)!;
		} else {
			ig = await ignoreManager(root);
			this.ignores.set(root.uri.path, ig);
		}

		// separate directories and files in selections, and collect file extensions of selected files

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
			const panel = new CallGraphPanel(this.context.extensionUri);
			panel.showCallGraph(svg, false);
		});
	}

  public generateFuncCallGraph(editor: vscode.TextEditor) {
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

			const panel = new CallGraphPanel(this.context.extensionUri);
			panel.showCallGraph(svg, true);
		});
	}
}
