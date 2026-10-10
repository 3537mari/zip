/* ZIP-SELF: minimal lexer runtime — spaces, integers, single-char tokens.
 * Mirrors zip-self/лексика.zip first milestone. */
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum {
    TOK_EOF = 0,
    TOK_EQ = 5,
    TOK_COLON = 6,
    TOK_LPAREN = 7,
    TOK_RPAREN = 8,
    TOK_LBRACE = 9,
    TOK_RBRACE = 10,
    TOK_COMMA = 11,
    TOK_WORD = 14,
    TOK_INT = 15,
    TOK_STR = 17,
    TOK_UNKNOWN = 99
};

typedef struct {
    const unsigned char *src;
    size_t len;
    size_t pos;
    int line;
    int col;
} Lexer;

static int peek(Lexer *L) {
    if (L->pos >= L->len) return 0;
    return L->src[L->pos];
}

static void advance(Lexer *L) {
    if (L->pos >= L->len) return;
    int c = L->src[L->pos++];
    if (c == '\n') {
        L->line++;
        L->col = 1;
    } else {
        L->col++;
    }
}

static void skip_ws(Lexer *L) {
    for (;;) {
        int c = peek(L);
        if (c == ' ' || c == '\t' || c == '\r' || c == '\n') advance(L);
        else break;
    }
}

static const char *tok_name(int k) {
    switch (k) {
        case TOK_EOF: return "КОНЕЦ_ФАЙЛА";
        case TOK_EQ: return "=";
        case TOK_COLON: return ":";
        case TOK_LPAREN: return "(";
        case TOK_RPAREN: return ")";
        case TOK_LBRACE: return "{";
        case TOK_RBRACE: return "}";
        case TOK_COMMA: return ",";
        case TOK_WORD: return "СЛОВО";
        case TOK_INT: return "ЦЕЛОЕ";
        case TOK_STR: return "СТРОКА";
        default: return "?";
    }
}

static void next_tok(Lexer *L, int *kind, long long *ival) {
    skip_ws(L);
    *ival = 0;
    int c = peek(L);
    if (c == 0) {
        *kind = TOK_EOF;
        return;
    }
    int line = L->line, col = L->col;
    (void)line;
    (void)col;
    if (isdigit(c) || (c == '-' && L->pos + 1 < L->len && isdigit(L->src[L->pos + 1]))) {
        int sign = 1;
        if (c == '-') {
            sign = -1;
            advance(L);
            c = peek(L);
        }
        long long n = 0;
        while (isdigit(peek(L))) {
            n = n * 10 + (peek(L) - '0');
            advance(L);
        }
        *kind = TOK_INT;
        *ival = n * sign;
        return;
    }
    switch (c) {
        case '=': *kind = TOK_EQ; advance(L); return;
        case ':': *kind = TOK_COLON; advance(L); return;
        case '(': *kind = TOK_LPAREN; advance(L); return;
        case ')': *kind = TOK_RPAREN; advance(L); return;
        case '{': *kind = TOK_LBRACE; advance(L); return;
        case '}': *kind = TOK_RBRACE; advance(L); return;
        case ',': *kind = TOK_COMMA; advance(L); return;
        default:
            /* skip one byte as unknown / word stub */
            *kind = TOK_WORD;
            advance(L);
            return;
    }
}

#ifdef ZIP_LEXER_MAIN
int main(int argc, char **argv) {
    const char *path = argc > 1 ? argv[1] : NULL;
    unsigned char *buf = NULL;
    size_t n = 0;
    if (path) {
        FILE *f = fopen(path, "rb");
        if (!f) {
            perror(path);
            return 1;
        }
        fseek(f, 0, SEEK_END);
        long sz = ftell(f);
        fseek(f, 0, SEEK_SET);
        if (sz < 0) {
            fclose(f);
            return 1;
        }
        buf = (unsigned char *)malloc((size_t)sz + 1);
        n = fread(buf, 1, (size_t)sz, f);
        buf[n] = 0;
        fclose(f);
    } else {
        /* demo: spaces + numbers + parens */
        const char *demo = "  42  (7, 100)  \n  0 ";
        n = strlen(demo);
        buf = (unsigned char *)malloc(n + 1);
        memcpy(buf, demo, n + 1);
    }

    Lexer L = {buf, n, 0, 1, 1};
    printf("=== ZIP-lexer ===\n");
    for (;;) {
        int k;
        long long v;
        int line = L.line, col = L.col;
        next_tok(&L, &k, &v);
        if (k == TOK_EOF) {
            printf("  [%d:%d] %s\n", line, col, tok_name(k));
            break;
        }
        if (k == TOK_INT)
            printf("  [%d:%d] %s(%lld)\n", line, col, tok_name(k), v);
        else
            printf("  [%d:%d] %s\n", line, col, tok_name(k));
    }
    free(buf);
    return 0;
}
#endif
