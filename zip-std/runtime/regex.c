/* Runtime RegEx ZIP STD (упрощённый backtracking). */
#define _CRT_SECURE_NO_WARNINGS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int zip_regex_класс(const char *открыть, const char *закрыть, char с) {
    const char *к = открыть + 1;
    int negate = 0;
    int hit = 0;
    if (к < закрыть && *к == '^') {
        negate = 1;
        к++;
    }
    while (к < закрыть) {
        if (к + 2 < закрыть && *(к + 1) == '-') {
            if (с >= *к && с <= *(к + 2)) hit = 1;
            к += 3;
        } else {
            if (*к == с) hit = 1;
            к++;
        }
    }
    return negate ? !hit : hit;
}

/* Совпадает ли один атом в начале строки; возвращает длину атома в шаблоне. */
static int zip_regex_атом(const char *шаблон, const char *строка, int *съел,
                          size_t *длина_атома) {
    *съел = 0;
    *длина_атома = 0;
    if (*шаблон == '\0') return 0;
    if (*шаблон == '.') {
        *длина_атома = 1;
        if (*строка == '\0') return 0;
        *съел = 1;
        return 1;
    }
    if (*шаблон == '\\') {
        if (*(шаблон + 1) == '\0') return 0;
        *длина_атома = 2;
        if (*строка != *(шаблон + 1)) return 0;
        *съел = 1;
        return 1;
    }
    if (*шаблон == '[') {
        const char *cl = strchr(шаблон, ']');
        if (!cl) return 0;
        *длина_атома = (size_t)(cl - шаблон + 1);
        if (*строка == '\0') return 0;
        if (!zip_regex_класс(шаблон, cl, *строка)) return 0;
        *съел = 1;
        return 1;
    }
    *длина_атома = 1;
    if (*строка == '\0' || *строка != *шаблон) return 0;
    *съел = 1;
    return 1;
}

static int zip_regex_совпадение(const char *шаблон, const char *строка) {
    size_t alen;
    int съел;
    char q;

    /* Пустой остаток шаблона = успех (префиксное совпадение / поиск). */
    if (*шаблон == '\0') return 1;
    if (*шаблон == '$' && *(шаблон + 1) == '\0') return *строка == '\0';
    if (*шаблон == '^') return zip_regex_совпадение(шаблон + 1, строка);

    /* Узнать длину атома (даже если сейчас не совпал — нужно для * / ?). */
    {
        int dummy_eat = 0;
        if (*шаблон == '\\' && *(шаблон + 1)) {
            alen = 2;
        } else if (*шаблон == '[') {
            const char *cl = strchr(шаблон, ']');
            if (!cl) return 0;
            alen = (size_t)(cl - шаблон + 1);
        } else if (*шаблон == '\0') {
            return *строка == '\0';
        } else {
            alen = 1;
        }
        (void)dummy_eat;
        (void)съел;
    }

    q = шаблон[alen];
    if (q == '*' || q == '+' || q == '?') {
        const char *rest = шаблон + alen + 1;
        const char *s = строка;
        const char *ends[1024];
        int count = 0;
        int min_n = (q == '+') ? 1 : 0;
        int max_n = (q == '?') ? 1 : 1000000;
        int n = 0;
        ends[count++] = s;
        while (n < max_n) {
            int one;
            size_t dummy;
            if (!zip_regex_атом(шаблон, s, &one, &dummy) || !one) break;
            s += one;
            n++;
            if (count < 1024) ends[count++] = s;
        }
        if (n < min_n) return 0;
        {
            int i;
            for (i = count - 1; i >= min_n; i--) {
                if (zip_regex_совпадение(rest, ends[i])) return 1;
            }
        }
        return 0;
    }

    if (!zip_regex_атом(шаблон, строка, &съел, &alen) || !съел) return 0;
    return zip_regex_совпадение(шаблон + alen, строка + съел);
}

