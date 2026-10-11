# nvim-zip

Плагин ZIP для Neovim.

## Возможности

- Подсветка синтаксиса ZIP
- LSP-интеграция (автодополнение, диагностика)
- Горячие клавиши для сборки/запуска/тестов
- Форматирование (опционально при сохранении)

## Установка

### lazy.nvim

```lua
{
    "3537mari/nvim-zip",
    dependencies = { "neovim/nvim-lspconfig" },
    config = function()
        require("zip").setup({
            lsp_path = "зип-lsp",
            enable_lsp = true,
            indent = 4,
        })
    end,
    ft = { "zip" },
}
```

Локально из монорепозитория:

```lua
{
    dir = "~/zip-lang/nvim-zip",
    dependencies = { "neovim/nvim-lspconfig" },
    config = function()
        require("zip").setup({})
    end,
    ft = { "zip" },
}
```

### packer.nvim

```lua
use {
    "3537mari/nvim-zip",
    requires = { "neovim/nvim-lspconfig" },
    config = function()
        require("zip").setup({})
    end,
}
```

## Горячие клавиши

| Клавиша | Действие |
|---------|----------|
| `<leader>zr` | Запустить программу |
| `<leader>zb` | Собрать программу |
| `<leader>zt` | Запустить тесты |
| `<leader>zp` | Профилировать |

## Требования

- Neovim 0.9+
- nvim-lspconfig
- ZIP-компилятор (`zipc`) в PATH
- LSP-сервер (`зип-lsp`) в PATH

## Замечание про `*.zip`

Расширение `.zip` в проекте ZIP — исходники языка. Плагин назначает им `filetype=zip`.
Если нужен архивный zip.vim Vim, откройте бинарный архив с `setfiletype zipfile` или отключите ftdetect.
