/* Файловый I/O для ZIP runtime. */
#define _CRT_SECURE_NO_WARNINGS
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#include <sys/stat.h>
#else
#include <fcntl.h>
#include <unistd.h>
#endif

extern int64_t zip_alloc(int64_t размер);

int64_t zip_file_open(const char *путь, const char *режим) {
    if (!путь || !режим) return -1;
#ifdef _WIN32
    int flags = 0;
    if (режим[0] == 'r') flags = _O_RDONLY;
    else if (режим[0] == 'w') flags = _O_WRONLY | _O_CREAT | _O_TRUNC;
    else if (режим[0] == 'a') flags = _O_WRONLY | _O_CREAT | _O_APPEND;
    else return -1;
    return (int64_t)_open(путь, flags | _O_BINARY, _S_IREAD | _S_IWRITE);
#else
    int flags = 0;
    if (режим[0] == 'r') flags = O_RDONLY;
    else if (режим[0] == 'w') flags = O_WRONLY | O_CREAT | O_TRUNC;
    else if (режим[0] == 'a') flags = O_WRONLY | O_CREAT | O_APPEND;
    else return -1;
    return (int64_t)open(путь, flags, 0644);
#endif
}

int64_t zip_file_write(const char *путь, const char *данные) {
    int64_t fd = zip_file_open(путь, "w");
    if (fd < 0) return -1;
    size_t n = данные ? strlen(данные) : 0;
#ifdef _WIN32
    int w = _write((int)fd, данные, (unsigned)n);
    _close((int)fd);
#else
    ssize_t w = write((int)fd, данные, n);
    close((int)fd);
#endif
    return (int64_t)w;
}

int64_t zip_file_append(const char *путь, const char *данные) {
    int64_t fd = zip_file_open(путь, "a");
    if (fd < 0) return -1;
    size_t n = данные ? strlen(данные) : 0;
#ifdef _WIN32
    int w = _write((int)fd, данные, (unsigned)n);
    _close((int)fd);
#else
    ssize_t w = write((int)fd, данные, n);
    close((int)fd);
#endif
    return (int64_t)w;
}

/* Возвращает указатель на C-строку (через malloc; для демо). */
int64_t zip_file_read(const char *путь) {
    FILE *f = fopen(путь, "rb");
    if (!f) return 0;
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (sz < 0) { fclose(f); return 0; }
    char *buf = (char *)malloc((size_t)sz + 1);
    if (!buf) { fclose(f); return 0; }
    size_t rd = fread(buf, 1, (size_t)sz, f);
    buf[rd] = 0;
    fclose(f);
    return (int64_t)(intptr_t)buf;
}

#if defined(ZIP_FILES_MAIN) && !defined(ZIP_FILES_MAIN_DISABLED)
extern int64_t zip_syscall(int64_t, int64_t, int64_t, int64_t, int64_t, int64_t, int64_t);
int main(void) {
    const char *path =
#ifdef _WIN32
        "zip_rt_test.txt";
#else
        "/tmp/тест_zip.txt";
#endif
    zip_file_write(path, "Привет из чистого ZIP!\n");
    int64_t p = zip_file_read(path);
    if (p) {
        const char *s = (const char *)(intptr_t)p;
        zip_syscall(1, 1, (int64_t)(intptr_t)s, (int64_t)strlen(s), 0, 0, 0);
        free((void *)(intptr_t)p);
    }
    return 0;
}
#endif