int zip_regex_проверить(const char *шаблон, const char *строка) {
    if (!шаблон || !строка) return 0;
    if (*шаблон == '^') {
        return zip_regex_совпадение(шаблон + 1, строка);
    }
    {
        const char *с = строка;
        for (;;) {
            if (zip_regex_совпадение(шаблон, с)) return 1;
            if (*с == '\0') break;
            с++;
        }
    }
    return 0;
}

int *zip_regex_найти(const char *шаблон, const char *строка, int *количество) {
    int *позиции;
    int найдено = 0;
    const char *с;
    int позиция = 0;
    if (!количество) return NULL;
    *количество = 0;
    if (!шаблон || !строка) return NULL;
    позиции = (int *)malloc(100 * sizeof(int));
    if (!позиции) return NULL;
    с = строка;
    while (*с) {
        if (zip_regex_совпадение(шаблон, с)) {
            if (найдено < 100) позиции[найдено++] = позиция;
        }
        с++;
        позиция++;
    }
    *количество = найдено;
    return позиции;
}

#ifdef ZIP_REGEX_MAIN
int main(void) {
    const char *email = "user@example.com";
    const char *шаблон_email = "^[a-zA-Z]+@[a-zA-Z]+\\.[a-z]+$";
    const char *телефон = "+7-999-123-45-67";
    const char *шаблон_телефон = "^\\+7-[0-9]+-[0-9]+-[0-9]+-[0-9]+$";
    int кол = 0;
    int *поз;

    if (zip_regex_проверить(шаблон_email, email)) {
        printf("E-mail корректен\n");
    } else {
        printf("E-mail некорректен\n");
    }
    if (zip_regex_проверить(шаблон_телефон, телефон)) {
        printf("Телефон корректен\n");
    } else {
        printf("Телефон некорректен\n");
    }
    поз = zip_regex_найти("[0-9]+", "abc123def456ghi789", &кол);
    printf("Найдены цифры\n");
    printf("%d\n", кол);
    free(поз);
    return 0;
}
#endif

#ifdef ZIP_REGEX_UNITTEST
static int fails;
#define CHECK(c)                                                               \
    do {                                                                       \
        if (!(c)) {                                                            \
            fprintf(stderr, "FAIL %s:%d %s\n", __FILE__, __LINE__, #c);        \
            fails++;                                                           \
        }                                                                      \
    } while (0)

int main(void) {
    CHECK(zip_regex_проверить("hello", "hello"));
    CHECK(!zip_regex_проверить("hello", "hell"));
    CHECK(zip_regex_проверить("hel*o", "heo"));
    CHECK(zip_regex_проверить("hel*o", "hello"));
    CHECK(zip_regex_проверить("hel*o", "hellllo"));
    CHECK(zip_regex_проверить("a+", "aaa"));
    CHECK(!zip_regex_проверить("a+", ""));
    CHECK(zip_regex_проверить("colou?r", "color"));
    CHECK(zip_regex_проверить("colou?r", "colour"));
    CHECK(zip_regex_проверить("[0-9]+", "123"));
    CHECK(zip_regex_проверить("[a-z]+", "abc"));
    CHECK(zip_regex_проверить("^abc$", "abc"));
    CHECK(!zip_regex_проверить("^abc$", "xabc"));
    CHECK(zip_regex_проверить("a.c", "abc"));
    CHECK(zip_regex_проверить("\\+7", "+7"));
    CHECK(zip_regex_проверить("^[a-zA-Z]+@[a-zA-Z]+\\.[a-z]+$",
                              "user@example.com"));
    CHECK(zip_regex_проверить("^\\+7-[0-9]+-[0-9]+-[0-9]+-[0-9]+$",
                              "+7-999-123-45-67"));
    {
        int n = 0;
        int *p = zip_regex_найти("[0-9]+", "a1b22c", &n);
        CHECK(n >= 2);
        free(p);
    }
    if (fails) {
        fprintf(stderr, "%d failed\n", fails);
        return 1;
    }
    printf("regex unit tests ok\n");
    return 0;
}
#endif
