import * as vscode from 'vscode';

export async function retryCommand<T>(
  retries: number,
  ms: number,
  command: string,
  ...rest: any[]
): Promise<T | undefined> {
  let result: T | undefined;

  for (let i = 0; i < retries; i++) {
    result = await vscode.commands.executeCommand(command, ...rest);
    if (result !== undefined) {
      break;
    }
    await new Promise( resolve => setTimeout(resolve, ms) );
  }

  return result;
}
