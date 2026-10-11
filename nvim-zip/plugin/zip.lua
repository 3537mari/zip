-- Точка входа плагина (загружается автоматически)
if vim.g.loaded_zip_plugin then
  return
end
vim.g.loaded_zip_plugin = true

-- Плагин активируется через require("zip").setup({...})
