import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

function defaultLspPath(): string {
  const cargoHome = process.env.CARGO_HOME || path.join(os.homedir(), ".cargo");
  const binary =
    process.platform === "win32" ? "iguana-lsp.exe" : "iguana-lsp";
  const cargoBinary = path.join(cargoHome, "bin", binary);
  return fs.existsSync(cargoBinary) ? cargoBinary : "iguana-lsp";
}

export function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration("iguana.lsp");
  const lspPath = config.get<string>("path") || defaultLspPath();

  const serverOptions: ServerOptions = {
    command: lspPath,
    args: [],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "iggy" }],
  };

  client = new LanguageClient(
    "iguana-lsp",
    "Iguana Language Server",
    serverOptions,
    clientOptions,
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
