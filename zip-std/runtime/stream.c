/* Runtime для потоковой обработки (Идея 16). */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct ZipУзелПотока {
    void* данные;
    int64_t длина;
    struct ZipУзелПотока* следующий;
} ZipУзелПотока;

typedef struct {
    ZipУзелПотока* голова;
    ZipУзелПотока* хвост;
    int64_t длина;
} ZipПоток;

ZipПоток* zip_создать_поток(void) {
    ZipПоток* п = (ZipПоток*)malloc(sizeof(ZipПоток));
    if (!п) return NULL;
    п->голова = NULL;
    п->хвост = NULL;
    п->длина = 0;
    return п;
}

void zip_поток_добавить(ZipПоток* п, void* данные, int64_t длина) {
    if (!п) return;
    ZipУзелПотока* узел = (ZipУзелПотока*)malloc(sizeof(ZipУзелПотока));
    if (!узел) return;
    узел->данные = данные;
    узел->длина = длина;
    узел->следующий = NULL;
    if (!п->голова) {
        п->голова = узел;
        п->хвост = узел;
    } else {
        п->хвост->следующий = узел;
        п->хвост = узел;
    }
    п->длина++;
}

void zip_поток_применить(ZipПоток* п, void* (*функция)(void*, int64_t)) {
    if (!п || !функция) return;
    ZipУзелПотока* текущий = п->голова;
    while (текущий) {
        текущий->данные = функция(текущий->данные, текущий->длина);
        текущий = текущий->следующий;
    }
}

void* zip_поток_собрать(ZipПоток* п, int64_t* количество) {
    if (!п) {
        if (количество) *количество = 0;
        return NULL;
    }
    void** результат = (void**)malloc((size_t)п->длина * sizeof(void*));
    if (!результат) {
        if (количество) *количество = 0;
        return NULL;
    }
    int64_t i = 0;
    ZipУзелПотока* текущий = п->голова;
    while (текущий) {
        результат[i++] = текущий->данные;
        текущий = текущий->следующий;
    }
    if (количество) *количество = п->длина;
    return результат;
}

#ifdef ZIP_STREAM_MAIN
int main(void) {
    ZipПоток* п = zip_создать_поток();
    int64_t a = 1, b = 2, c = 3;
    zip_поток_добавить(п, &a, sizeof(a));
    zip_поток_добавить(п, &b, sizeof(b));
    zip_поток_добавить(п, &c, sizeof(c));
    int64_t n = 0;
    void** out = (void**)zip_поток_собрать(п, &n);
    printf("stream length: %lld\n", (long long)n);
    free(out);
    free(п);
    return 0;
}
#endif
