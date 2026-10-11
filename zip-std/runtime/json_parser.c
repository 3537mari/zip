/* Runtime JSON-парсера ZIP STD (v1.2.0).
 *
 * API: zip_json_разобрать / zip_json_записать / zip_json_получить /
 *      zip_json_установить / zip_json_освободить
 */
#define _CRT_SECURE_NO_WARNINGS
#include <ctype.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define JSON_ОБЪЕКТ 1
#define JSON_МАССИВ 2
#define JSON_СТРОКА 3
#define JSON_ЧИСЛО 4
#define JSON_ФЛАГ 5
#define JSON_NULL 6

typedef struct ZipJsonЗначение {
    int тип;
    char *строка;
    double число;
    int флаг;
    struct ZipJsonЗначение **дети;
    char **ключи;
    int количество;
} ZipJsonЗначение;

typedef struct {
    const char *данные;
    size_t длина;
    size_t позиция;
} ZipJsonЛексер;

static char *zip_strdup(const char *s) {
    size_t n;
    char *p;
    if (!s) return NULL;
    n = strlen(s) + 1;
    p = (char *)malloc(n);
    if (p) memcpy(p, s, n);
    return p;
}

static void zip_json_пропустить_пробелы(ZipJsonЛексер *л) {
    while (л->позиция < л->длина &&
           isspace((unsigned char)л->данные[л->позиция])) {
        л->позиция++;
    }
}

static ZipJsonЗначение *zip_json_разобрать_значение(ZipJsonЛексер *л);

static ZipJsonЗначение *zip_json_новое(int тип) {
    ZipJsonЗначение *з = (ZipJsonЗначение *)calloc(1, sizeof(ZipJsonЗначение));
    if (з) з->тип = тип;
    return з;
}

void zip_json_освободить(ZipJsonЗначение *з) {
    int i;
    if (!з) return;
    free(з->строка);
    if (з->дети) {
        for (i = 0; i < з->количество; i++) {
            zip_json_освободить(з->дети[i]);
        }
        free(з->дети);
    }
    if (з->ключи) {
        for (i = 0; i < з->количество; i++) {
            free(з->ключи[i]);
        }
        free(з->ключи);
    }
    free(з);
}

static char *zip_json_разобрать_строку(ZipJsonЛексер *л) {
    size_t cap = 64, len = 0;
    char *out;
    if (л->позиция >= л->длина || л->данные[л->позиция] != '"') {
        return NULL;
    }
    л->позиция++;
    out = (char *)malloc(cap);
    if (!out) return NULL;
    while (л->позиция < л->длина && л->данные[л->позиция] != '"') {
        unsigned char c = (unsigned char)л->данные[л->позиция++];
        if (c == '\\' && л->позиция < л->длина) {
            char e = л->данные[л->позиция++];
            switch (e) {
            case 'n':
                c = '\n';
                break;
            case 't':
                c = '\t';
                break;
            case 'r':
                c = '\r';
                break;
            case '"':
                c = '"';
                break;
            case '\\':
                c = '\\';
                break;
            case '/':
                c = '/';
                break;
            case 'u':
                /* упрощённо: пропускаем 4 hex, кладём '?' */
                {
                    int k;
                    for (k = 0; k < 4 && л->позиция < л->длина; k++) {
                        л->позиция++;
                    }
                    c = '?';
                }
                break;
            default:
                c = (unsigned char)e;
                break;
            }
        }
        if (len + 1 >= cap) {
            char *nbuf;
            cap *= 2;
            nbuf = (char *)realloc(out, cap);
            if (!nbuf) {
                free(out);
                return NULL;
            }
            out = nbuf;
        }
        out[len++] = (char)c;
    }
    if (л->позиция < л->длина && л->данные[л->позиция] == '"') {
        л->позиция++;
    }
    out[len] = '\0';
    return out;
}

static double zip_json_разобрать_число(ZipJsonЛексер *л) {
    char буфер[96];
    size_t i = 0;
    while (л->позиция < л->длина && i + 1 < sizeof(буфер)) {
        char с = л->данные[л->позиция];
        if (isdigit((unsigned char)с) || с == '-' || с == '+' || с == '.' ||
            с == 'e' || с == 'E') {
            буфер[i++] = с;
            л->позиция++;
        } else {
            break;
        }
    }
    буфер[i] = '\0';
    return atof(буфер);
}

