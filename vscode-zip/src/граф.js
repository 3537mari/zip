const vscode = require("vscode");

/**
 * @param {import("vscode-languageclient/node").LanguageClient | undefined} client
 * @param {vscode.ExtensionContext} context
 */
function активироватьГраф(client, context) {
  context.subscriptions.push(
    vscode.commands.registerCommand("zip.showAssociations", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showInformationMessage("Нет активного редактора");
        return;
      }
      const слово = получитьСловоПодКурсором(editor);
      if (!слово) {
        vscode.window.showInformationMessage("Нет слова под курсором");
        return;
      }
      const ассоциации = await получитьАссоциации(client, слово);
      const panel = vscode.window.createWebviewPanel(
        "zipAssociations",
        `Ассоциации: ${слово}`,
        vscode.ViewColumn.Beside,
        { enableScripts: false }
      );
      panel.webview.html = сгенерироватьHTMLАссоциаций(слово, ассоциации);
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("zip.showGraph", async () => {
      const граф = await получитьГраф(client);
      const panel = vscode.window.createWebviewPanel(
        "zipGraph",
        "ZIP: граф ассоциаций",
        vscode.ViewColumn.Beside,
        { enableScripts: false }
      );
      panel.webview.html = сгенерироватьHTMLГрафа(граф);
    })
  );
}

/**
 * @param {vscode.TextEditor} editor
 */
function получитьСловоПодКурсором(editor) {
  const pos = editor.selection.active;
  const range = editor.document.getWordRangeAtPosition(pos, /[\wА-Яа-яЁё]+/);
  if (!range) return null;
  return editor.document.getText(range);
}

/**
 * @param {import("vscode-languageclient/node").LanguageClient | undefined} client
 * @param {string} слово
 */
async function получитьАссоциации(client, слово) {
  if (!client) {
    return { слово, ассоциации: [], описание: "(LSP не запущен)" };
  }
  try {
    const рез = await client.sendRequest("workspace/executeCommand", {
      command: "zip.associations",
      arguments: [слово],
    });
    return рез || { слово, ассоциации: [] };
  } catch (e) {
    vscode.window.showWarningMessage(`Ассоциации: ${e}`);
    return { слово, ассоциации: [], описание: String(e) };
  }
}

/**
 * @param {import("vscode-languageclient/node").LanguageClient | undefined} client
 */
async function получитьГраф(client) {
  if (!client) {
    return { узлы: [], рёбра: [] };
  }
  try {
    const рез = await client.sendRequest("workspace/executeCommand", {
      command: "zip.graph",
      arguments: [],
    });
    return рез || { узлы: [], рёбра: [] };
  } catch (e) {
    vscode.window.showWarningMessage(`Граф: ${e}`);
    return { узлы: [], рёбра: [] };
  }
}

/**
 * @param {string} слово
 * @param {any} данные
 */
function сгенерироватьHTMLАссоциаций(слово, данные) {
  const ассоциации = Array.isArray(данные.ассоциации) ? данные.ассоциации : [];
  const описание = данные.описание || "";
  const тип = данные.тип || "";
  const строки = ассоциации
    .map(
      (а) => `
      <div class="связь">
        <span class="иконка">${escapeHtml(а.иконка || "")}</span>
        <strong>${escapeHtml(а.связанное || "")}</strong>
        <small>(${escapeHtml(а.тип || "")})</small>
      </div>`
    )
    .join("");
  return `<!DOCTYPE html>
<html lang="ru">
<head>
  <meta charset="UTF-8">
  <style>
    body {
      font-family: "Segoe UI", system-ui, sans-serif;
      padding: 20px;
      background: linear-gradient(160deg, #f7f3ea 0%, #e8eef5 100%);
      color: #1a1a1a;
    }
    .сущность { font-size: 28px; font-weight: 700; margin-bottom: 8px; letter-spacing: 0.04em; }
    .meta { opacity: 0.75; margin-bottom: 20px; }
    .связь {
      padding: 10px 12px; margin: 6px 0;
      background: rgba(255,255,255,0.75);
      border-left: 3px solid #2f5d50;
      border-radius: 2px;
    }
    .иконка { margin-right: 8px; }
  </style>
</head>
<body>
  <div class="сущность">${escapeHtml(слово)}</div>
  <div class="meta">${escapeHtml(тип)}${описание ? " — " + escapeHtml(описание) : ""}</div>
  ${строки || "<p>Нет ассоциаций в графе.</p>"}
</body>
</html>`;
}

/**
 * @param {any} граф
 */
function сгенерироватьHTMLГрафа(граф) {
  const узлы = Array.isArray(граф.узлы) ? граф.узлы : [];
  const рёбра = Array.isArray(граф.рёбра) ? граф.рёбра : [];
  const списокУзлов = узлы
    .slice(0, 80)
    .map(
      (у) =>
        `<li><strong>${escapeHtml(у.имя)}</strong> <small>${escapeHtml(у.тип || "")}</small> — ${escapeHtml(у.описание || "")}</li>`
    )
    .join("");
  const списокРёбер = рёбра
    .slice(0, 120)
    .map(
      (р) =>
        `<li>${escapeHtml(р.иконка || "")} ${escapeHtml(р.от)} → ${escapeHtml(р.к)} <small>(${escapeHtml(р.тип || "")})</small></li>`
    )
    .join("");
  return `<!DOCTYPE html>
<html lang="ru">
<head>
  <meta charset="UTF-8">
  <style>
    body {
      font-family: "Segoe UI", system-ui, sans-serif;
      padding: 20px;
      background: linear-gradient(160deg, #eef5f2 0%, #f4efe6 100%);
      color: #1a1a1a;
    }
    h1 { font-size: 22px; margin-bottom: 8px; }
    h2 { font-size: 16px; margin-top: 24px; }
    ul { line-height: 1.6; padding-left: 18px; }
  </style>
</head>
<body>
  <h1>Граф ассоциаций ZIP</h1>
  <p>Узлов: ${узлы.length}, рёбер: ${рёбра.length}</p>
  <h2>Узлы</h2>
  <ul>${списокУзлов}</ul>
  <h2>Связи</h2>
  <ul>${списокРёбер}</ul>
</body>
</html>`;
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

module.exports = { активироватьГраф };
