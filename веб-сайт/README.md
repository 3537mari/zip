# Сайт ZIP

Официальный сайт (после DNS): https://zip-lang.org  
Зеркало GitHub Pages: https://3537mari.github.io/zip/

## Локальная разработка

```bash
cd веб-сайт
bundle install
bundle exec jekyll serve
# Открой http://localhost:4000/zip/
```

## WASM-плейграунд

```bash
cd zipc
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web --features wasm --out-dir ../веб-сайт/wasm
```

Плейграунд: `/плейграунд.html` (JS-fallback работает и без WASM).

## Публикация

Автоматически при push в `веб-сайт/**` через `.github/workflows/deploy-site.yml`  
→ ветка `gh-pages`.

Требование: репозиторий **public** (GitHub Pages недоступен для private на free-плане).

## Структура

- `index.md` — главная
- `установка.md`, `туториал.md`, `документация.md`
- `плейграунд.html` — онлайн-компилятор
- `документация/`, `примеры/`, `пакеты/`
- `wasm/` — артефакты wasm-pack
- `assets/` — стили и статика
