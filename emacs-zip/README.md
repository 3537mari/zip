# emacs-zip

Поддержка языка ZIP в Emacs.

## Возможности

- Подсветка синтаксиса (font-lock)
- Отступы
- LSP-интеграция (eglot или lsp-mode)
- Компиляция через `M-x compile` / горячие клавиши

## Установка

### straight.el

```elisp
(straight-use-package
 '(zip-mode :type git :host github :repo "3537mari/emacs-zip"
            :files ("*.el")))
```

Локально из монорепозитория:

```elisp
(add-to-list 'load-path "~/zip-lang/emacs-zip")
(require 'zip-mode)
```

### use-package

```elisp
(use-package zip-mode
  :load-path "~/zip-lang/emacs-zip"
  :mode ("\\.zip\\'" "\\.зип\\'")
  :hook (zip-mode . zip-lsp-setup))
```

## Горячие клавиши

| Клавиша | Действие |
|---------|----------|
| `C-c C-r` | Запустить программу |
| `C-c C-b` | Собрать программу |
| `C-c C-t` | Запустить тесты |
| `C-c C-p` | Профилировать |

## Требования

- Emacs 27+
- eglot (Emacs 29+) или lsp-mode
- ZIP-компилятор (`zipc`) в PATH
- LSP-сервер (`зип-lsp`) в PATH
