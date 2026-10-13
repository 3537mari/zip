/* Host/simulator smoke: compile+run через C-API. */
#include <stdio.h>
#include <string.h>
#include "zip.h"

int main(void) {
    const char *ver = zip_version();
    printf("zip_version=%s\n", ver ? ver : "(null)");

    const char *src = "функция главная() { возврат(2 + 3) }";
    char *out = zip_compile_and_run(src);
    if (!out) {
        fprintf(stderr, "zip_compile_and_run failed\n");
        return 1;
    }
    printf("output=%s\n", out);
    int ok = strstr(out, "5") != NULL;
    zip_free(out);
    if (!ok) {
        fprintf(stderr, "expected 5 in output\n");
        return 2;
    }
    printf("smoke_ok\n");
    return 0;
}
