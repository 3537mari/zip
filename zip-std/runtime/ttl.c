/* Runtime для временных значений (Идея 12 — TTL). */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <time.h>
#endif

typedef struct {
    int64_t данные;
    int64_t время_создания; /* Unix timestamp (мс) */
    int64_t ttl_мс;
    int64_t активно; /* 1 = активно, 0 = истекло */
} ZipTTL;

static int64_t zip_текущее_время_мс(void) {
#ifdef _WIN32
    FILETIME ft;
    ULARGE_INTEGER uli;
    GetSystemTimeAsFileTime(&ft);
    uli.LowPart = ft.dwLowDateTime;
    uli.HighPart = ft.dwHighDateTime;
    /* 100-нс интервалы с 1601-01-01 → мс с 1970-01-01 */
    return (int64_t)((uli.QuadPart - 116444736000000000ULL) / 10000ULL);
#else
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    return (int64_t)ts.tv_sec * 1000 + (int64_t)ts.tv_nsec / 1000000;
#endif
}

ZipTTL* zip_создать_ttl(int64_t данные, int64_t ttl_мс) {
    ZipTTL* т = (ZipTTL*)malloc(sizeof(ZipTTL));
    if (!т) return NULL;
    т->данные = данные;
    т->время_создания = zip_текущее_время_мс();
    т->ttl_мс = ttl_мс;
    т->активно = 1;
    return т;
}

int64_t zip_ttl_жив(ZipTTL* т) {
    if (!т || !т->активно) return 0;
    int64_t прошло = zip_текущее_время_мс() - т->время_создания;
    if (прошло > т->ttl_мс) {
        т->активно = 0;
        return 0;
    }
    return 1;
}

int64_t zip_ttl_получить(ZipTTL* т) {
    if (!zip_ttl_жив(т)) return 0;
    return т->данные;
}

void zip_ttl_освободить(ZipTTL* т) {
    if (!т) return;
    т->активно = 0;
    free(т);
}

#ifdef ZIP_TTL_MAIN
int main(void) {
    ZipTTL* кэш = zip_создать_ttl(42, 5000);
    if (!zip_ttl_жив(кэш)) return 1;
    printf("%lld\n", (long long)zip_ttl_получить(кэш));
    zip_ttl_освободить(кэш);
    return 0;
}
#endif