static int zip_json_добавить_элемент(ZipJsonЗначение *массив,
                                     ZipJsonЗначение *элемент) {
    ZipJsonЗначение **новые;
    новые = (ZipJsonЗначение **)realloc(
        массив->дети, (size_t)(массив->количество + 1) * sizeof(ZipJsonЗначение *));
    if (!новые) return 0;
    массив->дети = новые;
    массив->дети[массив->количество++] = элемент;
    return 1;
}

static int zip_json_добавить_поле(ZipJsonЗначение *объект, char *ключ,
                                  ZipJsonЗначение *значение) {
    char **нк;
    ZipJsonЗначение **нд;
    нк = (char **)realloc(объект->ключи,
                          (size_t)(объект->количество + 1) * sizeof(char *));
    нд = (ZipJsonЗначение **)realloc(
        объект->дети, (size_t)(объект->количество + 1) * sizeof(ZipJsonЗначение *));
    if (!нк || !нд) {
        free(нк);
        return 0;
    }
    объект->ключи = нк;
    объект->дети = нд;
    объект->ключи[объект->количество] = ключ;
    объект->дети[объект->количество] = значение;
    объект->количество++;
    return 1;
}

static ZipJsonЗначение *zip_json_разобрать_массив(ZipJsonЛексер *л) {
    ZipJsonЗначение *массив = zip_json_новое(JSON_МАССИВ);
    л->позиция++; /* [ */
    zip_json_пропустить_пробелы(л);
    while (л->позиция < л->длина && л->данные[л->позиция] != ']') {
        ZipJsonЗначение *элемент = zip_json_разобрать_значение(л);
        if (!элемент) break;
        if (!zip_json_добавить_элемент(массив, элемент)) {
            zip_json_освободить(элемент);
            break;
        }
        zip_json_пропустить_пробелы(л);
        if (л->позиция < л->длина && л->данные[л->позиция] == ',') {
            л->позиция++;
            zip_json_пропустить_пробелы(л);
        }
    }
    if (л->позиция < л->длина && л->данные[л->позиция] == ']') {
        л->позиция++;
    }
    return массив;
}

static ZipJsonЗначение *zip_json_разобрать_объект(ZipJsonЛексер *л) {
    ZipJsonЗначение *объект = zip_json_новое(JSON_ОБЪЕКТ);
    л->позиция++; /* { */
    zip_json_пропустить_пробелы(л);
    while (л->позиция < л->длина && л->данные[л->позиция] != '}') {
        char *ключ;
        ZipJsonЗначение *значение;
        zip_json_пропустить_пробелы(л);
        ключ = zip_json_разобрать_строку(л);
        if (!ключ) break;
        zip_json_пропустить_пробелы(л);
        if (л->позиция < л->длина && л->данные[л->позиция] == ':') {
            л->позиция++;
        }
        значение = zip_json_разобрать_значение(л);
        if (!значение) {
            free(ключ);
            break;
        }
        if (!zip_json_добавить_поле(объект, ключ, значение)) {
            free(ключ);
            zip_json_освободить(значение);
            break;
        }
        zip_json_пропустить_пробелы(л);
        if (л->позиция < л->длина && л->данные[л->позиция] == ',') {
            л->позиция++;
            zip_json_пропустить_пробелы(л);
        }
    }
    if (л->позиция < л->длина && л->данные[л->позиция] == '}') {
        л->позиция++;
    }
    return объект;
}

static int zip_json_начинается(ZipJsonЛексер *л, const char *слово) {
    size_t n = strlen(слово);
    if (л->позиция + n > л->длина) return 0;
    if (strncmp(л->данные + л->позиция, слово, n) != 0) return 0;
    if (л->позиция + n < л->длина) {
        unsigned char c = (unsigned char)л->данные[л->позиция + n];
        if (isalnum(c) || c == '_') return 0;
    }
    return 1;
}

static ZipJsonЗначение *zip_json_разобрать_значение(ZipJsonЛексер *л) {
    char с;
    zip_json_пропустить_пробелы(л);
    if (л->позиция >= л->длина) return NULL;
    с = л->данные[л->позиция];
    if (с == '{') return zip_json_разобрать_объект(л);
    if (с == '[') return zip_json_разобрать_массив(л);
    if (с == '"') {
        ZipJsonЗначение *з = zip_json_новое(JSON_СТРОКА);
        з->строка = zip_json_разобрать_строку(л);
        return з;
    }
    if (zip_json_начинается(л, "true")) {
        ZipJsonЗначение *з = zip_json_новое(JSON_ФЛАГ);
        л->позиция += 4;
        з->флаг = 1;
        return з;
    }
    if (zip_json_начинается(л, "false")) {
        ZipJsonЗначение *з = zip_json_новое(JSON_ФЛАГ);
        л->позиция += 5;
        з->флаг = 0;
        return з;
    }
    if (zip_json_начинается(л, "null")) {
        л->позиция += 4;
        return zip_json_новое(JSON_NULL);
    }
    {
        ZipJsonЗначение *з = zip_json_новое(JSON_ЧИСЛО);
        з->число = zip_json_разобрать_число(л);
        return з;
    }
}

