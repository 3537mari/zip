// ZIP interactive course — WASM + theme
let zipModule = null;
let compileFn = null;

async function initializeZip() {
  const candidates = [
    "../wasm/zipc.js",
    "./wasm/zipc.js",
    "../../веб-сайт/wasm/zipc.js",
  ];
  for (const path of candidates) {
    try {
      const module = await import(path);
      await module.default();
      zipModule = module;
      compileFn = module.compileZip || module.скомпилировать || module.compile;
      console.log("ZIP WASM ready via", path);
      return;
    } catch (e) {
      console.warn("WASM try failed:", path, e);
    }
  }
  compileFn = null;
}

function fallback(src) {
  const lines = src.split(/\n/).filter(Boolean).length;
  const hasMain = /функция\s+главная/.test(src);
  const hasPrint = /ПечатьСтроки|печать_строки|ПечатьЧисло/.test(src);
  return [
    "Разбор (fallback без WASM)",
    "Строк: " + lines,
    "главная: " + (hasMain ? "да" : "нет"),
    "печать: " + (hasPrint ? "да" : "нет"),
    "",
    "Полный WASM доступен после деплоя GitHub Pages.",
  ].join("\n");
}

function compileOnly(code) {
  try {
    return compileFn ? String(compileFn(code)) : fallback(code);
  } catch (error) {
    return "Ошибка: " + (error && error.message ? error.message : error);
  }
}

async function runCode(code) {
  const result = compileOnly(code);
  showResult(result);
  return result;
}

function showResult(text) {
  const modal = document.createElement("div");
  modal.className = "modal";
  modal.innerHTML =
    '<div class="modal-content">' +
    "<h3>Результат</h3>" +
    "<pre></pre>" +
    '<button type="button">Закрыть</button>' +
    "</div>";
  modal.querySelector("pre").textContent = text;
  modal.querySelector("button").onclick = () => modal.remove();
  modal.onclick = (e) => {
    if (e.target === modal) modal.remove();
  };
  document.body.appendChild(modal);
}

function checkQuiz(btn, correct) {
  const wrap = btn.closest(".quiz");
  wrap.querySelectorAll("button").forEach((b) => {
    b.classList.remove("ok", "bad");
  });
  btn.classList.add(correct ? "ok" : "bad");
}

function переключить_тему() {
  document.body.classList.toggle("тёмная");
  const isDark = document.body.classList.contains("тёмная");
  localStorage.setItem("zip-theme", isDark ? "dark" : "light");
  const btn = document.querySelector(".theme-toggle");
  if (btn) btn.textContent = isDark ? "☀️" : "🌙";
}

function применить_тему() {
  const theme = localStorage.getItem("zip-theme");
  if (theme === "dark") {
    document.body.classList.add("тёмная");
  }
  const btn = document.querySelector(".theme-toggle");
  if (btn) {
    btn.textContent = document.body.classList.contains("тёмная") ? "☀️" : "🌙";
  }
}

function номер_урока() {
  const m = location.pathname.match(/урок-0?(\d+)/);
  return m ? parseInt(m[1], 10) : null;
}

document.addEventListener("DOMContentLoaded", () => {
  применить_тему();
  initializeZip();
  const n = номер_урока();
  if (n && typeof прогресс !== "undefined") {
    прогресс.отметить_урок(n, "начат");
  }
});

window.runCode = runCode;
window.compileOnly = compileOnly;
window.checkQuiz = checkQuiz;
window.переключить_тему = переключить_тему;
window.showResult = showResult;
