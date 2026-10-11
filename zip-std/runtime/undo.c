/* Runtime для автоматических транзакций (Идея 1 — комплементарные корни). */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    void** данные;
    int64_t длина;
    int64_t ёмкость;
} ZipСтек;

ZipСтек* zip_создать_стек(void) {
    ZipСтек* с = (ZipСтек*)malloc(sizeof(ZipСтек));
    if (!с) return NULL;
    с->ёмкость = 16;
    с->длина = 0;
    с->данные = (void**)malloc((size_t)с->ёмкость * sizeof(void*));
    if (!с->данные) {
        free(с);
        return NULL;
    }
    return с;
}

void zip_стек_положить(ZipСтек* с, void* элемент) {
    if (!с) return;
    if (с->длина >= с->ёмкость) {
        int64_t нов = с->ёмкость * 2;
        void** p = (void**)realloc(с->данные, (size_t)нов * sizeof(void*));
        if (!p) return;
        с->данные = p;
        с->ёмкость = нов;
    }
    с->данные[с->длина++] = элемент;
}

void* zip_стек_взять(ZipСтек* с) {
    if (!с || с->длина == 0) return NULL;
    return с->данные[--с->длина];
}

/* Автоматический откат: применяет обратные операции со стека. */
void zip_откатить(ZipСтек* с) {
    if (!с) return;
    while (с->длина > 0) {
        void* оп = zip_стек_взять(с);
        if (!оп) continue;
        void (*обратная)(void*) = (void (*)(void*))оп;
        обратная(NULL);
    }
}

void zip_освободить_стек(ZipСтек* с) {
    if (!с) return;
    free(с->данные);
    free(с);
}

#ifdef ZIP_UNDO_MAIN
static int64_t счётчик;

static void undo_inc(void* _) {
    (void)_;
    счётчик -= 1;
}

static void do_inc(ZipСтек* стек) {
    счётчик += 1;
    zip_стек_положить(стек, (void*)undo_inc);
}

int main(void) {
    ZipСтек* стек = zip_создать_стек();
    счётчик = 0;
    do_inc(стек);
    do_inc(стек);
    do_inc(стек);
    printf("после: %lld\n", (long long)счётчик);
    zip_откатить(стек);
    printf("откат: %lld\n", (long long)счётчик);
    zip_освободить_стек(стек);
    return счётчик == 0 ? 0 : 1;
}
#endif