ZipJsonЗначение *zip_json_разобрать(const char *строка, size_t длина) {
    ZipJsonЛексер л;
    if (!строка) return NULL;
    л.данные = строка;
    л.длина = длина;
    л.позиция = 0;
    return zip_json_разобрать_значение(&л);
}

static int zip_json_нужен_буфер(size_t *позиция, size_t *ёмкость, char **буфер,
                                size_t нужно) {
    char *n;
    if (*позиция + нужно + 1 <= *ёмкость) return 1;
    while (*позиция + нужно + 1 > *ёмкость) {
        *ёмкость *= 2;
    }
    n = (char *)realloc(*буфер, *ёмкость);
    if (!n) return 0;
    *буфер = n;
    return 1;
}

static void zip_json_записать_в(ZipJsonЗначение *з, char **буфер, size_t *позиция,
                                size_t *ёмкость) {
    int i;
    if (!з || !*буфер) return;
    switch (з->тип) {
    case JSON_ОБЪЕКТ:
        if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, 2)) return;
        (*буфер)[(*позиция)++] = '{';
        for (i = 0; i < з->количество; i++) {
            size_t kl;
            if (i > 0) {
                if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, 1)) return;
                (*буфер)[(*позиция)++] = ',';
            }
            kl = з->ключи[i] ? strlen(з->ключи[i]) : 0;
            if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, kl + 4)) return;
            (*буфер)[(*позиция)++] = '"';
            if (з->ключи[i]) {
                memcpy(*буфер + *позиция, з->ключи[i], kl);
                *позиция += kl;
            }
            (*буфер)[(*позиция)++] = '"';
            (*буфер)[(*позиция)++] = ':';
            zip_json_записать_в(з->дети[i], буфер, позиция, ёмкость);
        }
        if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, 1)) return;
        (*буфер)[(*позиция)++] = '}';
        break;
    case JSON_МАССИВ:
        if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, 2)) return;
        (*буфер)[(*позиция)++] = '[';
        for (i = 0; i < з->количество; i++) {
            if (i > 0) {
                if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, 1)) return;
                (*буфер)[(*позиция)++] = ',';
            }
            zip_json_записать_в(з->дети[i], буфер, позиция, ёмкость);
        }
        if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, 1)) return;
        (*буфер)[(*позиция)++] = ']';
        break;
    case JSON_СТРОКА: {
        const char *s = з->строка ? з->строка : "";
        size_t j;
        if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, strlen(s) * 2 + 2))
            return;
        (*буфер)[(*позиция)++] = '"';
        for (j = 0; s[j]; j++) {
            char c = s[j];
            if (c == '"' || c == '\\') {
                if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, 2)) return;
                (*буфер)[(*позиция)++] = '\\';
                (*буфер)[(*позиция)++] = c;
            } else if (c == '\n') {
                if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, 2)) return;
                (*буфер)[(*позиция)++] = '\\';
                (*буфер)[(*позиция)++] = 'n';
            } else {
                if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, 1)) return;
                (*буфер)[(*позиция)++] = c;
            }
        }
        (*буфер)[(*позиция)++] = '"';
        break;
    }
    case JSON_ЧИСЛО: {
        char tmp[64];
        int n = snprintf(tmp, sizeof(tmp), "%.15g", з->число);
        if (n < 0) n = 0;
        if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, (size_t)n)) return;
        memcpy(*буфер + *позиция, tmp, (size_t)n);
        *позиция += (size_t)n;
        break;
    }
    case JSON_ФЛАГ:
        if (з->флаг) {
            if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, 4)) return;
            memcpy(*буфер + *позиция, "true", 4);
            *позиция += 4;
        } else {
            if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, 5)) return;
            memcpy(*буфер + *позиция, "false", 5);
            *позиция += 5;
        }
        break;
    case JSON_NULL:
        if (!zip_json_нужен_буфер(позиция, ёмкость, буфер, 4)) return;
        memcpy(*буфер + *позиция, "null", 4);
        *позиция += 4;
        break;
    }
    if (*позиция < *ёмкость) (*буфер)[*позиция] = '\0';
}

