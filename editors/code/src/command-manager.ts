import * as vscode from 'vscode';
import { extname } from 'path';
import { Ignore } from 'ignore';

import { readIgnores } from './utils/ignore';
import { FileClassifier } from './utils/file-classifier';
import { Generator } from './generator';
import { CallGraphPanel } from './webview';
import { languagesByExtension } from './utils/languages';

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
			ig = await readIgnores(root);
			this.ignores.set(root.uri.path, ig);
		}

		for await (const uri of allSelections) {
			if (!uri.path.startsWith(root.uri.path)) {
				vscode.window.showErrorMessage("Can not generate call graph across multiple workspace folders");
				return;
			}
		}

		// classify files by programming language

		const files = await vscode.window.withProgress({
			location: vscode.ProgressLocation.Notification,
			title: "Detecting project languages",
			cancellable: true
		}, (_, token) => {
			token.onCancellationRequested(() => {
				cancelled = true;
			});
			const classifer = new FileClassifier(root.uri.path, ig);
			return classifer.classifyFilesByLanguage(allSelections, token);
		});

		if (cancelled) {
			return;
		}

		const languages = Array.from(files.keys()).map(lang => ({ label: lang }));
		let lang: string;
		if (languages.length > 1) {
			const selectedItem = await vscode.window.showQuickPick(languages, {
				title: "Pick a language to generate call graph",
			});

			if (!selectedItem) {
				return;
			}
			lang = selectedItem.label;
		} else if (languages.length === 1) {
			lang = languages[0].label;
		} else {
			return;
		}

		vscode.window.withProgress({
			location: vscode.ProgressLocation.Window,
			title: "Crabviz: Generating call graph",
		}, _ => {
			const generator = new Generator(root.uri, lang);

			return generator.generateCallGraph(files.get(lang)!);
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
		const lang = languagesByExtension[ext];

		const generator = new Generator(root.uri, lang);

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
