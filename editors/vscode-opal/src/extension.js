const vscode = require("vscode");
const path = require("path");
const fs = require("fs");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

let client;
const outputChannel = vscode.window.createOutputChannel("Opal Language Server");

function activate(context) {
  outputChannel.appendLine("Opal extension activating...");

  const workspaceFolder = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  outputChannel.appendLine("Workspace folder: " + (workspaceFolder || "none"));

  // Find opal-lsp binary — check workspace, then home, then PATH
  let serverPath = null;
  const candidates = [];

  if (workspaceFolder) {
    candidates.push(
      path.join(workspaceFolder, "target", "release", "opal-lsp"),
      path.join(workspaceFolder, "target", "debug", "opal-lsp")
    );
  }
  // Also check common install locations
  candidates.push(
    path.join(process.env.HOME || "", ".cargo", "bin", "opal-lsp"),
    "/usr/local/bin/opal-lsp"
  );

  for (const candidate of candidates) {
    outputChannel.appendLine("Checking: " + candidate);
    try {
      fs.accessSync(candidate, fs.constants.X_OK);
      serverPath = candidate;
      outputChannel.appendLine("Found LSP binary: " + candidate);
      break;
    } catch {}
  }

  if (!serverPath) {
    const msg = "Opal LSP binary not found. Build with: cargo build --release -p opal-lsp";
    outputChannel.appendLine("ERROR: " + msg);
    vscode.window.showWarningMessage(msg);
    return;
  }

  const serverOptions = {
    run: { command: serverPath, transport: TransportKind.stdio },
    debug: { command: serverPath, transport: TransportKind.stdio },
  };

  const clientOptions = {
    documentSelector: [{ scheme: "file", language: "opal" }],
    outputChannel,
  };

  client = new LanguageClient(
    "opal-lsp",
    "Opal Language Server",
    serverOptions,
    clientOptions
  );

  client.start().then(() => {
    outputChannel.appendLine("Opal LSP started successfully");
  }).catch((err) => {
    outputChannel.appendLine("ERROR starting LSP: " + err.message);
  });

  context.subscriptions.push(client);
}

function deactivate() {
  if (client) {
    return client.stop();
  }
}

module.exports = { activate, deactivate };
