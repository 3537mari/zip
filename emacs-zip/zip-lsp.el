;;; zip-lsp.el --- LSP integration for ZIP  -*- lexical-binding: t; -*-

;;; Commentary:
;; Интеграция с LSP-сервером ZIP через lsp-mode или eglot.

;;; Code:

(defgroup zip-lsp nil
  "LSP для ZIP."
  :group 'languages)

(defcustom zip-lsp-executable "зип-lsp"
  "Путь к LSP-серверу ZIP."
  :type 'string
  :group 'zip-lsp)

;;;###autoload
(defun zip-lsp-setup ()
  "Настроить LSP для ZIP-режима."
  (cond
   ;; eglot (встроен в Emacs 29+)
   ((fboundp 'eglot-ensure)
    (require 'eglot nil t)
    (when (boundp 'eglot-server-programs)
      (add-to-list 'eglot-server-programs
                   `(zip-mode . (,zip-lsp-executable))))
    (eglot-ensure))

   ;; lsp-mode
   ((fboundp 'lsp)
    (require 'lsp-mode)
    (add-to-list 'lsp-language-id-configuration '(zip-mode . "zip"))
    (lsp-register-client
     (make-lsp-client
      :new-connection (lsp-stdio-connection
                       (lambda () (list zip-lsp-executable)))
      :activation-fn (lsp-activate-on "zip")
      :server-id 'zip-lsp))
    (lsp-deferred))))

(provide 'zip-lsp)
;;; zip-lsp.el ends here
