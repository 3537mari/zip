/* Runtime для размеченных объединений (Идеи 14–15). */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef enum {
    ZIP_TAG_НИЧТО = 0,
    ZIP_TAG_НЕЧТО = 1,
    ZIP_TAG_УСПЕХ = 2,
    ZIP_TAG_ОШИБКА = 3
} ZipТег;

typedef struct {
    ZipТег тег;
    int64_t значение;
    char* текст;
} ZipОбъединение;

static char* zip_dup(const char* s) {
    if (!s) s = "";
    size_t n = strlen(s) + 1;
    char* p = (char*)malloc(n);
    if (p) memcpy(p, s, n);
    return p;
}

ZipОбъединение* zip_создать_нечто(int64_t значение) {
    ZipОбъединение* об = (ZipОбъединение*)malloc(sizeof(ZipОбъединение));
    if (!об) return NULL;
    об->тег = ZIP_TAG_НЕЧТО;
    об->значение = значение;
    об->текст = NULL;
    return об;
}

ZipОбъединение* zip_создать_ничто(void) {
    ZipОбъединение* об = (ZipОбъединение*)malloc(sizeof(ZipОбъединение));
    if (!об) return NULL;
    об->тег = ZIP_TAG_НИЧТО;
    об->значение = 0;
    об->текст = NULL;
    return об;
}

ZipОбъединение* zip_создать_успех(int64_t значение) {
    ZipОбъединение* об = (ZipОбъединение*)malloc(sizeof(ZipОбъединение));
    if (!об) return NULL;
    об->тег = ZIP_TAG_УСПЕХ;
    об->значение = значение;
    об->текст = NULL;
    return об;
}

ZipОбъединение* zip_создать_ошибку(const char* текст) {
    ZipОбъединение* об = (ZipОбъединение*)malloc(sizeof(ZipОбъединение));
    if (!об) return NULL;
    об->тег = ZIP_TAG_ОШИБКА;
    об->значение = 0;
    об->текст = zip_dup(текст);
    return об;
}

int64_t zip_проверить_нечто(ZipОбъединение* об) {
    return об && об->тег == ZIP_TAG_НЕЧТО ? 1 : 0;
}

int64_t zip_проверить_ничто(ZipОбъединение* об) {
    return об && об->тег == ZIP_TAG_НИЧТО ? 1 : 0;
}

int64_t zip_проверить_успех(ZipОбъединение* об) {
    return об && об->тег == ZIP_TAG_УСПЕХ ? 1 : 0;
}

int64_t zip_проверить_ошибку(ZipОбъединение* об) {
    return об && об->тег == ZIP_TAG_ОШИБКА ? 1 : 0;
}

int64_t zip_извлечь_значение(ZipОбъединение* об) {
    return об ? об->значение : 0;
}

char* zip_извлечь_ошибку(ZipОбъединение* об) {
    static char пусто[] = "";
    return об && об->тег == ZIP_TAG_ОШИБКА && об->текст ? об->текст : пусто;
}

void zip_освободить_объединение(ZipОбъединение* об) {
    if (!об) return;
    free(об->текст);
    free(об);
}

/* R3: ошибка несоответствия типов при специализации. */
void zip_ошибка_типа(const char* имя_глагола) {
    fprintf(stderr, "Ошибка: нет подходящего варианта глагола '%s'\n",
            имя_глагола ? имя_глагола : "?");
    exit(1);
}

/* R3: конкатенация строк для специализации САПАР(Строка, Строка). */
char* соединить_строки(const char* а, const char* б) {
    if (!а) а = "";
    if (!б) б = "";
    size_t na = strlen(а);
    size_t nb = strlen(б);
    char* out = (char*)malloc(na + nb + 1);
    if (!out) return NULL;
    memcpy(out, а, na);
    memcpy(out + na, б, nb + 1);
    return out;
}

char* zip_соединить_строки(const char* а, const char* б) {
    return соединить_строки(а, б);
}

#ifdef ZIP_UNION_MAIN
int main(void) {
    ZipОбъединение* a = zip_создать_нечто(42);
    ZipОбъединение* b = zip_создать_ничто();
    ZipОбъединение* c = zip_создать_успех(5);
    ZipОбъединение* d = zip_создать_ошибку("деление на ноль");
    printf("%lld\n", (long long)(zip_проверить_нечто(a) ? zip_извлечь_значение(a) : 0));
    printf("%s\n", zip_проверить_ничто(b) ? "ничто" : "?");
    printf("%lld\n", (long long)(zip_проверить_успех(c) ? zip_извлечь_значение(c) : 0));
    printf("%s\n", zip_проверить_ошибку(d) ? zip_извлечь_ошибку(d) : "?");
    zip_освободить_объединение(a);
    zip_освободить_объединение(b);
    zip_освободить_объединение(c);
    zip_освободить_объединение(d);
    return 0;
}
#endif
