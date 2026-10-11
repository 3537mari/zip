# Runtime на ZIP

## Цель v4.0.0

Убрать C-runtime из пользовательского кода. Системные операции —
через абстракцию syscalls на ZIP (+ тонкая C/VM-обёртка).

## Архитектура

```
ZIP-программа
    ↓  (#runtime("syscall") / вызовы sys_*)
syscalls.zip / аллокатор.zip / вывод.zip / файлы.zip
    ↓  (zip_syscall / builtins VM)
C-обёртка (~30–80 строк)  или  встроенные VM v3
    ↓  (системный вызов ОС)
Ядро ОС
```

## Что заменено

| Было (C) | Стало (ZIP) |
|----------|-------------|
| `fopen` | `файл_открыть` → `sys_open` |
| `fread` | `файл_прочитать` → `sys_read` |
| `fwrite` | `файл_записать` → `sys_write` |
| `malloc` | `выделить` → `sys_mmap` / bump |
| `printf` | `напечатать_число` → `sys_write` |
| `exit` | `sys_exit` |

## Модули STD

| Файл | Назначение |
|------|------------|
| [`zip-std/syscalls.zip`](../zip-std/syscalls.zip) | Номера syscalls + обёртки |
| [`zip-std/аллокатор.zip`](../zip-std/аллокатор.zip) | Bump-аллокатор |
| [`zip-std/вывод.zip`](../zip-std/вывод.zip) | stdout / stderr |
| [`zip-std/файлы.zip`](../zip-std/файлы.zip) | Файловый I/O |
| [`zip-std/runtime/syscall.c`](../zip-std/runtime/syscall.c) | `zip_syscall` (Linux/Windows) |
| [`zip-std/runtime/allocator.c`](../zip-std/runtime/allocator.c) | `zip_alloc` |
| [`zip-std/runtime/io.c`](../zip-std/runtime/io.c) | Печать |
| [`zip-std/runtime/files.c`](../zip-std/runtime/files.c) | Файлы |

## Syscalls в ZIP

```zip
sys_read(fd, буфер, размер)
sys_write(fd, буфер, размер)
sys_open(путь, флаги, режим)
sys_mmap(размер)
sys_exit(код)
```

## Свой аллокатор

Bump allocator с расширением:

```zip
функция выделить(размер) {
    возврат(zip_alloc(размер))
}
```

## Запуск демо

```bash
# Через C-bootstrap → ZIPB → VM (builtins)
./bootstrap/zipc examples/чистый_zip.zip /tmp/чистый_zip.zipb
./zip-vm/target/release/zip-vm /tmp/чистый_zip.zipb

# Через #runtime C (clang)
./zipc/target/release/zipc собрать examples/чистый_zip.zip -o /tmp/чистый
# (первый #runtime("syscall") / или #runtime("чистый_zip"))
```

## Что осталось (v4.x)

- [ ] Прямой inline-asm для syscalls (без C-обёртки)
- [ ] Переписать VM на ZIP
- [ ] Переписать bootstrap на ZIP
- [ ] STD полностью на ZIP (без VM builtins)
