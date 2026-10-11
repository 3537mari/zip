-- Основная логика плагина nvim-zip
local config = require("zip.config")
local lsp = require("zip.lsp")

local M = {}

function M.setup(opts)
  config.setup(opts)
  lsp.setup(config.options)

  vim.api.nvim_create_autocmd("FileType", {
    pattern = "zip",
    callback = function()
      vim.bo.tabstop = config.options.indent
      vim.bo.shiftwidth = config.options.indent
      vim.bo.expandtab = true
      vim.bo.commentstring = "// %s"
    end,
  })

  vim.keymap.set("n", "<leader>zr", function()
    local файл = vim.fn.expand("%:p")
    vim.cmd("!zipc запустить " .. vim.fn.shellescape(файл))
  end, { desc = "ZIP: запустить программу" })

  vim.keymap.set("n", "<leader>zb", function()
    local файл = vim.fn.expand("%:p")
    local выход = vim.fn.expand("%:p:r")
    vim.cmd(
      "!zipc собрать "
        .. vim.fn.shellescape(файл)
        .. " -o "
        .. vim.fn.shellescape(выход)
        .. " --llvm"
    )
  end, { desc = "ZIP: собрать" })

  vim.keymap.set("n", "<leader>zt", function()
    local файл = vim.fn.expand("%:p")
    vim.cmd("!zipc тест " .. vim.fn.shellescape(файл))
  end, { desc = "ZIP: тесты" })

  vim.keymap.set("n", "<leader>zp", function()
    local файл = vim.fn.expand("%:p")
    vim.cmd("!зип-профиль " .. vim.fn.shellescape(файл) .. " --топ 10")
  end, { desc = "ZIP: профилировать" })
end

return M
