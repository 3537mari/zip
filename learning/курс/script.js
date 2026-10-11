// ZIP interactive course — WASM compile + fallback analyzer
let zipModule = null;
let compileFn = null;

async function initializeZip() {
  try {
    const module = await import("../../веб-сайт/wasm/zipc.js");
    await module.default();
    zipModule = module;
    compileFn = module.compileZip || module.скомпилировать || module.compile;
    console.log("ZIP WASM ready");
  } catch (error) {
    console.warn("WASM unavailable, using fallback", error);
    compileFn = null;
  }
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
    "Полный WASM доступен на сайте после деплоя Actions.",
  ].join("\n");
}

async function runCode(code) {
  try {
    const result = compileFn ? compileFn(code) : fallback(code);
    showResult(result);
  } catch (error) {
    showResult("Ошибка: " + (error && error.message ? error.message : error));
  }
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

document.addEventListener("DOMContentLoaded", initializeZip);
window.runCode = runCode;
window.checkQuiz = checkQuiz;
