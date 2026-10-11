/* Runtime для агентов (Идея 17). */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(_WIN32)
#include <windows.h>
typedef HANDLE zip_thread_t;
#else
#include <pthread.h>
typedef pthread_t zip_thread_t;
#endif

typedef struct {
    void* состояние;
    int64_t активен;
    zip_thread_t поток;
} ZipАгент;

ZipАгент* zip_создать_агент(void* начальное_состояние) {
    ZipАгент* а = (ZipАгент*)malloc(sizeof(ZipАгент));
    if (!а) return NULL;
    а->состояние = начальное_состояние;
    а->активен = 1;
    memset(&а->поток, 0, sizeof(а->поток));
    return а;
}

#if defined(_WIN32)
static DWORD WINAPI zip_agent_trampoline(LPVOID arg) {
    void* (*функция)(void*) = (void* (*)(void*))arg;
    /* unused in MVP stub */
    (void)функция;
    return 0;
}
#endif

void zip_агент_запустить(ZipАгент* а, void* (*функция)(void*)) {
    if (!а || !функция) return;
#if defined(_WIN32)
    (void)а;
    (void)функция;
    /* MVP: синхронный вызов без фонового потока на Windows */
    функция(а);
#else
    pthread_create(&а->поток, NULL, функция, а);
#endif
}

void zip_агент_остановить(ZipАгент* а) {
    if (!а) return;
    а->активен = 0;
#if !defined(_WIN32)
    if (а->поток) pthread_join(а->поток, NULL);
#else
    (void)а;
#endif
}

void zip_агент_отправить(const char* сообщение) {
    fprintf(stdout, "[Агент] %s\n", сообщение ? сообщение : "");
}

#ifdef ZIP_AGENT_MAIN
static void* demo_loop(void* arg) {
    ZipАгент* а = (ZipАгент*)arg;
    zip_агент_отправить("агент запущен");
    а->активен = 0;
    return NULL;
}

int main(void) {
    ZipАгент* а = zip_создать_агент(NULL);
    zip_агент_запустить(а, demo_loop);
    zip_агент_остановить(а);
    free(а);
    return 0;
}
#endif
