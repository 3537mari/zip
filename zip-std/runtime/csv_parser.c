/* Runtime CSV-парсера ZIP STD. */
#define _CRT_SECURE_NO_WARNINGS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char ***строки;
    int *количество_колонок;
    int количество_строк;
    int ёмкость_строк;
} ZipCsv;

static int zip_csv_обеспечить(ZipCsv *т) {
    char ***нс;
    int *нк;
    int новая;
    if (т->количество_строк < т->ёмкость_строк) return 1;
    новая = т->ёмкость_строк ? т->ёмкость_строк * 2 : 16;
    нс = (char ***)realloc(т->строки, (size_t)новая * sizeof(char **));
    нк = (int *)realloc(т->количество_колонок, (size_t)новая * sizeof(int));
    if (!нс || !нк) return 0;
    т->строки = нс;
    т->количество_колонок = нк;
    т->ёмкость_строк = новая;
    return 1;
}

ZipCsv *zip_csv_создать(void) {
    ZipCsv *т = (ZipCsv *)calloc(1, sizeof(ZipCsv));
    if (!т) return NULL;
    т->ёмкость_строк = 16;
    т->строки = (char ***)calloc((size_t)т->ёмкость_строк, sizeof(char **));
    т->количество_колонок =
        (int *)calloc((size_t)т->ёмкость_строк, sizeof(int));
    if (!т->строки || !т->количество_колонок) {
        free(т->строки);
        free(т->количество_колонок);
        free(т);
        return NULL;
    }
    return т;
}

void zip_csv_освободить(ZipCsv *т) {
    int i, j;
    if (!т) return;
    for (i = 0; i < т->количество_строк; i++) {
        if (!т->строки[i]) continue;
        for (j = 0; j < т->количество_колонок[i]; j++) {
            free(т->строки[i][j]);
        }
        free(т->строки[i]);
    }
    free(т->строки);
    free(т->количество_колонок);
    free(т);
}

static char *zip_csv_dup_n(const char *s, size_t n) {
    char *p = (char *)malloc(n + 1);
    if (!p) return NULL;
    memcpy(p, s, n);
    p[n] = '\0';
    return p;
}

ZipCsv *zip_csv_разобрать(const char *текст, char разделитель) {
    ZipCsv *т;
    const char *p;
    if (!текст) return NULL;
    т = zip_csv_создать();
    if (!т) return NULL;
    p = текст;
    while (*p) {
        char **поля = NULL;
        int кол = 0, ёмк = 0;
        if (*p == '\r') p++;
        if (*p == '\n') {
            p++;
            continue;
        }
        while (*p && *p != '\n' && *p != '\r') {
            char *поле = NULL;
            if (*p == '"') {
                size_t cap = 32, len = 0;
                p++;
                поле = (char *)malloc(cap);
                if (!поле) {
                    zip_csv_освободить(т);
                    return NULL;
                }
                while (*p) {
                    if (*p == '"') {
                        if (*(p + 1) == '"') {
                            if (len + 1 >= cap) {
                                char *n = (char *)realloc(поле, cap *= 2);
                                if (!n) {
                                    free(поле);
                                    zip_csv_освободить(т);
                                    return NULL;
                                }
                                поле = n;
                            }
                            поле[len++] = '"';
                            p += 2;
                            continue;
                        }
                        p++;
                        break;
                    }
                    if (len + 1 >= cap) {
                        char *n = (char *)realloc(поле, cap *= 2);
                        if (!n) {
                            free(поле);
                            zip_csv_освободить(т);
                            return NULL;
                        }
                        поле = n;
                    }
                    поле[len++] = *p++;
                }
                поле[len] = '\0';
            } else {
                const char *start = p;
                while (*p && *p != разделитель && *p != '\n' && *p != '\r') {
                    p++;
                }
                поле = zip_csv_dup_n(start, (size_t)(p - start));
            }
            if (кол >= ёмк) {
                int nёмк = ёмк ? ёмк * 2 : 8;
                char **nf = (char **)realloc(поля, (size_t)nёмк * sizeof(char *));
                if (!nf) {
                    free(поле);
                    zip_csv_освободить(т);
                    return NULL;
                }
                поля = nf;
                ёмк = nёмк;
            }
            поля[кол++] = поле;
            if (*p == разделитель) {
                p++;
            } else {
                break;
            }
        }
        if (!zip_csv_обеспечить(т)) {
            int j;
            for (j = 0; j < кол; j++) free(поля[j]);
            free(поля);
            zip_csv_освободить(т);
            return NULL;
        }
        т->строки[т->количество_строк] = поля;
        т->количество_колонок[т->количество_строк] = кол;
        т->количество_строк++;
        if (*p == '\r') p++;
        if (*p == '\n') p++;
    }
    return т;
}

const char *zip_csv_получить(ZipCsv *т, int строка, int колонка) {
    if (!т || строка < 0 || строка >= т->количество_строк) return NULL;
    if (колонка < 0 || колонка >= т->количество_колонок[строка]) return NULL;
    return т->строки[строка][колонка];
}

