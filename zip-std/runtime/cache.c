/* Runtime кэша результатов (Идея 2 — эпигенетика). */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    int64_t ключ;
    int64_t значение;
    int64_t занято;
} ZipЗаписьКэша;

typedef struct {
    ZipЗаписьКэша* записи;
    int64_t размер;
    int64_t попадания;
    int64_t промахи;
} ZipКэш;

ZipКэш* zip_создать_кэш(int64_t размер) {
    if (размер <= 0) размер = 16;
    ZipКэш* к = (ZipКэш*)malloc(sizeof(ZipКэш));
    if (!к) return NULL;
    к->размер = размер;
    к->записи = (ZipЗаписьКэша*)calloc((size_t)размер, sizeof(ZipЗаписьКэша));
    к->попадания = 0;
    к->промахи = 0;
    if (!к->записи) {
        free(к);
        return NULL;
    }
    return к;
}

int64_t zip_кэш_взять(ZipКэш* к, int64_t ключ, int* найден) {
    if (!к || !найден) return 0;
    for (int64_t i = 0; i < к->размер; i++) {
        if (к->записи[i].занято && к->записи[i].ключ == ключ) {
            к->попадания++;
            *найден = 1;
            return к->записи[i].значение;
        }
    }
    к->промахи++;
    *найден = 0;
    return 0;
}

void zip_кэш_положить(ZipКэш* к, int64_t ключ, int64_t значение) {
    if (!к) return;
    for (int64_t i = 0; i < к->размер; i++) {
        if (!к->записи[i].занято) {
            к->записи[i].ключ = ключ;
            к->записи[i].значение = значение;
            к->записи[i].занято = 1;
            return;
        }
        if (к->записи[i].ключ == ключ) {
            к->записи[i].значение = значение;
            return;
        }
    }
    /* FIFO: заменить первый */
    к->записи[0].ключ = ключ;
    к->записи[0].значение = значение;
    к->записи[0].занято = 1;
}

void zip_освободить_кэш(ZipКэш* к) {
    if (!к) return;
    free(к->записи);
    free(к);
}

#ifdef ZIP_CACHE_MAIN
int main(void) {
    ZipКэш* к = zip_создать_кэш(4);
    int найден = 0;
    zip_кэш_положить(к, 10, 100);
    int64_t v = zip_кэш_взять(к, 10, &найден);
    printf("hit=%d val=%lld\n", найден, (long long)v);
    (void)zip_кэш_взять(к, 99, &найден);
    printf("miss=%d hits=%lld misses=%lld\n",
           найден == 0, (long long)к->попадания, (long long)к->промахи);
    zip_освободить_кэш(к);
    return 0;
}
#endif
