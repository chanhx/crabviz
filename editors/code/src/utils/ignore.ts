import { workspace, RelativePattern, WorkspaceFolder } from "vscode";
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
  '*.svg',

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

export async function readIgnores(
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
