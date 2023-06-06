import * as vscode from 'vscode';

export function showCallGraph(context: vscode.ExtensionContext, svg: string) {
	const panel = vscode.window.createWebviewPanel('crabviz', 'crabviz', vscode.ViewColumn.One, {
		localResourceRoots: [
			vscode.Uri.joinPath(context.extensionUri, 'media')
		],
		enableScripts: true
	});

	panel.iconPath = vscode.Uri.joinPath(context.extensionUri, 'media', 'icon.svg');
	panel.webview.html = renderContent(context, panel.webview, svg);
}

function renderContent(context: vscode.ExtensionContext, webview: vscode.Webview, svg: String) {
	const resourceUri = vscode.Uri.joinPath(context.extensionUri, 'media');

	const stylesPath = vscode.Uri.joinPath(resourceUri, 'styles.css');
	const preprocessJsPath = vscode.Uri.joinPath(resourceUri, 'preprocess.js');
	const svgPanZoomJsPath = vscode.Uri.joinPath(resourceUri, 'svg-pan-zoom.min.js');

	const stylesUri = webview.asWebviewUri(stylesPath);
	const preprossessJsUri = webview.asWebviewUri(preprocessJsPath);
	const svgPanZoomJsUri = webview.asWebviewUri(svgPanZoomJsPath);

	const nonce = getNonce();

  return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
		<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource}; img-src ${webview.cspSource}; script-src 'nonce-${nonce}';">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
		<link rel="stylesheet" href="${stylesUri}">
		<script nonce="${nonce}" src="${svgPanZoomJsUri}"></script>
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