char *zip_json_записать(ZipJsonЗначение *з) {
    size_t ёмкость = 256;
    size_t позиция = 0;
    char *буфер = (char *)malloc(ёмкость);
    if (!буфер) return NULL;
    буфер[0] = '\0';
    zip_json_записать_в(з, &буфер, &позиция, &ёмкость);
    if (позиция < ёмкость) буфер[позиция] = '\0';
    return буфер;
}

static ZipJsonЗначение *zip_json_найти_ключ(ZipJsonЗначение *объект,
                                            const char *ключ) {
    int i;
    if (!объект || объект->тип != JSON_ОБЪЕКТ || !ключ) return NULL;
    for (i = 0; i < объект->количество; i++) {
        if (объект->ключи[i] && strcmp(объект->ключи[i], ключ) == 0) {
            return объект->дети[i];
        }
    }
    return NULL;
}

/* Путь: "user.name", "items[1]", "адрес.улица" */
ZipJsonЗначение *zip_json_получить(ZipJsonЗначение *з, const char *путь) {
    char *копия;
    char *часть;
    char *save = NULL;
    ZipJsonЗначение *текущее = з;
    if (!з || !путь) return NULL;
    копия = zip_strdup(путь);
    if (!копия) return NULL;
#ifdef _WIN32
    часть = strtok_s(копия, ".", &save);
#else
    часть = strtok_r(копия, ".", &save);
#endif
    while (часть && текущее) {
        char *скобка = strchr(часть, '[');
        char ключ[256];
        int есть_индекс = 0;
        int индекс = 0;
        if (скобка) {
            size_t kn = (size_t)(скобка - часть);
            if (kn >= sizeof(ключ)) kn = sizeof(ключ) - 1;
            memcpy(ключ, часть, kn);
            ключ[kn] = '\0';
            индекс = atoi(скобка + 1);
            есть_индекс = 1;
        } else {
            strncpy(ключ, часть, sizeof(ключ) - 1);
            ключ[sizeof(ключ) - 1] = '\0';
        }

        if (ключ[0] != '\0') {
            if (текущее->тип == JSON_ОБЪЕКТ) {
                текущее = zip_json_найти_ключ(текущее, ключ);
            } else {
                текущее = NULL;
            }
        }
        if (есть_индекс && текущее) {
            if (текущее->тип == JSON_МАССИВ && индекс >= 0 &&
                индекс < текущее->количество) {
                текущее = текущее->дети[индекс];
            } else {
                текущее = NULL;
            }
        }
#ifdef _WIN32
        часть = strtok_s(NULL, ".", &save);
#else
        часть = strtok_r(NULL, ".", &save);
#endif
    }
    free(копия);
    return текущее;
}

