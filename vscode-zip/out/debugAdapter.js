#!/usr/bin/env node
/**
 * Тонкая обёртка: запускает нативный зип-отладчик (DAP stdio).
 * VS Code / Cursor подключаются к этому процессу как к debug adapter.
 */
const { spawn } = require("child_process");
const fs = require("fs");
const path = require("path");

function findDebugger() {
  const env = process.env.ZIP_DEBUGGER;
  if (env && fs.existsSync(env)) return env;

  const repoRoot = path.resolve(__dirname, "..", "..");
  const candidates = [
    path.join(repoRoot, "зип-отладчик", "target", "release", "zip-debugger.exe"),
    path.join(repoRoot, "зип-отладчик", "target", "release", "zip-debugger"),
    path.join(repoRoot, "зип-отладчик", "target", "release", "зип-отладчик.exe"),
    path.join(repoRoot, "зип-отладчик", "target", "release", "зип-отладчик"),
    path.join(repoRoot, "zip-debugger", "target", "release", "zip-debugger.exe"),
    path.join(repoRoot, "zip-debugger", "target", "release", "zip-debugger"),
  ];
  for (const c of candidates) {
    if (fs.existsSync(c)) return c;
  }
  return process.platform === "win32" ? "zip-debugger.exe" : "zip-debugger";
}

const bin = findDebugger();
const child = spawn(bin, [], {
  stdio: ["pipe", "pipe", "inherit"],
  windowsHide: true,
});

process.stdin.pipe(child.stdin);
child.stdout.pipe(process.stdout);

child.on("error", (err) => {
  console.error(`не удалось запустить отладчик (${bin}): ${err.message}`);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) process.kill(process.pid, signal);
  process.exit(code ?? 0);
});

process.on("SIGINT", () => child.kill("SIGINT"));
process.on("SIGTERM", () => child.kill("SIGTERM"));
