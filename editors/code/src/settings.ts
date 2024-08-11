import * as vscode from "vscode";

export const languageId = "dot";
export const docSelector = {
  language: languageId,
};
export const fileExtension = ".dot";

export function extensionConfig() {
  return vscode.workspace.getConfiguration("codevisual");
}

export function extensionBaseConfig(id: string) {
  return vscode.workspace.getConfiguration(id);
}

export function extension() {
  return vscode.extensions.getExtension("atomic.codevisual");
}
