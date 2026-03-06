const vscode = require("vscode");
const path = require("path");
const fs = require("fs");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

let client;

function activate(context) {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;

  // Find opal-lsp binary
  let serverPath = null;
  const candidates = [];

  if (workspaceFolder) {
    candidates.push(
      path.join(workspaceFolder, "target", "release", "opal-lsp"),
      path.join(workspaceFolder, "target", "debug", "opal-lsp")
    );
  }

  for (const candidate of candidates) {
    try {
      fs.accessSync(candidate, fs.constants.X_OK);
      serverPath = candidate;
      break;
    } catch {}
  }

  if (!serverPath) {
    vscode.window.showWarningMessage(
      "Opal LSP: binary not found. Build with: cargo build --release -p opal-lsp"
    );
    return;
  }

  const serverOptions = {
    run: { command: serverPath, transport: TransportKind.stdio },
    debug: { command: serverPath, transport: TransportKind.stdio },
  };

  const clientOptions = {
    documentSelector: [{ scheme: "file", language: "opal" }],
  };

  client = new LanguageClient(
    "opal-lsp",
    "Opal Language Server",
    serverOptions,
    clientOptions
  );

  client.start();
  context.subscriptions.push(client);
}

function deactivate() {
  if (client) {
    return client.stop();
  }
}

module.exports = { activate, deactivate };
