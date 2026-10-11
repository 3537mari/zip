" Подсветка синтаксиса ZIP для Neovim
if exists("b:current_syntax")
    finish
endif

" Ключевые слова
syn keyword zipKeyword если иначе пока для в возврат прервать продолжить
syn keyword zipKeyword функция тип модуль импорт глагол агент поток
syn keyword zipKeyword Нечто Ничто Успех Ошибка истина ложь
syn keyword zipKeyword и или не

" Операторы
syn match zipOperator ">>"
syn match zipOperator ">>>"
syn match zipOperator "||"
syn match zipOperator "!"
syn match zipOperator "~"
syn match zipOperator "=="
syn match zipOperator "!="
syn match zipOperator ">="
syn match zipOperator "<="
syn match zipOperator "—"
syn match zipOperator "\.\."

" Аннотации
syn match zipAnnotation "@[а-яА-ЯёЁ]\+"

" Числа
syn match zipNumber "\<\d\+\(\.\d\+\)\?\>"

" Строки
syn region zipString start=+"+ end=+"+ contains=zipEscape
syn match zipEscape "\\." contained

" Комментарии
syn match zipComment "//.*$"
syn region zipComment start="/\*" end="\*/"

" Корни (3 заглавные кириллические буквы)
syn match zipRoot "\<[А-ЯЁ]\{3\}\>"

" Шаблоны (СГСГ: САПАР)
syn match zipTemplate "\<[А-ЯЁ][а-яё][А-ЯЁ][а-яё]\>"

" Функции
syn match zipFunction "\<[а-яё][а-яё_0-9]*\ze\s*("

" Определение цветов
hi def link zipKeyword      Keyword
hi def link zipOperator     Operator
hi def link zipAnnotation   PreProc
hi def link zipNumber       Number
hi def link zipString       String
hi def link zipEscape       SpecialChar
hi def link zipComment      Comment
hi def link zipRoot         Constant
hi def link zipTemplate     Function
hi def link zipFunction     Identifier

let b:current_syntax = "zip"
