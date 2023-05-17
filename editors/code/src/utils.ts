import * as vscode from 'vscode';

export function delay(ms: number) {
  return new Promise( resolve => setTimeout(resolve, ms) );
}

export async function retryCommand<T>(
  retries: number,
  delayMs: number,
  command: string, ...rest: any[]
): Promise<T | undefined> {
  let result: T | undefined;

  for (let i = 0; i < retries; i++) {
    result = await vscode.commands.executeCommand(command, ...rest);
    if (result !== undefined) {
      break;
    }
    await delay(delayMs);
  }

  return result;
}
