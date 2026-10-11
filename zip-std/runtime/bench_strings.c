/* ZIP STD: бенчмарк строк — 1 000 000 операций. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#ifdef ZIP_STRINGS_BENCH
int main(void) {
    const int N = 1000000;
    char *buf = (char *)malloc(64);
    if (!buf) return 1;
    size_t len = 0;
    size_t cap = 64;
    buf[0] = '\0';

    clock_t t0 = clock();
    unsigned h = 2166136261u;
    for (int i = 0; i < N; i++) {
        char tmp[32];
        int n = snprintf(tmp, sizeof(tmp), "%d,", i % 1000);
        if (len + (size_t)n + 1 > cap) {
            cap *= 2;
            char *nb = (char *)realloc(buf, cap);
            if (!nb) {
                free(buf);
                return 1;
            }
            buf = nb;
        }
        memcpy(buf + len, tmp, (size_t)n);
        len += (size_t)n;
        buf[len] = '\0';
        /* FNV-like hash + truncate to keep buffer bounded */
        for (int k = 0; k < n; k++) {
            h ^= (unsigned char)tmp[k];
            h *= 16777619u;
        }
        if (len > 4096) {
            len = 0;
            buf[0] = '\0';
        }
    }
    clock_t t1 = clock();
    long ms = (long)((t1 - t0) * 1000.0 / CLOCKS_PER_SEC);
    printf("%u\n", h);
    printf("%ld\n", ms);
    free(buf);
    return 0;
}
#endif