int zip_csv_строк(ZipCsv *т) { return т ? т->количество_строк : 0; }

int zip_csv_колонок(ZipCsv *т, int строка) {
    if (!т || строка < 0 || строка >= т->количество_строк) return 0;
    return т->количество_колонок[строка];
}

char *zip_csv_записать(ZipCsv *т, char разделитель) {
    size_t cap = 256, pos = 0;
    char *buf;
    int i, j;
    if (!т) return NULL;
    buf = (char *)malloc(cap);
    if (!buf) return NULL;
    for (i = 0; i < т->количество_строк; i++) {
        for (j = 0; j < т->количество_колонок[i]; j++) {
            const char *поле = т->строки[i][j] ? т->строки[i][j] : "";
            int quote = strchr(поле, разделитель) || strchr(поле, '"') ||
                        strchr(поле, '\n') || strchr(поле, '\r');
            size_t need = strlen(поле) * 2 + 8;
            if (pos + need >= cap) {
                while (pos + need >= cap) cap *= 2;
                {
                    char *n = (char *)realloc(buf, cap);
                    if (!n) {
                        free(buf);
                        return NULL;
                    }
                    buf = n;
                }
            }
            if (j > 0) buf[pos++] = разделитель;
            if (quote) {
                size_t k;
                buf[pos++] = '"';
                for (k = 0; поле[k]; k++) {
                    if (поле[k] == '"') buf[pos++] = '"';
                    buf[pos++] = поле[k];
                }
                buf[pos++] = '"';
            } else {
                size_t n = strlen(поле);
                memcpy(buf + pos, поле, n);
                pos += n;
            }
        }
        if (pos + 2 >= cap) {
            cap *= 2;
            {
                char *n = (char *)realloc(buf, cap);
                if (!n) {
                    free(buf);
                    return NULL;
                }
                buf = n;
            }
        }
        buf[pos++] = '\n';
    }
    buf[pos] = '\0';
    return buf;
}

#ifdef ZIP_CSV_MAIN
int main(void) {
    const char *csv =
        "имя,возраст,город\n"
        "Иван,25,Москва\n"
        "Пётр,30,Санкт-Петербург";
    ZipCsv *т = zip_csv_разобрать(csv, ',');
    printf("Строк:\n");
    printf("%d\n", zip_csv_строк(т));
    printf("Заголовки:\n");
    printf("%s\n", zip_csv_получить(т, 0, 0));
    printf("%s\n", zip_csv_получить(т, 0, 1));
    printf("%s\n", zip_csv_получить(т, 0, 2));
    printf("Первая запись:\n");
    printf("%s\n", zip_csv_получить(т, 1, 0));
    printf("%s\n", zip_csv_получить(т, 1, 1));
    printf("%s\n", zip_csv_получить(т, 1, 2));
    zip_csv_освободить(т);
    return 0;
}
#endif

#ifdef ZIP_CSV_UNITTEST
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
        ZipCsv *т = zip_csv_разобрать("имя,возраст\nИван,25\nПётр,30", ',');
        CHECK(zip_csv_строк(т) == 3);
        CHECK(strcmp(zip_csv_получить(т, 0, 0), "имя") == 0);
        CHECK(strcmp(zip_csv_получить(т, 1, 0), "Иван") == 0);
        CHECK(strcmp(zip_csv_получить(т, 2, 1), "30") == 0);
        zip_csv_освободить(т);
    }
    {
        ZipCsv *т = zip_csv_разобрать("\"имя, с запятой\",возраст", ',');
        CHECK(zip_csv_строк(т) == 1);
        CHECK(strcmp(zip_csv_получить(т, 0, 0), "имя, с запятой") == 0);
        CHECK(strcmp(zip_csv_получить(т, 0, 1), "возраст") == 0);
        zip_csv_освободить(т);
    }
    {
        ZipCsv *т = zip_csv_разобрать("a\tb\nc\td", '\t');
        CHECK(zip_csv_строк(т) == 2);
        CHECK(strcmp(zip_csv_получить(т, 1, 1), "d") == 0);
        zip_csv_освободить(т);
    }
    {
        ZipCsv *т = zip_csv_разобрать("a,\"b\"\"c\",d", ',');
        CHECK(strcmp(zip_csv_получить(т, 0, 1), "b\"c") == 0);
        {
            char *out = zip_csv_записать(т, ',');
            CHECK(out && strstr(out, "b\"\"c") != NULL);
            free(out);
        }
        zip_csv_освободить(т);
    }
    {
        ZipCsv *т = zip_csv_разобрать("x,y", ',');
        CHECK(zip_csv_колонок(т, 0) == 2);
        CHECK(zip_csv_получить(т, 9, 0) == NULL);
        zip_csv_освободить(т);
    }
    if (fails) {
        fprintf(stderr, "%d failed\n", fails);
        return 1;
    }
    printf("csv unit tests ok\n");
    return 0;
}
#endif
