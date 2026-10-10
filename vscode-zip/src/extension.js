const vscode = require("vscode");
const path = require("path");
const fs = require("fs");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

/** @type {LanguageClient | undefined} */
let client;

function findLspBinary(configured) {
  if (configured && configured.trim()) {
    return configured.trim();
  }
  const extRoot = path.resolve(__dirname, "..");
  const repoRoot = path.resolve(extRoot, "..");
  const candidates = [
    path.join(repoRoot, "зип-lsp", "target", "release", "zip-lsp.exe"),
    path.join(repoRoot, "зип-lsp", "target", "release", "зип-lsp.exe"),
    path.join(repoRoot, "зип-lsp", "target", "release", "zip-lsp"),
    path.join(repoRoot, "зип-lsp", "target", "release", "зип-lsp"),
    path.join(repoRoot, "zip-lsp", "target", "release", "zip-lsp.exe"),
    "zip-lsp",
    "зип-lsp",
  ];
  for (const c of candidates) {
    if (path.isAbsolute(c) || c.includes(path.sep) || c.includes("/")) {
      if (fs.existsSync(c)) return c;
    }
  }
  return process.platform === "win32" ? "zip-lsp.exe" : "zip-lsp";
}

/**
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
  const config = vscode.workspace.getConfiguration("zip");
  const serverPath = findLspBinary(config.get("lsp.path"));

  const serverOptions = {
    run: { command: serverPath, transport: TransportKind.stdio },
    debug: { command: serverPath, transport: TransportKind.stdio },
  };

  const clientOptions = {
    documentSelector: [
      { scheme: "file", language: "zip" },
      { scheme: "untitled", language: "zip" },
    ],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.{zip,зип}"),
    },
  };

  client = new LanguageClient("zip-lsp", "ЗИП-LSP", serverOptions, clientOptions);
  context.subscriptions.push(client.start());
  vscode.window.setStatusBarMessage(`ZIP LSP: ${serverPath}`, 5000);
}

function deactivate() {
  if (!client) return undefined;
  return client.stop();
}

module.exports = { activate, deactivate };
