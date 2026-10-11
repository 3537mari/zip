#include "zipb.h"

static char *read_file(const char *path, size_t *out_len) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        fprintf(stderr, "ошибка: не могу открыть %s\n", path);
        return NULL;
    }
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (sz < 0) { fclose(f); return NULL; }
    char *buf = (char *)zip_alloc((size_t)sz + 1);
    size_t rd = fread(buf, 1, (size_t)sz, f);
    buf[rd] = 0;
    fclose(f);
    *out_len = rd;
    return buf;
}

static int write_file(const char *path, Buf *b) {
    FILE *f = fopen(path, "wb");
    if (!f) {
        fprintf(stderr, "ошибка: не могу создать %s\n", path);
        return -1;
    }
    fwrite(b->data, 1, b->len, f);
    fclose(f);
    return 0;
}

static void usage(const char *argv0) {
    fprintf(stderr, "ZIP bootstrap compiler v%s\n", ZIPC_BOOTSTRAP_VERSION);
    fprintf(stderr, "Использование: %s <вход.zip> <выход.zipb>\n", argv0);
    fprintf(stderr, "         %s --version | --help\n", argv0);
    fprintf(stderr, "\nКомпилирует .zip → ZIPB v3 (.zipb). Только C-зависимости.\n");
}

int main(int argc, char **argv) {
    if (argc == 2 && (!strcmp(argv[1], "--help") || !strcmp(argv[1], "-h"))) {
        usage(argv[0]);
        return 0;
    }
    if (argc == 2 && (!strcmp(argv[1], "--version") || !strcmp(argv[1], "версия"))) {
        printf("ZIP bootstrap compiler v%s\n", ZIPC_BOOTSTRAP_VERSION);
        return 0;
    }
    if (argc == 3 && !strcmp(argv[1], "--dump")) {
        return dump_file(argv[2]);
    }
    if (argc < 3) {
        usage(argv[0]);
        return 1;
    }

    size_t len = 0;
    char *src = read_file(argv[1], &len);
    if (!src) return 1;

    printf("Компиляция: %s (%zu байт)\n", argv[1], len);

    size_t ntok = 0;
    Token *toks = lex(src, &ntok);
    if (!toks) {
        free(src);
        return 1;
    }
    printf("Токенов: %zu\n", ntok);

    Node *ast = parse(toks, ntok);
    if (!ast) {
        free(src);
        tokens_free(toks, ntok);
        return 1;
    }
    int verr = validate_ast(ast);
    if (verr) fprintf(stderr, "предупреждений AST: %d\n", verr);
    printf("AST создан\n");

    Buf *bc = codegen(ast);
    printf("Байт-код: %zu байт\n", bc->len);

    if (write_file(argv[2], bc) != 0) {
        free(src);
        tokens_free(toks, ntok);
        node_free(ast);
        buf_free(bc);
        return 1;
    }
    printf("OK: записано %s\n", argv[2]);

    free(src);
    tokens_free(toks, ntok);
    node_free(ast);
    buf_free(bc);
    return 0;
}
