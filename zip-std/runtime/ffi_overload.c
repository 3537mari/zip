/* R6: Пример перегруженных C-функций. */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Открыть файл только для чтения. */
FILE* zip_открыть_1(const char* путь) {
    return fopen(путь, "r");
}

/* Открыть файл в указанном режиме. */
FILE* zip_открыть_2(const char* путь, const char* режим) {
    return fopen(путь, режим);
}

/* Найти максимальное из двух чисел. */
int64_t zip_максимум_2(int64_t а, int64_t б) {
    return а > б ? а : б;
}

/* Найти максимальное из трёх чисел. */
int64_t zip_максимум_3(int64_t а, int64_t б, int64_t в) {
    int64_t м = а > б ? а : б;
    return м > в ? м : в;
}
