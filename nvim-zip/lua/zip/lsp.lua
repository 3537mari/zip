-- Интеграция с LSP-сервером ZIP
local M = {}

function M.setup(config)
  if not config.enable_lsp then
    return
  end

  local ok, lspconfig = pcall(require, "lspconfig")
  if not ok then
    vim.notify("nvim-lspconfig не установлен", vim.log.levels.WARN)
    return
  end

  local configs = require("lspconfig.configs")

  if not configs.zip then
    configs.zip = {
      default_config = {
        cmd = { config.lsp_path },
        filetypes = { "zip" },
        root_dir = lspconfig.util.root_pattern("зипм.манифест", ".git"),
        settings = {},
        init_options = {},
      },
    }
  end

  lspconfig.zip.setup({})

  if config.format_on_save then
    vim.api.nvim_create_autocmd("BufWritePre", {
      pattern = { "*.zip", "*.зип" },
      callback = function()
        vim.lsp.buf.format({ async = false })
      end,
    })
  end
end

return M
