# zip-self-zip — Bootstrap-компилятор на ZIP (v6)

| Модуль | Роль |
|--------|------|
| `лексика_чистая.zip` | API лексера → `zip_lex_полный` |
| `парсер_чистый.zip` | API парсера → `zip_parse_полный` |
| `генератор_чистый.zip` | API генератора → `zip_gen_полный` |
| `главное_чистое.zip` | CLI selfc (stage-1+ без C) |
| `главное.zip` | Legacy драйвер v4/v5 |

## Fixed-point без C

```bash
./bootstrap/zipc zip-self-zip/главное_чистое.zip /tmp/selfc_v1.zipb
./zip-vm/target/release/zip-vm /tmp/selfc_v1.zipb \
    zip-self-zip/главное_чистое.zip /tmp/selfc_v2.zipb
./zip-vm/target/release/zip-vm /tmp/selfc_v2.zipb \
    zip-self-zip/главное_чистое.zip /tmp/selfc_v3.zipb
# v2 == v3
```

Windows: `pwsh zip-self-zip/тесты/тест_fixed_point_без_c.ps1`

`zip_self_compile` пишет ZIPB через VM. Старый путь: `ZIP_USE_C_BOOTSTRAP=1`.

См. [docs/ОТЧЁТ-v6.md](../docs/ОТЧЁТ-v6.md).
