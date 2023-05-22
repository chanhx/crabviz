import * as vscode from 'vscode';
import * as path from 'path';

export const ignoredExtensions = [
  'css',
  'csv',
  'gif',
  'gitignore',
  'html',
  'md',
  'jpeg',
  'jpg',
  'json',
  'lex',
  'lock',
  'log',
  'png',
  'toml',
  'txt',
  'xml',
  'yaml',
  'yacc',
  'yml',
];

export async function readIgnoreRules(): Promise<string[]> {
	let excludes: string[] = [];

	const folders = vscode.workspace.workspaceFolders;
	if (!folders) {
		return [];
	}

	for (const folder of folders) {
		const ignores = await vscode.workspace.findFiles(new vscode.RelativePattern(folder, '**/.gitignore'));

		for (const ignore of ignores) {
			const dir = path.dirname(ignore.path);
			const relativePath = path.relative(folder.uri.path, dir);

			const rules = await vscode.workspace.fs.readFile(ignore)
				.then(content => content.toString()
				.split('\n')
				.filter(rule => rule.trim().length > 0));
			excludes = excludes.concat(rules.map(rule => {
				if (rule.startsWith("/")) {
					rule = rule.slice(1);
				}
				return path.join(relativePath, rule);
			}));
		}
	}

	return excludes;
}
