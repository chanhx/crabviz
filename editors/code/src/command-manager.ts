import * as vscode from 'vscode';
import { extname } from 'path';
import { Ignore } from 'ignore';

import { readIgnores } from './utils/ignore';
import { FileClassifier } from './utils/file-classifier';
import { Generator } from './generator';
import { CallGraphPanel } from './webviewCrabviz/webview';
import { getLanguages } from './utils/languages';


import InteractiveWebviewGenerator from "./webviewInteractive/interactiveWebview";
import PreviewPanel from "./webviewInteractive/previewPanel";
import * as settings from "./settings";

export class CommandManager {
  private context: vscode.ExtensionContext;

	private ignores: Map<string, Ignore>;
	private languages: Map<string, string>;

  // Originally, viz-js was used to render the DOT to static SVG before building the Crabviz webview page
  // Now the DOT is rendered to SVG inside the webview using d3-graphviz in the webview page taken from the project vscode-interactive-graphviz
  private isRenderSvg = false;
  private graphvizView: InteractiveWebviewGenerator;
  private generator!: Generator;

  public constructor(context: vscode.ExtensionContext) {
    this.context = context;
		this.ignores = new Map();
		this.languages = getLanguages();

    this.graphvizView = new InteractiveWebviewGenerator(context, this);
  }

  public async handleCallGraph(contextSelection: vscode.Uri, allSelections: vscode.Uri[]) {
		let cancelled = false;

		// selecting no file is actually selecting the entire workspace
		if (allSelections.length === 0) {
			allSelections.push(contextSelection);
		}

		const root = vscode.workspace.workspaceFolders!
			.find(folder => contextSelection.path.startsWith(folder.uri.path))!;

		const ig = await this.readIgnores(root);

		for (const uri of allSelections) {
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
			token.onCancellationRequested(() => cancelled = true);

			const classifer = new FileClassifier(root.uri.path, this.languages, ig);
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
			location: vscode.ProgressLocation.Notification,
			title: "Visual: Generating call graph",
			cancellable: true,
		}, (progress, token) => {
			token.onCancellationRequested(() => cancelled = true);

      // Generate DOT
			this.generator = new Generator(root.uri, lang);
			return this.generator.generateCallGraph(files.get(lang)!, progress, token);
		})
		.then(([dot, svg, symbolLookup]) => {
			if (cancelled) { return; }

      if (this.isRenderSvg) {
        // Render static SVG in crabviz panel
        const panel = new CallGraphPanel(this.context.extensionUri);
        panel.showCallGraphCrabviz(svg, false, symbolLookup);
      } else {
        this.showCallGraphInteractive(dot);
      }
		});
	}

  public async updateCallGraph() {
    const isTest = true;
    const dot = this.generator.getDot(isTest);
    this.showCallGraphInteractive(dot);
  }

  public showCallGraphInteractive(dot: string) {
    // Render graph in interactive viewview
    const options : {
      document?: vscode.TextDocument,
      uri?: vscode.Uri,
      content?: string,
      callback?: (panel: PreviewPanel) => void,
      allowMultiplePanels?: boolean,
      title?: string,
      search?: any,
      displayColumn?: vscode.ViewColumn | {
        viewColumn: vscode.ViewColumn;
        preserveFocus?: boolean | undefined;
      }
    } = {};

    if (!options.content
      && !options.document
      && !options.uri
      && vscode.window.activeTextEditor?.document) {
      options.document = vscode.window.activeTextEditor.document;
    }

    if (!options.uri && options.document) {
      options.uri = options.document.uri;
    }

    if (typeof options.displayColumn === "object" && options.displayColumn.preserveFocus === undefined) {
      options.displayColumn.preserveFocus = settings.extensionConfig().get("preserveFocus"); // default to user settings
    }

    const execute = (o:any) => { 
      this.graphvizView.revealOrCreatePreview(
        o.displayColumn,
        o.uri,
        o)
      .then((webpanel: PreviewPanel) => {
        // trigger dot render on page load success
        // just in case webpanel takes longer to load, wait for page
        // to ping back and perform action
        // eslint-disable-next-line no-param-reassign
        webpanel.waitingForRendering = o.content;
        // eslint-disable-next-line no-param-reassign
        webpanel.search = o.search;

        // allow caller to handle messages by providing them with the newly created webpanel
        // e.g. caller can override webpanel.handleMessage = function(message){};
        if (o.callback) {
          o.callback(webpanel);
        }
      });
    };

    // Set content and render
    options.content = dot;
    execute(options);
  }
  
  public async handleCallGraphForFunction(editor: vscode.TextEditor) {
		const uri = editor.document.uri;
		const anchor = editor.selection.start;

		const root = vscode.workspace.workspaceFolders!
			.find(folder => uri.path.startsWith(folder.uri.path))!;

		const ig = await this.readIgnores(root);

		const lang = this.languages.get(extname(uri.path)) ?? "";

		this.generator = new Generator(root.uri, lang);

		vscode.window.withProgress({
			location: vscode.ProgressLocation.Window,
			title: "Visual: Generating call graph",
		}, _ => {
			return this.generator.generateFuncCallGraph(uri, anchor, ig);
		})
		.then(svg => {
			if (!svg) {
				vscode.window.showErrorMessage('No results');
				return;
			}

			const panel = new CallGraphPanel(this.context.extensionUri);
			panel.showCallGraphCrabviz(svg, true, {});
		});
	}

	async readIgnores(root: vscode.WorkspaceFolder): Promise<Ignore> {
		if (this.ignores.has(root.uri.path)) {
			return this.ignores.get(root.uri.path)!;
		} else {
			const ig = await readIgnores(root);
			this.ignores.set(root.uri.path, ig);

			return ig;
		}
	}
}
