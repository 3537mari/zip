/* Runtime Date/Time ZIP STD. */
#define _CRT_SECURE_NO_WARNINGS
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <sys/time.h>
#endif

int64_t zip_время_сейчас(void) {
#ifdef _WIN32
    FILETIME ft;
    ULARGE_INTEGER uli;
    GetSystemTimeAsFileTime(&ft);
    uli.LowPart = ft.dwLowDateTime;
    uli.HighPart = ft.dwHighDateTime;
    return (int64_t)(uli.QuadPart / 10000ULL) - 11644473600000LL;
#else
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (int64_t)tv.tv_sec * 1000 + tv.tv_usec / 1000;
#endif
}

typedef struct {
    int год;
    int месяц;
    int день;
    int час;
    int минута;
    int секунда;
    int миллисекунда;
    int день_недели;
    int день_года;
} ZipDateTime;

static void zip_localtime(time_t t, struct tm *out) {
#ifdef _WIN32
    localtime_s(out, &t);
#else
    localtime_r(&t, out);
#endif
}

ZipDateTime *zip_datetime_разобрать(int64_t timestamp_мс) {
    ZipDateTime *дт;
    struct tm тм;
    time_t сек;
    if (timestamp_мс < 0) return NULL;
    дт = (ZipDateTime *)calloc(1, sizeof(ZipDateTime));
    if (!дт) return NULL;
    сек = (time_t)(timestamp_мс / 1000);
    дт->миллисекунда = (int)(timestamp_мс % 1000);
    zip_localtime(сек, &тм);
    дт->год = тм.tm_year + 1900;
    дт->месяц = тм.tm_mon + 1;
    дт->день = тм.tm_mday;
    дт->час = тм.tm_hour;
    дт->минута = тм.tm_min;
    дт->секунда = тм.tm_sec;
    дт->день_недели = тм.tm_wday;
    дт->день_года = тм.tm_yday;
    return дт;
}

void zip_datetime_освободить(ZipDateTime *дт) { free(дт); }

int zip_datetime_год(ZipDateTime *д) { return д ? д->год : 0; }
int zip_datetime_месяц(ZipDateTime *д) { return д ? д->месяц : 0; }
int zip_datetime_день(ZipDateTime *д) { return д ? д->день : 0; }
int zip_datetime_час(ZipDateTime *д) { return д ? д->час : 0; }
int zip_datetime_минута(ZipDateTime *д) { return д ? д->минута : 0; }
int zip_datetime_секунда(ZipDateTime *д) { return д ? д->секунда : 0; }

int64_t zip_datetime_собрать(int год, int месяц, int день, int час, int минута,
                             int секунда) {
    struct tm тм;
    time_t сек;
    memset(&тм, 0, sizeof(тм));
    тм.tm_year = год - 1900;
    тм.tm_mon = месяц - 1;
    тм.tm_mday = день;
    тм.tm_hour = час;
    тм.tm_min = минута;
    тм.tm_sec = секунда;
    тм.tm_isdst = -1;
    сек = mktime(&тм);
    if (сек == (time_t)-1) return -1;
    return (int64_t)сек * 1000;
}

char *zip_datetime_формат(int64_t timestamp_мс, const char *формат) {
    time_t сек;
    struct tm тм;
    char *буфер;
    if (!формат || timestamp_мс < 0) return NULL;
    сек = (time_t)(timestamp_мс / 1000);
    zip_localtime(сек, &тм);
    буфер = (char *)malloc(256);
    if (!буфер) return NULL;
    if (strftime(буфер, 256, формат, &тм) == 0) {
        буфер[0] = '\0';
    }
    return буфер;
}

/* Упрощённый парсер: поддерживает %Y %m %d %H %M %S и литералы. */
int64_t zip_datetime_парсить(const char *строка, const char *формат) {
    const char *s = строка;
    const char *f = формат;
    int год = 1970, месяц = 1, день = 1, час = 0, минута = 0, секунда = 0;
    if (!строка || !формат) return -1;
    while (*f && *s) {
        if (*f == '%') {
            f++;
            if (*f == 'Y') {
                if (sscanf(s, "%4d", &год) != 1) return -1;
                s += 4;
            } else if (*f == 'm') {
                if (sscanf(s, "%2d", &месяц) != 1) return -1;
                s += 2;
            } else if (*f == 'd') {
                if (sscanf(s, "%2d", &день) != 1) return -1;
                s += 2;
            } else if (*f == 'H') {
                if (sscanf(s, "%2d", &час) != 1) return -1;
                s += 2;
            } else if (*f == 'M') {
                if (sscanf(s, "%2d", &минута) != 1) return -1;
                s += 2;
            } else if (*f == 'S') {
                if (sscanf(s, "%2d", &секунда) != 1) return -1;
                s += 2;
            } else if (*f == '%') {
                if (*s != '%') return -1;
                s++;
            } else {
                return -1;
            }
            f++;
        } else {
            if (*f != *s) return -1;
            f++;
            s++;
        }
    }
    if (*f != '\0') return -1;
    return zip_datetime_собрать(год, месяц, день, час, минута, секунда);
}