int zip_json_установить(ZipJsonЗначение *корень, const char *путь,
                        ZipJsonЗначение *новое) {
    char *копия;
    char *часть;
    char *save = NULL;
    ZipJsonЗначение *родитель = корень;
    ZipJsonЗначение *текущее = корень;
    char *части[32];
    int nчасти = 0;
    int i;
    if (!корень || !путь || !новое) return 0;
    копия = zip_strdup(путь);
    if (!копия) return 0;
#ifdef _WIN32
    часть = strtok_s(копия, ".", &save);
#else
    часть = strtok_r(копия, ".", &save);
#endif
    while (часть && nчасти < 32) {
        части[nчасти++] = часть;
#ifdef _WIN32
        часть = strtok_s(NULL, ".", &save);
#else
        часть = strtok_r(NULL, ".", &save);
#endif
    }
    if (nчасти == 0) {
        free(копия);
        return 0;
    }
    for (i = 0; i < nчасти - 1; i++) {
        char *скобка = strchr(части[i], '[');
        char ключ[256];
        int есть_индекс = 0;
        int индекс = 0;
        if (скобка) {
            size_t kn = (size_t)(скобка - части[i]);
            if (kn >= sizeof(ключ)) kn = sizeof(ключ) - 1;
            memcpy(ключ, части[i], kn);
            ключ[kn] = '\0';
            индекс = atoi(скобка + 1);
            есть_индекс = 1;
        } else {
            strncpy(ключ, части[i], sizeof(ключ) - 1);
            ключ[sizeof(ключ) - 1] = '\0';
        }
        родитель = текущее;
        if (ключ[0]) {
            текущее = zip_json_найти_ключ(текущее, ключ);
            if (!текущее) {
                free(копия);
                return 0;
            }
        }
        if (есть_индекс) {
            if (текущее->тип != JSON_МАССИВ || индекс < 0 ||
                индекс >= текущее->количество) {
                free(копия);
                return 0;
            }
            текущее = текущее->дети[индекс];
        }
        (void)родитель;
    }
    {
        char *последний = части[nчасти - 1];
        char *скобка = strchr(последний, '[');
        if (скобка) {
            char ключ[256];
            int индекс;
            size_t kn = (size_t)(скобка - последний);
            ZipJsonЗначение *массив;
            if (kn >= sizeof(ключ)) kn = sizeof(ключ) - 1;
            memcpy(ключ, последний, kn);
            ключ[kn] = '\0';
            индекс = atoi(скобка + 1);
            массив = ключ[0] ? zip_json_найти_ключ(текущее, ключ) : текущее;
            if (!массив || массив->тип != JSON_МАССИВ || индекс < 0 ||
                индекс >= массив->количество) {
                free(копия);
                return 0;
            }
            zip_json_освободить(массив->дети[индекс]);
            массив->дети[индекс] = новое;
        } else if (текущее->тип == JSON_ОБЪЕКТ) {
            int j;
            for (j = 0; j < текущее->количество; j++) {
                if (текущее->ключи[j] && strcmp(текущее->ключи[j], последний) == 0) {
                    zip_json_освободить(текущее->дети[j]);
                    текущее->дети[j] = новое;
                    free(копия);
                    return 1;
                }
            }
            if (!zip_json_добавить_поле(текущее, zip_strdup(последний), новое)) {
                free(копия);
                return 0;
            }
        } else {
            free(копия);
            return 0;
        }
    }
    free(копия);
    return 1;
}

int zip_json_тип(ZipJsonЗначение *з) { return з ? з->тип : 0; }
int zip_json_количество(ZipJsonЗначение *з) { return з ? з->количество : 0; }
const char *zip_json_как_строка(ZipJsonЗначение *з) {
    return (з && з->тип == JSON_СТРОКА) ? з->строка : NULL;
}
double zip_json_как_число(ZipJsonЗначение *з) {
    return (з && з->тип == JSON_ЧИСЛО) ? з->число : 0.0;
}
int zip_json_как_флаг(ZipJsonЗначение *з) {
    return (з && з->тип == JSON_ФЛАГ) ? (з->флаг != 0) : 0;
}

ZipJsonЗначение *zip_json_сделать_число(double x) {
    ZipJsonЗначение *з = zip_json_новое(JSON_ЧИСЛО);
    if (з) з->число = x;
    return з;
}

ZipJsonЗначение *zip_json_сделать_строку(const char *s) {
    ZipJsonЗначение *з = zip_json_новое(JSON_СТРОКА);
    if (з) з->строка = zip_strdup(s ? s : "");
    return з;
}


#ifdef ZIP_JSON_BENCH
int main(void) {
    const int N = 10000;
    size_t cap = 64 + (size_t)N * 32;
    char *buf = (char *)malloc(cap);
    if (!buf) return 1;
    size_t pos = 0;
    pos += (size_t)snprintf(buf + pos, cap - pos, "{\"данные\":[");
    for (int i = 0; i < N; i++) {
        if (i) pos += (size_t)snprintf(buf + pos, cap - pos, ",");
        pos += (size_t)snprintf(buf + pos, cap - pos, "{\"id\":%d}", i);
        if (pos + 64 >= cap) {
            cap *= 2;
            char *nb = (char *)realloc(buf, cap);
            if (!nb) { free(buf); return 1; }
            buf = nb;
        }
    }
    pos += (size_t)snprintf(buf + pos, cap - pos, "]}");

    clock_t t0 = clock();
    ZipJsonЗначение *v = zip_json_разобрать(buf, pos);
    clock_t t1 = clock();
    long ms = (long)((t1 - t0) * 1000.0 / CLOCKS_PER_SEC);
    if (v) zip_json_освободить(v);
    free(buf);
    printf("%ld\n", ms);
    return 0;
}
#endif

