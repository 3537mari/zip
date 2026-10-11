# Bootstrap ZIP на C

Минимальный компилятор ZIP на C для первой сборки без Rust/LLVM.

**Размер MVP:** ~1000 строк C (лексер / парсер / генератор ZIPB).  
Расширение до полного покрытия selfc — итеративно (цель дорожной карты ~5000 LOC).

## Зачем

Раньше bootstrap был на Rust. Этот каталог — C-путь:
лексер → парсер → AST → ZIPB v3.

## Сборка

```bash
# Linux/macOS
make
# или
gcc -O3 -o zipc утилиты.c лексер.c парсер.c генератор.c главное.c

# Windows (clang)
pwsh ./build.ps1
# или
clang -O3 -o zipc.exe утилиты.c лексер.c парсер.c генератор.c главное.c
```

## Использование

```bash
./zipc программа.zip программа.zipb
./zipc --version
```

## Самокомпиляция (план)

```bash
./zipc ../zip-self/компилятор.zip selfc.zipb
# далее: zip-vm selfc.zipb … (полный self-host требует зрелого ZIPB от selfc)
```

## Зависимости

Только C-компилятор (gcc / clang / MSVC). Без Rust и LLVM.
