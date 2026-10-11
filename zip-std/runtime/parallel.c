/* Runtime параллелизма (Идея 2 — эпигенетика).
 * На Windows без pthread — последовательный fallback для ZIP_PARALLEL_MAIN.
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#ifndef _WIN32
#include <pthread.h>
#endif

typedef void (*ZipFn)(void*);

typedef struct {
    ZipFn функция;
    void* аргумент;
} ZipЗадача;

typedef struct {
#ifndef _WIN32
    pthread_t* потоки;
#endif
    int64_t количество;
} ZipПулПотоков;

#ifndef _WIN32
static void* zip_обёртка_потока(void* arg) {
    ZipЗадача* задача = (ZipЗадача*)arg;
    if (задача && задача->функция) {
        задача->функция(задача->аргумент);
    }
    free(задача);
    return NULL;
}
#endif

ZipПулПотоков* zip_создать_пул(int64_t количество) {
    if (количество <= 0) количество = 1;
    ZipПулПотоков* п = (ZipПулПотоков*)malloc(sizeof(ZipПулПотоков));
    if (!п) return NULL;
    п->количество = количество;
#ifndef _WIN32
    п->потоки = (pthread_t*)calloc((size_t)количество, sizeof(pthread_t));
    if (!п->потоки) {
        free(п);
        return NULL;
    }
#endif
    return п;
}

void zip_запустить_параллельно(ZipПулПотоков* п, ZipFn функция, void** аргументы) {
    if (!п || !функция) return;
#ifdef _WIN32
    for (int64_t i = 0; i < п->количество; i++) {
        void* a = аргументы ? аргументы[i] : NULL;
        функция(a);
    }
#else
    for (int64_t i = 0; i < п->количество; i++) {
        ZipЗадача* задача = (ZipЗадача*)malloc(sizeof(ZipЗадача));
        задача->функция = функция;
        задача->аргумент = аргументы ? аргументы[i] : NULL;
        pthread_create(&п->потоки[i], NULL, zip_обёртка_потока, задача);
    }
#endif
}

void zip_дождаться_пул(ZipПулПотоков* п) {
#ifndef _WIN32
    if (!п) return;
    for (int64_t i = 0; i < п->количество; i++) {
        pthread_join(п->потоки[i], NULL);
    }
#else
    (void)п;
#endif
}

void zip_освободить_пул(ZipПулПотоков* п) {
    if (!п) return;
#ifndef _WIN32
    free(п->потоки);
#endif
    free(п);
}

#ifdef ZIP_PARALLEL_MAIN
static int64_t сумма;

static void работник(void* arg) {
    int64_t v = arg ? *(int64_t*)arg : 1;
    сумма += v;
}

int main(void) {
    int64_t args_data[4] = {1, 2, 3, 4};
    void* args[4];
    for (int i = 0; i < 4; i++) args[i] = &args_data[i];
    сумма = 0;
    ZipПулПотоков* п = zip_создать_пул(4);
    zip_запустить_параллельно(п, работник, args);
    zip_дождаться_пул(п);
    printf("sum=%lld\n", (long long)сумма);
    zip_освободить_пул(п);
    return сумма == 10 ? 0 : 1;
}
#endif
