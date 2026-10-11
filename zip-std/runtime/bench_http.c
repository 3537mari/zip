/* ZIP STD: микробенч HTTP — 10 000 разборов GET-запроса (без сети). */
#include <stdio.h>
#include <string.h>
#include <time.h>

#ifdef ZIP_HTTP_BENCH
static int parse_get(const char *req) {
    /* минимальный разбор: метод + путь */
    if (strncmp(req, "GET ", 4) != 0) return -1;
    const char *p = req + 4;
    if (*p != '/') return -1;
    while (*p && *p != ' ') p++;
    return (int)(p - (req + 4));
}

int main(void) {
    const char *req = "GET /api/v1/item/42 HTTP/1.1\r\nHost: localhost\r\n\r\n";
    const int N = 5000000;
    clock_t t0 = clock();
    long sum = 0;
    for (int i = 0; i < N; i++) {
        sum += parse_get(req);
    }
    clock_t t1 = clock();
    long ms = (long)((t1 - t0) * 1000.0 / CLOCKS_PER_SEC);
    printf("%ld\n", sum);
    printf("%ld\n", ms);
    return 0;
}
#endif
