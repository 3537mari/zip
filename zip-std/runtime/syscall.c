/* Единая точка входа для syscalls ZIP. */
#define _CRT_SECURE_NO_WARNINGS
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <windows.h>
#include <io.h>
#include <fcntl.h>

int64_t zip_syscall(int64_t номер, int64_t а1, int64_t а2, int64_t а3,
                    int64_t а4, int64_t а5, int64_t а6) {
    (void)а4; (void)а5; (void)а6;
    switch (номер) {
    case 0: { /* READ */
        DWORD прочитано = 0;
        HANDLE h = (HANDLE)(intptr_t)а1;
        if (а1 == 0) h = GetStdHandle(STD_INPUT_HANDLE);
        if (!ReadFile(h, (LPVOID)(intptr_t)а2, (DWORD)а3, &прочитано, NULL))
            return -1;
        return (int64_t)прочитано;
    }
    case 1: { /* WRITE */
        DWORD записано = 0;
        HANDLE h;
        if (а1 == 1) h = GetStdHandle(STD_OUTPUT_HANDLE);
        else if (а1 == 2) h = GetStdHandle(STD_ERROR_HANDLE);
        else h = (HANDLE)(intptr_t)а1;
        if (!WriteFile(h, (LPCVOID)(intptr_t)а2, (DWORD)а3, &записано, NULL))
            return -1;
        return (int64_t)записано;
    }
    case 2: /* OPEN */
        return (int64_t)_open((const char *)(intptr_t)а1, (int)а2, (int)а3);
    case 3: /* CLOSE */
        return (int64_t)_close((int)а1);
    case 9: { /* MMAP — VirtualAlloc; а1=addr(ignored), а2=size */
        size_t sz = (size_t)(а2 > 0 ? а2 : а1);
        if (sz == 0) sz = 4096;
        void *p = VirtualAlloc(NULL, sz, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        return p ? (int64_t)(intptr_t)p : -1;
    }
    case 12: /* BRK */
        return 0;
    case 60: /* EXIT */
    case 231:
        ExitProcess((UINT)а1);
        return 0;
    default:
        return -1;
    }
}
#else
#include <unistd.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#include <fcntl.h>

int64_t zip_syscall(int64_t номер, int64_t а1, int64_t а2, int64_t а3,
                    int64_t а4, int64_t а5, int64_t а6) {
    return (int64_t)syscall((long)номер, а1, а2, а3, а4, а5, а6);
}
#endif

/* Высокоуровневые обёртки для демо / #runtime */
int64_t zip_sys_write(int64_t fd, const void *buf, int64_t n) {
    return zip_syscall(1, fd, (int64_t)(intptr_t)buf, n, 0, 0, 0);
}

int64_t zip_sys_exit(int64_t код) {
    return zip_syscall(60, код, 0, 0, 0, 0, 0);
}

/* CSPRNG без OpenSSL — см. также crypto.c::zip_заполнить_случайными */
#if defined(_WIN32)
#include <bcrypt.h>
#pragma comment(lib, "bcrypt.lib")
int64_t zip_заполнить_случайными(void *буфер, int64_t количество) {
    if (!буфер || количество <= 0) return -1;
    NTSTATUS st = BCryptGenRandom(
        NULL, (PUCHAR)буфер, (ULONG)количество, BCRYPT_USE_SYSTEM_PREFERRED_RNG);
    return (st == 0) ? количество : -1;
}
#elif defined(__APPLE__)
#include <sys/random.h>
int64_t zip_заполнить_случайными(void *буфер, int64_t количество) {
    if (!буфер || количество <= 0) return -1;
    uint8_t *p = (uint8_t *)буфер;
    int64_t left = количество;
    while (left > 0) {
        size_t chunk = left > 256 ? 256 : (size_t)left;
        if (getentropy(p, chunk) != 0) return -1;
        p += chunk;
        left -= (int64_t)chunk;
    }
    return количество;
}
#else
#include <sys/random.h>
int64_t zip_заполнить_случайными(void *буфер, int64_t количество) {
    if (!буфер || количество <= 0) return -1;
    ssize_t n = getrandom(буфер, (size_t)количество, 0);
    return (n == количество) ? количество : -1;
}
#endif

#if defined(ZIP_SYSCALL_MAIN) && !defined(ZIP_SYSCALL_MAIN_DISABLED)
int main(void) {
    const char *msg = "=== ЧИСТЫЙ ZIP (syscall runtime) ===\n";
    zip_sys_write(1, msg, (int64_t)strlen(msg));
    zip_sys_write(1, "sys_write: OK\n", 14);
    return 0;
}
#endif
