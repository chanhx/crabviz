import * as vscode from 'vscode';

interface ExtensionManifest {
  contributes?: {
    languages?: {
      aliases?: string[],
      extensions?: string[],
    }[],
  }
}

export function getLanguages(): Map<string, string> {
  let m = new Map(
    vscode.extensions.all
      .map(e => <any[]>(e.packageJSON as ExtensionManifest)?.contributes?.languages)
      .filter(langs => langs)
      .reduce((a, b) => a.concat(b), [])
      .filter(lang => (lang.aliases?.length ?? 0 > 0) && (lang.extensions?.length ?? 0 > 0))
      .flatMap<[string, string]>(lang => lang.extensions?.map((ext: string) => [ext, lang.aliases?.[0]]))
  );

  return m;
}
