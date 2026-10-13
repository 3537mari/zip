# WASM-модуль ZIP

Здесь лежит компилированная ZIP-VM для браузера (`zip_vm.js` / `zip_vm_bg.wasm`),
плюс устаревший compile-only пакет `zipc.js` (совместимость).

## Сборка VM (рекомендуется)

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.100

cd zip-vm
cargo build --lib --release \
  --target wasm32-unknown-unknown \
  --no-default-features --features wasm

wasm-bindgen --target web --out-dir ../веб-сайт/wasm \
  target/wasm32-unknown-unknown/release/zip_vm.wasm
```

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
console.log(результат);
```

## Экспортируемые функции

| JS-имя | Назначение |
|--------|------------|
| `compileZip(src)` | ZIP → ZIPB (hex) |
| `runZip(hex)` | ZIPB hex → вывод |
| `compileAndRun(src)` | ZIP → вывод |
| `versionZip()` | Версия crate |

Тесты: открыть [`тест_wasm.html`](тест_wasm.html) через локальный HTTP-сервер.