#ifdef ZIP_JSON_MAIN
int main(void) {
    const char *строка =
        "{"
        "\"имя\": \"Иван\","
        "\"возраст\": 25,"
        "\"город\": \"Москва\","
        "\"навыки\": [\"ZIP\", \"Rust\", \"Python\"],"
        "\"адрес\": {\"улица\": \"Ленина\", \"дом\": 10}"
        "}";
    ZipJsonЗначение *данные = zip_json_разобрать(строка, strlen(строка));
    ZipJsonЗначение *имя = zip_json_получить(данные, "имя");
    ZipJsonЗначение *возраст = zip_json_получить(данные, "возраст");
    ZipJsonЗначение *улица = zip_json_получить(данные, "адрес.улица");
    printf("Имя:\n");
    printf("%s\n", zip_json_как_строка(имя) ? zip_json_как_строка(имя) : "?");
    printf("Возраст:\n");
    printf("%.0f\n", zip_json_как_число(возраст));
    printf("Улица:\n");
    printf("%s\n", zip_json_как_строка(улица) ? zip_json_как_строка(улица) : "?");
    zip_json_освободить(данные);
    return 0;
}
#endif

#ifdef ZIP_JSON_UNITTEST
static int fails = 0;
#define CHECK(c)                                                               \
    do {                                                                       \
        if (!(c)) {                                                            \
            fprintf(stderr, "FAIL %s:%d: %s\n", __FILE__, __LINE__, #c);       \
            fails++;                                                           \
        }                                                                      \
    } while (0)

int main(void) {
    {
        const char *s = "{\"имя\": \"Иван\", \"возраст\": 25}";
        ZipJsonЗначение *v = zip_json_разобрать(s, strlen(s));
        ZipJsonЗначение *имя = zip_json_получить(v, "имя");
        ZipJsonЗначение *возраст = zip_json_получить(v, "возраст");
        CHECK(v && zip_json_тип(v) == JSON_ОБЪЕКТ);
        CHECK(имя && strcmp(zip_json_как_строка(имя), "Иван") == 0);
        CHECK(возраст && zip_json_как_число(возраст) == 25.0);
        zip_json_освободить(v);
    }
    {
        const char *s = "[1, 2, 3, 4, 5]";
        ZipJsonЗначение *v = zip_json_разобрать(s, strlen(s));
        CHECK(v && zip_json_тип(v) == JSON_МАССИВ);
        CHECK(zip_json_количество(v) == 5);
        zip_json_освободить(v);
    }
    {
        const char *s = "{\"user\": {\"name\": \"Пётр\", \"age\": 30}}";
        ZipJsonЗначение *v = zip_json_разобрать(s, strlen(s));
        ZipJsonЗначение *name = zip_json_получить(v, "user.name");
        CHECK(name && strcmp(zip_json_как_строка(name), "Пётр") == 0);
        zip_json_освободить(v);
    }
    {
        const char *s = "{\"items\": [10, 20, 30]}";
        ZipJsonЗначение *v = zip_json_разобрать(s, strlen(s));
        ZipJsonЗначение *второе = zip_json_получить(v, "items[1]");
        CHECK(второе && zip_json_как_число(второе) == 20.0);
        zip_json_освободить(v);
    }
    {
        const char *s = "{\"a\": 1}";
        ZipJsonЗначение *v = zip_json_разобрать(s, strlen(s));
        char *out;
        zip_json_установить(v, "a", zip_json_сделать_число(42));
        out = zip_json_записать(v);
        CHECK(out && strstr(out, "42") != NULL);
        free(out);
        zip_json_освободить(v);
    }
    {
        const char *s = "{\"ok\": true, \"x\": null, \"f\": false}";
        ZipJsonЗначение *v = zip_json_разобрать(s, strlen(s));
        CHECK(zip_json_как_флаг(zip_json_получить(v, "ok")) == 1);
        CHECK(zip_json_тип(zip_json_получить(v, "x")) == JSON_NULL);
        CHECK(zip_json_как_флаг(zip_json_получить(v, "f")) == 0);
        zip_json_освободить(v);
    }
    {
        const char *s = "{\"msg\": \"a\\\"b\"}";
        ZipJsonЗначение *v = zip_json_разобрать(s, strlen(s));
        CHECK(strcmp(zip_json_как_строка(zip_json_получить(v, "msg")), "a\"b") ==
              0);
        zip_json_освободить(v);
    }
    if (fails) {
        fprintf(stderr, "%d checks failed\n", fails);
        return 1;
    }
    printf("json unit tests ok\n");
    return 0;
}
#endif
