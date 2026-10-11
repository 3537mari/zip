;;; zip-keymap.el --- Keymap for ZIP mode  -*- lexical-binding: t; -*-

;;; Commentary:
;; Горячие клавиши ZIP-режима.

;;; Code:

(defvar zip-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map (kbd "C-c C-r") #'zip-run)
    (define-key map (kbd "C-c C-b") #'zip-build)
    (define-key map (kbd "C-c C-t") #'zip-test)
    (define-key map (kbd "C-c C-p") #'zip-profile)
    map)
  "Карта клавиш ZIP-режима.")

(provide 'zip-keymap)
;;; zip-keymap.el ends here
