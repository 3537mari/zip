/* Минимальные обёртки I/O для ZIP runtime. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <unistd.h>
#endif

extern int64_t zip_syscall(int64_t, int64_t, int64_t, int64_t, int64_t, int64_t, int64_t);

size_t zip_strlen(const char *строка) {
    return строка ? strlen(строка) : 0;
}

void zip_print_i64(int64_t n) {
    char buf[32];
    int i = 0;
    int neg = 0;
    if (n == 0) {
        buf[0] = '0';
        i = 1;
    } else {
        if (n < 0) { neg = 1; n = -n; }
        while (n > 0 && i < 30) {
            buf[i++] = (char)('0' + (n % 10));
            n /= 10;
        }
        if (neg) buf[i++] = '-';
        for (int a = 0, b = i - 1; a < b; a++, b--) {
            char t = buf[a]; buf[a] = buf[b]; buf[b] = t;
        }
    }
    zip_syscall(1, 1, (int64_t)(intptr_t)buf, i, 0, 0, 0);
}

void zip_print_str(const char *s) {
    if (!s) return;
    zip_syscall(1, 1, (int64_t)(intptr_t)s, (int64_t)strlen(s), 0, 0, 0);
}

#if defined(ZIP_IO_MAIN) && !defined(ZIP_IO_MAIN_DISABLED)
int main(void) {
    zip_print_str("число: ");
    zip_print_i64(42);
    zip_print_str("\n");
    return 0;
}
#endif
