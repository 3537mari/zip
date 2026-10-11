" Определение файлов ZIP-языка
augroup zip_ftdetect
    autocmd!
    autocmd BufNewFile,BufRead *.zip setfiletype zip
    autocmd BufNewFile,BufRead *.зип setfiletype zip
augroup END
