import { workspace, CancellationToken, Uri, FileType } from "vscode";
import * as path from "path";

import { languagesByExtension } from "./languages";
import { Ignore } from 'ignore';

export class FileClassifier {
  private root: string;
  private ig: Ignore;
  private files: Map<string, Uri[]>;

  public constructor(root: string, ig: Ignore) {
    this.root = root;
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
        const relativePath = path.relative(this.root, uri.path);

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

      const filePath = path.join(dir.path, entry[0]);
      const fileType = entry[1];

      if (this.ig.ignores(path.relative(this.root, filePath))) {
        continue;
      }

      const isDirectory = (fileType & FileType.Directory) === FileType.Directory;
      const isFile = (fileType & FileType.File) === FileType.File;

      if (!isDirectory && !isFile) {
        continue;
      }

      const uri = Uri.parse(filePath);

      if (isDirectory) {
        await this.classifyFilesInDirectory(uri, token);
      } else {
        this.addFile(uri);
      }
    }
  }

  private addFile(uri: Uri): boolean {
    const ext = path.extname(uri.path).substring(1);
    if (ext.length <= 0) {
      return false;
    }

    const lang = languagesByExtension[ext];
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