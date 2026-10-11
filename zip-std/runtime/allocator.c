/* Минимальный C-helper для аллокатора ZIP (bump + mmap/VirtualAlloc). */
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <sys/mman.h>
#endif

extern int64_t zip_syscall(int64_t, int64_t, int64_t, int64_t, int64_t, int64_t, int64_t);

static uint8_t *g_base = NULL;
static size_t g_size = 0;
static size_t g_used = 0;

static void *rt_mmap(size_t n) {
#ifdef _WIN32
    return VirtualAlloc(NULL, n, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
#else
    void *p = mmap(NULL, n, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    return p == MAP_FAILED ? NULL : p;
#endif
}

void *zip_alloc_structure(size_t размер) {
    return calloc(1, размер);
}

void *zip_memcpy(void *назначение, const void *источник, size_t длина) {
    return memcpy(назначение, источник, длина);
}

void *zip_memset(void *буфер, int значение, size_t длина) {
    return memset(буфер, значение, длина);
}

int64_t zip_alloc(int64_t размер) {
    size_t n = (size_t)размер;
    if (n == 0) n = 1;
    n = (n + 15) & ~(size_t)15;
    if (!g_base) {
        g_size = 1024 * 1024;
        g_base = (uint8_t *)rt_mmap(g_size);
        g_used = 0;
        if (!g_base) return 0;
    }
    if (g_used + n > g_size) {
        g_size = 16 * 1024 * 1024;
        g_base = (uint8_t *)rt_mmap(g_size);
        g_used = 0;
        if (!g_base) return 0;
    }
    int64_t addr = (int64_t)(intptr_t)(g_base + g_used);
    g_used += n;
    return addr;
}

int64_t zip_alloc_used(void) {
    return (int64_t)g_used;
}

#if defined(ZIP_ALLOC_MAIN) && !defined(ZIP_ALLOC_MAIN_DISABLED)
#ifdef _WIN32
#include <windows.h>
#else
#include <unistd.h>
#endif
int main(void) {
    int64_t a = zip_alloc(1024);
    int64_t b = zip_alloc(2048);
    if (a != 0 && b != 0 && a != b) {
        const char *ok = "allocator: OK\n";
#ifdef _WIN32
        DWORD w;
        WriteFile(GetStdHandle(STD_OUTPUT_HANDLE), ok, (DWORD)strlen(ok), &w, NULL);
#else
        write(1, ok, strlen(ok));
#endif
        return 0;
    }
    return 1;
}
#endif
