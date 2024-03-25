import { workspace, CancellationToken, Uri, FileType } from "vscode";
import * as path from "path";

import { Ignore } from 'ignore';

export class FileClassifier {
  private root: string;
  private languages: Map<string, string>;
  private ig: Ignore;
  private files: Map<string, Uri[]>;

  public constructor(root: string, languages: Map<string, string>, ig: Ignore) {
    this.root = root;
    this.languages = languages;
    this.ig = ig;
    this.files = new Map<string, Uri[]>();
  }

  public async classifyFilesByLanguage(
    uris: Uri[],
    token: CancellationToken
  ): Promise<Map<string, Uri[]>> {
    for await (const uri of uris) {
      const fileType = (await workspace.fs.stat(uri)).type;

      if ((fileType & FileType.Directory) === FileType.Directory) {
        const relativePath = path.posix.relative(this.root, uri.path);

        if (relativePath.length > 0 && this.ig.ignores(relativePath)) {
          continue;
        }
        await this.classifyFilesInDirectory(uri, token);
      } else if ((fileType & FileType.File) === FileType.File) {
        this.addFile(uri);
      }
    }

    return this.files;
  }

  private async classifyFilesInDirectory(
    dir: Uri,
    token: CancellationToken
  ) {
    const entries = await workspace.fs.readDirectory(dir);

    for await (const entry of entries) {
      if (token.isCancellationRequested) {
        return;
      }

      const uri = Uri.joinPath(dir, entry[0]);

      if (this.ig.ignores(path.posix.relative(this.root, uri.path))) {
        continue;
      }

      const fileType = entry[1];
      const isDirectory = (fileType & FileType.Directory) === FileType.Directory;
      const isFile = (fileType & FileType.File) === FileType.File;

      if (!isDirectory && !isFile) {
        continue;
      }

      if (isDirectory) {
        await this.classifyFilesInDirectory(uri, token);
      } else {
        this.addFile(uri);
      }
    }
  }

  private addFile(uri: Uri): boolean {
    const ext = path.extname(uri.path);
    if (ext.length <= 0) {
      return false;
    }

    const lang = this.languages.get(ext);
    if (!lang) {
      return false;
    }

    if (this.files.has(lang)) {
      this.files.get(lang)?.push(uri);
    } else {
      this.files.set(lang, [uri]);
    }

    return true;
  }
}