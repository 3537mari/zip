# WASM пакет zipc

Сборка:

```bash
cd zipc
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web --features wasm --out-dir ../веб-сайт/wasm
```

Плейграунд (`плейграунд.html`) подхватывает `./wasm/zipc.js`. Без сборки работает JS-fallback.
