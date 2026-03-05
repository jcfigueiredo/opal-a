const vscode = require("vscode");
const path = require("path");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

let client;

function activate(context) {
  // Look for the LSP binary in order of preference:
  // 1. Workspace-relative target/release/opal-lsp
  // 2. Absolute path from config
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  const configPath = vscode.workspace.getConfiguration("opal").get("lsp.path");

  let serverPath = configPath;
  if (!serverPath && workspaceFolder) {
    const candidates = [
      path.join(workspaceFolder, "target", "release", "opal-lsp"),
      path.join(workspaceFolder, "target", "debug", "opal-lsp"),
    ];
    for (const candidate of candidates) {
      try {
        require("fs").accessSync(candidate, require("fs").constants.X_OK);
        serverPath = candidate;
        break;
      } catch {}
    }
  }

  if (!serverPath) {
    vscode.window.showWarningMessage(
      "Opal LSP binary not found. Build with: cargo build --release -p opal-lsp"
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
