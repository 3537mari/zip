# STD на чистом ZIP

## Модули v4 (`zip-std/*_zip.zip`)

| Модуль | Файл | Статус |
|--------|------|--------|
| Ядро | [`ядро_zip.zip`](../zip-std/ядро_zip.zip) | Числа на ZIP; строки/СИПРА → `zip_std_*` |
| Тензоры | [`тензоры_zip.zip`](../zip-std/тензоры_zip.zip) | API на ZIP; GEMM → VM |
| Нейросети | [`нейросети_zip.zip`](../zip-std/нейросети_zip.zip) | Слои → VM |
| Крипто | [`крипто_zip.zip`](../zip-std/крипто_zip.zip) | SHA-256 → VM (sha2) |
| JSON | [`json_zip.zip`](../zip-std/json_zip.zip) | MVP parse |
| CSV | [`csv_zip.zip`](../zip-std/csv_zip.zip) | MVP parse |
| RegEx | [`regex_zip.zip`](../zip-std/regex_zip.zip) | MVP contains |
| Сеть | [`сеть_zip.zip`](../zip-std/сеть_zip.zip) | HTTP GET заглушка |
| Syscalls asm | [`syscalls_inline.zip`](../zip-std/syscalls_inline.zip) | x86-64 Linux |

Legacy (64 корня, C-runtime): [`ядро.zip`](../zip-std/ядро.zip), [`тензоры.zip`](../zip-std/тензоры.zip), …

## Архитектура

```
Исходник ZIP (*_zip.zip)
    → bootstrap zipc → .zipb
    → zip-vm (встроенные zip_std_v4)
    → syscalls / файлы (runtime)
```

Постепенно логика переносится из VM в ZIP по мере расширения bootstrap (структуры, индексация, f64).

## Преимущества

1. Портативность — syscalls вместо libc в пользовательском коде
2. Прозрачность — исходники STD читаются
3. Самодостаточность — C только для bootstrap и тонких runtime

## Тесты

```bash
cargo test --manifest-path zipc/Cargo.toml --test тест_std_zip
```
