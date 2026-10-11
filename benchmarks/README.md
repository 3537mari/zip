# Бенчмарки ZIP vs Python vs Rust vs Go

Сравнительные задачи:

| Задача | Описание |
|--------|----------|
| `fibonacci/` | Fibonacci(40), наивная рекурсия |
| `gemm/` | GEMM 512×512 |
| `json/` | Парсинг JSON (~10 000 объектов) |
| `strings/` | 1 000 000 строковых операций |
| `http/` | 5 000 000 разборов HTTP GET (микробенч) |
| `nn/` | XOR MLP, 10 000 эпох |

## Запуск

```powershell
pwsh ./scripts/запустить_все.ps1
```

```bash
./scripts/запустить_все.sh
```

Результаты: `результаты.md`, отчёт: [`../docs/БЕНЧМАРКИ.md`](../docs/БЕНЧМАРКИ.md).

## Примечания

- **ZIP Fibonacci** — native-бэкенд zipc (LLVM IR сейчас ломается на рекурсии).
- **ZIP GEMM / JSON / strings / http / nn** — STD `#runtime(...)` (C + clang -O3), как в экосистеме ZIP.
- Python GEMM использует **NumPy** (BLAS).
- Rust JSON — `serde_json`.