int64_t zip_datetime_добавить(int64_t timestamp_мс, int64_t интервал_мс) {
    return timestamp_мс + интервал_мс;
}

int64_t zip_datetime_разница(int64_t т1, int64_t т2) { return т1 - т2; }

int64_t zip_секунда(int64_t n) { return n * 1000LL; }
int64_t zip_минута(int64_t n) { return n * 60LL * 1000LL; }
int64_t zip_час(int64_t n) { return n * 3600LL * 1000LL; }
int64_t zip_день(int64_t n) { return n * 86400LL * 1000LL; }
int64_t zip_неделя(int64_t n) { return n * 604800LL * 1000LL; }

#ifdef ZIP_DATETIME_MAIN
int main(void) {
    int64_t т = zip_время_сейчас();
    char *s = zip_datetime_формат(т, "%Y-%m-%d %H:%M:%S");
    ZipDateTime *д = zip_datetime_разобрать(т);
    int64_t завтра = zip_datetime_добавить(т, zip_день(1));
    char *s2 = zip_datetime_формат(завтра, "%d.%m.%Y");
    int64_t разн = zip_datetime_разница(завтра, т);
    printf("Сейчас:\n%s\n", s ? s : "?");
    printf("Год:\n%d\n", д ? д->год : 0);
    printf("Месяц:\n%d\n", д ? д->месяц : 0);
    printf("Завтра:\n%s\n", s2 ? s2 : "?");
    printf("Разница в часах:\n%lld\n", (long long)(разн / zip_час(1)));
    free(s);
    free(s2);
    zip_datetime_освободить(д);
    return 0;
}
#endif

#ifdef ZIP_DATETIME_UNITTEST
static int fails;
#define CHECK(c)                                                               \
    do {                                                                       \
        if (!(c)) {                                                            \
            fprintf(stderr, "FAIL %s:%d %s\n", __FILE__, __LINE__, #c);        \
            fails++;                                                           \
        }                                                                      \
    } while (0)

int main(void) {
    {
        int64_t т = zip_время_сейчас();
        ZipDateTime *д = zip_datetime_разобрать(т);
        CHECK(д && д->год >= 2026);
        zip_datetime_освободить(д);
    }
    {
        int64_t т = zip_datetime_собрать(2026, 10, 11, 15, 30, 0);
        char *s = zip_datetime_формат(т, "%Y-%m-%d %H:%M:%S");
        CHECK(s && strcmp(s, "2026-10-11 15:30:00") == 0);
        free(s);
    }
    {
        int64_t т = zip_datetime_собрать(2026, 1, 1, 0, 0, 0);
        int64_t завтра = zip_datetime_добавить(т, zip_день(1));
        ZipDateTime *д = zip_datetime_разобрать(завтра);
        CHECK(д && д->день == 2);
        zip_datetime_освободить(д);
    }
    {
        int64_t т = zip_datetime_парсить("2026-10-11 15:30:00", "%Y-%m-%d %H:%M:%S");
        ZipDateTime *д = zip_datetime_разобрать(т);
        CHECK(д && д->год == 2026 && д->месяц == 10 && д->день == 11);
        CHECK(д->час == 15 && д->минута == 30);
        zip_datetime_освободить(д);
    }
    {
        int64_t a = zip_datetime_собрать(2026, 1, 1, 0, 0, 0);
        int64_t b = zip_datetime_добавить(a, zip_час(5));
        CHECK(zip_datetime_разница(b, a) == zip_час(5));
    }
    {
        CHECK(zip_секунда(2) == 2000);
        CHECK(zip_минута(1) == 60000);
        CHECK(zip_неделя(1) == zip_день(7));
    }
    if (fails) {
        fprintf(stderr, "%d failed\n", fails);
        return 1;
    }
    printf("datetime unit tests ok\n");
    return 0;
}
#endif
