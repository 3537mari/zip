;;; zip-mode.el --- Major mode for ZIP language  -*- lexical-binding: t; -*-

;;; Commentary:
;; Поддержка языка ZIP в Emacs.
;; Подсветка, отступы, интеграция с LSP.

;;; Code:

(require 'rx)
(require 'zip-keymap)
(require 'zip-lsp)

(defgroup zip nil
  "Поддержка языка ZIP."
  :group 'languages)

(defcustom zip-indent-offset 4
  "Отступ в ZIP."
  :type 'integer
  :group 'zip)

(defvar zip-font-lock-keywords
  `(
    ;; Ключевые слова
    (,(rx symbol-start
          (or "если" "иначе" "пока" "для" "в" "возврат"
              "прервать" "продолжить" "функция" "тип"
              "модуль" "импорт" "глагол" "агент" "поток"
              "Нечто" "Ничто" "Успех" "Ошибка" "истина" "ложь"
              "и" "или" "не")
          symbol-end)
     . font-lock-keyword-face)

    ;; Операторы
    (,(rx (or ">>" ">>>" "||" "!" "~" "==" "!=" ">=" "<=" "—" ".."))
     . font-lock-builtin-face)

    ;; Аннотации
    (,(rx "@" (one-or-more (any "а-яА-ЯёЁ")))
     . font-lock-preprocessor-face)

    ;; Числа
    (,(rx word-start (one-or-more digit) (? "." (one-or-more digit)) word-end)
     . font-lock-constant-face)

    ;; Строки
    (,(rx "\"" (zero-or-more (or (not (any "\\\"")) (seq "\\" anything))) "\"")
     . font-lock-string-face)

    ;; Комментарии
    (,(rx "//" (zero-or-more not-newline))
     . font-lock-comment-face)

    ;; Корни (3 заглавные кириллические буквы)
    (,(rx symbol-start (repeat 3 (any "А-ЯЁ")) symbol-end)
     . font-lock-constant-face)

    ;; Шаблоны (СГСГ)
    (,(rx symbol-start
          (any "А-ЯЁ") (any "а-яё") (any "А-ЯЁ") (any "а-яё")
          symbol-end)
     . font-lock-function-name-face)
    )
  "Правила подсветки ZIP.")

;;;###autoload
(define-derived-mode zip-mode prog-mode "ZIP"
  "Основной режим для ZIP."
  :group 'zip

  (setq-local comment-start "// ")
  (setq-local comment-end "")
  (setq-local indent-tabs-mode nil)
  (setq-local tab-width zip-indent-offset)
  (setq-local font-lock-defaults '(zip-font-lock-keywords))

  (when (fboundp 'zip-lsp-setup)
    (zip-lsp-setup)))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.zip\\'" . zip-mode))
;;;###autoload
(add-to-list 'auto-mode-alist '("\\.зип\\'" . zip-mode))

(defun zip-run ()
  "Запустить текущую ZIP-программу."
  (interactive)
  (let ((file (buffer-file-name)))
    (unless file
      (user-error "Буфер не связан с файлом"))
    (compile (format "zipc запустить %s --llvm"
                     (shell-quote-argument file)))))

(defun zip-build ()
  "Собрать текущую ZIP-программу."
  (interactive)
  (let* ((file (buffer-file-name))
         (output (and file (file-name-sans-extension file))))
    (unless file
      (user-error "Буфер не связан с файлом"))
    (compile (format "zipc собрать %s -o %s --llvm"
                     (shell-quote-argument file)
                     (shell-quote-argument output)))))

(defun zip-test ()
  "Запустить тесты."
  (interactive)
  (let ((file (buffer-file-name)))
    (unless file
      (user-error "Буфер не связан с файлом"))
    (compile (format "zipc тест %s --llvm"
                     (shell-quote-argument file)))))

(defun zip-profile ()
  "Профилировать текущую программу."
  (interactive)
  (let ((file (buffer-file-name)))
    (unless file
      (user-error "Буфер не связан с файлом"))
    (compile (format "зип-профиль %s --топ 10"
                     (shell-quote-argument file)))))

(provide 'zip-mode)
;;; zip-mode.el ends here
