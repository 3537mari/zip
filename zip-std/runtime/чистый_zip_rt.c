/* Демо: «чистый ZIP» через C-runtime syscalls (сборка #runtime("чистый_zip")). */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <windows.h>
#endif

extern int64_t zip_syscall(int64_t, int64_t, int64_t, int64_t, int64_t, int64_t, int64_t);
extern int64_t zip_alloc(int64_t);
extern void zip_print_i64(int64_t);
extern void zip_print_str(const char *);
extern int64_t zip_file_write(const char *, const char *);
extern int64_t zip_file_read(const char *);

static void wstr(const char *s) {
    zip_syscall(1, 1, (int64_t)(intptr_t)s, (int64_t)strlen(s), 0, 0, 0);
}

#ifdef ZIP_CLEAN_MAIN
int main(void) {
    wstr("=== ЧИСТЫЙ ZIP ===\n");
    int64_t buf = zip_alloc(1024);
    if (buf != 0) wstr("Аллокатор: OK\n");
    wstr("Число: ");
    zip_print_i64(42);
    wstr("\n");

#ifdef _WIN32
    const char *path = "zip_clean_demo.txt";
#else
    const char *path = "/tmp/тест_zip.txt";
#endif
    zip_file_write(path, "Привет из чистого ZIP!\n");
    wstr("Файл записан\n");
    int64_t contents = zip_file_read(path);
    if (contents != 0) {
        wstr("Прочитано: ");
        wstr((const char *)(intptr_t)contents);
        free((void *)(intptr_t)contents);
    }
    return 0;
}
#endif
