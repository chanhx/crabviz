import { workspace, CancellationToken, RelativePattern, Uri, WorkspaceFolder, FileType } from "vscode";
import * as path from "path";

import ignore, { Ignore } from 'ignore';

const preIgnored = [
  '.*',

  '*.html',
  '*.css',
  '*.md',
  '*.txt',

  '*.jpeg',
  '*.jpg',
  '*.gif',
  '*.png',

  '*.csv',
  '*.json',

  '*.lex',
  '*.yacc',

  '*.lock',
  '*.log',

  '*.toml',
  '*.xml',
  '*.yaml',
  '*.yml',
];

export async function ignoreManager(
  folder: WorkspaceFolder
): Promise<Ignore> {
  const ignores = await workspace.findFiles(
    new RelativePattern(folder, "**/.gitignore")
  );

  const rulePromises = ignores.map(async (ignore) => {
    const dir = path.dirname(ignore.path);
    const relativePath = path.relative(folder.uri.path, dir);

    return await workspace.fs.readFile(ignore).then((content) =>
      content
        .toString()
        .split("\n")
        .filter((rule) => rule.trim().length > 0)
        .map((rule) => {
          if (rule.startsWith("/")) {
            rule = rule.slice(1);
          }
          return path.join(relativePath, rule);
        })
    );
  });

  const rules = (await Promise.all(rulePromises)).flat();

	return ignore().add(rules).add(preIgnored);
}

export async function collectFileExtensions(
  root: string,
  directories: Uri[],
	ig: Ignore,
	token: CancellationToken
): Promise<Set<string>> {
  let extensions = new Set<string>();

  for await (const dir of directories) {
    const relativePath = path.relative(root, dir.path);

    if (relativePath.length > 0 && ig.ignores(relativePath)) {
      continue;
    }
    await _collectFileExtensions(root, dir, ig, extensions, token);
  }

  return extensions;
}

async function _collectFileExtensions(
  root: string,
	dir: Uri,
	ig: Ignore,
  extensions: Set<string>,
	token: CancellationToken
) {
  const entries = await workspace.fs.readDirectory(dir);

  for await (const entry of entries) {
    const filePath = path.join(dir.path, entry[0]);
    const fileType = entry[1];

    if (ig.ignores(path.relative(root, filePath))) {
      continue;
    }

    if (token.isCancellationRequested) {
      return;
    }

    if ((fileType & FileType.Directory) === FileType.Directory) {
      await _collectFileExtensions(root, Uri.parse(filePath), ig, extensions, token);
      continue;
    } else if ((fileType | FileType.File) !== FileType.File) {
      continue;
    }

    const ext = path.extname(entry[0]).substring(1);

    if (ext.length > 0) {
      extensions.add(ext);
    }
  }
}

export async function collectFiles(
  root: string,
	directories: Uri[],
  fileExtensions: Set<string>,
	ig: Ignore,
): Promise<Set<Uri>> {
  let files = new Set<Uri>();

  for await (const dir of directories) {
    const relativePath = path.relative(root, dir.path);

    if (relativePath.length > 0 && ig.ignores(relativePath)) {
      continue;
    }
    await _collectFiles(root, dir, ig, files, fileExtensions);
  }

  return files;
}

async function _collectFiles(
  root: string,
	dir: Uri,
	ig: Ignore,
  files: Set<Uri>,
  fileExtensions: Set<string>,
) {
  const entries = await workspace.fs.readDirectory(dir);

  for await (const entry of entries) {
    const filePath = path.join(dir.path, entry[0]);
    const fileType = entry[1];

    if (ig.ignores(path.relative(root, filePath))) {
      continue;
    }

    const isDirectory = (fileType & FileType.Directory) === FileType.Directory;
    const isFile = (fileType & FileType.File) === FileType.File;

    if (!isDirectory && !isFile) {
      continue;
    }

    const uri = Uri.parse(filePath);

    if (isDirectory) {
      await _collectFiles(root, uri, ig, files, fileExtensions);
    } else if (isFile) {
      const ext = path.extname(entry[0]).substring(1);
      if (fileExtensions.has(ext)) {
        files.add(uri);
      }
    }
  }
}
