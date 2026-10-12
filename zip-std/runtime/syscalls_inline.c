#define _CRT_SECURE_NO_WARNINGS
#include <stdint.h>

extern int64_t zip_syscall(int64_t, int64_t, int64_t, int64_t, int64_t, int64_t, int64_t);

#if defined(__x86_64__) && defined(__linux__)

int64_t zip_asm_syscall(int64_t номер, int64_t а1, int64_t а2, int64_t а3,
                        int64_t а4, int64_t а5, int64_t а6) {
    int64_t результат;
    __asm__ __volatile__(
        "mov %1, %%rax\n"
        "mov %2, %%rdi\n"
        "mov %3, %%rsi\n"
        "mov %4, %%rdx\n"
        "mov %5, %%r10\n"
        "mov %6, %%r8\n"
        "mov %7, %%r9\n"
        "syscall\n"
        "mov %%rax, %0\n"
        : "=r"(результат)
        : "r"(номер), "r"(а1), "r"(а2), "r"(а3), "r"(а4), "r"(а5), "r"(а6)
        : "rax", "rdi", "rsi", "rdx", "r10", "r8", "r9", "memory");
    return результат;
}

#else

int64_t zip_asm_syscall(int64_t номер, int64_t а1, int64_t а2, int64_t а3,
                        int64_t а4, int64_t а5, int64_t а6) {
    return zip_syscall(номер, а1, а2, а3, а4, а5, а6);
}

#endif

int64_t asm_syscall(int64_t n, int64_t a1, int64_t a2, int64_t a3, int64_t a4,
                    int64_t a5, int64_t a6) {
    return zip_asm_syscall(n, a1, a2, a3, a4, a5, a6);
}

#if defined(ZIP_SYSCALLS_INLINE_MAIN) && !defined(ZIP_SYSCALLS_INLINE_MAIN_DISABLED)
#include <stdio.h>
int main(void) {
    const char msg[] = "inline syscall ok\n";
    zip_asm_syscall(1, 1, (int64_t)(intptr_t)msg, (int64_t)(sizeof(msg) - 1), 0, 0, 0);
    return 0;
}
#endif
