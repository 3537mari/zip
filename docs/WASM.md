# ZIP на WASM (WebAssembly)

## Что нужно знать

После v6.1.0 **компилятор ZIP (`selfc.zipb`) платформонезависим**. В браузере нет
`std::process` и файлового I/O, поэтому для WASM нужна **отдельная сборка VM**
с web-совместимым backend (`wasm-bindgen`, захват вывода в буфер).

ZIPB тот же: один `.zipb` работает на x86-64, ARM64, RISC-V и в браузере.

## Сборка

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.100

cd zip-vm
cargo build --lib --release \
    --target wasm32-unknown-unknown \
    --no-default-features --features wasm

wasm-bindgen \
    --target web \
    --out-dir ../веб-сайт/wasm \
    target/wasm32-unknown-unknown/release/zip_vm.wasm
```

Артефакты: `zip_vm.js`, `zip_vm_bg.wasm`, `zip_vm.d.ts`.

## Использование в JS

```js
import init, { compileAndRun, versionZip } from './wasm/zip_vm.js';

await init();
const результат = compileAndRun(`
функция главная() {
    напечатать_строку("Привет из WASM!")
    возврат(42)
}
`);
document.body.innerHTML = `<pre>${результат}</pre>`;
```

| JS API | Назначение |
|--------|------------|
| `compileZip(src)` | ZIP → ZIPB hex |
| `runZip(hex)` | ZIPB → вывод |
| `compileAndRun(src)` | ZIP → вывод |
| `versionZip()` | Версия |

## Плейграунд

https://3537mari.github.io/zip/плейграунд.html

- Написать ZIP-программу
- Скомпилировать в ZIPB (hex)
- Запустить и увидеть вывод

Локальные тесты: `веб-сайт/wasm/тест_wasm.html` (нужен HTTP-сервер).

## CI

См. [`.github/workflows/wasm.yml`](../.github/workflows/wasm.yml) — сборка + headless Chrome.

## Размер

`zip_vm_bg.wasm` ≈ 200–400 КБ при `opt-level = "z"`, `lto = true` (лимит приёмки: &lt; 5 МБ).

## Что дальше

- Нативные пакеты ARM64 / RISC-V в Releases
- Windows ARM (нативная сборка)
