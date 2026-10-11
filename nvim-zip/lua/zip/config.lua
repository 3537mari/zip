-- Настройки плагина nvim-zip
local M = {}

M.defaults = {
  -- Путь к LSP-серверу (если не в PATH)
  lsp_path = "зип-lsp",

  -- Включить LSP
  enable_lsp = true,

  -- Включить подсветку
  enable_syntax = true,

  -- Включить автодополнение
  enable_completion = true,

  -- Отступы
  indent = 4,

  -- Форматирование при сохранении
  format_on_save = false,
}

M.options = {}

function M.setup(opts)
  M.options = vim.tbl_deep_extend("force", M.defaults, opts or {})
end

return M
