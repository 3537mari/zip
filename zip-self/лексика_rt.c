/* ZIP-SELF: full lexer runtime (UTF-8, words, roots, templates, operators).
 * Spec: zip-self/лексика.zip — mirror of zipc/src/лексика.rs */
#define _CRT_SECURE_NO_WARNINGS
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

enum {
    T_EOF = 0,
    T_IMPULSE = 1,      /* >> */
    T_CHOIR = 2,        /* || */
    T_FIX = 3,          /* ! */
    T_FLOW = 4,         /* ~ */
    T_EQ = 5,           /* = */
    T_COLON = 6,
    T_LPAREN = 7,
    T_RPAREN = 8,
    T_LBRACE = 9,
    T_RBRACE = 10,
    T_COMMA = 11,
    T_ROOT = 12,
    T_TEMPLATE = 13,
    T_WORD = 14,
    T_INT = 15,
    T_FLOAT = 16,
    T_STRING = 17,
    T_DIRECTIVE = 18,
    T_NET_IMPULSE = 19, /* >>> */
    T_DASH = 20,
    T_PLUS = 21,
    T_STAR = 22,
    T_SLASH = 23,
    T_EQEQ = 24,
    T_NE = 25,
    T_GT = 26,
    T_LT = 27,
    T_GE = 28,
    T_LE = 29,
    /* keywords printed as themselves */
    T_KW_IF = 40,
    T_KW_ELSE = 41,
    T_KW_WHILE = 42,
    T_KW_FOR = 43,
    T_KW_IN = 44,
    T_KW_RETURN = 45,
    T_KW_BREAK = 46,
    T_KW_CONTINUE = 47,
    T_KW_FN = 48,
    T_KW_TYPE = 49,
    T_KW_MOD = 50,
    T_KW_IMPORT = 51,
    T_KW_PRINT = 52,
    T_KW_PRINT_STR = 53,
    T_KW_PRINT_NUM = 54,
    T_KW_TRUE = 55,
    T_KW_FALSE = 56,
    T_KW_AND = 57,
    T_KW_OR = 58,
    T_KW_NOT = 59,
};

typedef struct {
    int kind;
    int64_t ival;
    double fval;
    char *sval; /* owned for string/word/root/template/directive */
    int line;
    int col;
} Token;

typedef struct {
    const unsigned char *src;
    size_t len;
    size_t pos;
    int line;
    int col;
    Token *toks;
    int ntoks;
    int cap;
} Lexer;

/* ---------- UTF-8 ---------- */
static int utf8_decode(const unsigned char *s, size_t n, size_t *adv, uint32_t *cp) {
    if (n == 0) return 0;
    unsigned char c0 = s[0];
    if (c0 < 0x80) {
        *cp = c0;
        *adv = 1;
        return 1;
    }
    if ((c0 & 0xE0) == 0xC0 && n >= 2) {
        *cp = ((c0 & 0x1F) << 6) | (s[1] & 0x3F);
        *adv = 2;
        return 1;
    }
    if ((c0 & 0xF0) == 0xE0 && n >= 3) {
        *cp = ((c0 & 0x0F) << 12) | ((s[1] & 0x3F) << 6) | (s[2] & 0x3F);
        *adv = 3;
        return 1;
    }
    if ((c0 & 0xF8) == 0xF0 && n >= 4) {
        *cp = ((c0 & 0x07) << 18) | ((s[1] & 0x3F) << 12) | ((s[2] & 0x3F) << 6) | (s[3] & 0x3F);
        *adv = 4;
        return 1;
    }
    *cp = c0;
    *adv = 1;
    return 0;
}

static int peek_cp(Lexer *L, uint32_t *cp, size_t *adv) {
    if (L->pos >= L->len) return 0;
    return utf8_decode(L->src + L->pos, L->len - L->pos, adv, cp);
}

static void advance_cp(Lexer *L) {
    uint32_t cp;
    size_t adv;
    if (!peek_cp(L, &cp, &adv)) return;
    if (cp == '\n') {
        L->line++;
        L->col = 1;
    } else {
        L->col++;
    }
    L->pos += adv;
}

static bool is_cons(uint32_t c) {
    static const char *set =
        "БВГДЖЗЙКЛМНПРСТФХЦЧШЩбвгджзйклмнпрстфхцчшщ";
    char buf[8];
    int n = 0;
    if (c < 0x80) {
        buf[0] = (char)c;
        n = 1;
    } else if (c < 0x800) {
        buf[0] = (char)(0xC0 | (c >> 6));
        buf[1] = (char)(0x80 | (c & 0x3F));
        n = 2;
    } else {
        buf[0] = (char)(0xE0 | (c >> 12));
        buf[1] = (char)(0x80 | ((c >> 6) & 0x3F));
        buf[2] = (char)(0x80 | (c & 0x3F));
        n = 3;
    }
    buf[n] = 0;
    return strstr(set, buf) != NULL;
}

static bool is_full_vowel(uint32_t c) {
    return c == 0x0410 || c == 0x0415 || c == 0x0401 || c == 0x0418 || c == 0x041E ||
           c == 0x0423 || c == 0x042B || c == 0x042D || c == 0x0430 || c == 0x0435 ||
           c == 0x0451 || c == 0x0438 || c == 0x043E || c == 0x0443 || c == 0x044B ||
           c == 0x044D;
}

static bool is_weak(uint32_t c) {
    return c == 0x042E || c == 0x042F || c == 0x044E || c == 0x044F; /* ЮЯюя */
}

static bool is_sign(uint32_t c) {
    return c == 0x042C || c == 0x042A || c == 0x044C || c == 0x044A; /* ЬЪьъ */
}

static bool is_vowel(uint32_t c) { return is_full_vowel(c) || is_weak(c); }
static bool is_russian(uint32_t c) { return is_cons(c) || is_vowel(c) || is_sign(c); }
static bool is_skeleton(uint32_t c) { return is_cons(c) || is_weak(c) || is_sign(c); }

static bool is_upper_cyr(uint32_t c) {
    return (c >= 0x0410 && c <= 0x042F) || c == 0x0401;
}
static bool is_lower_cyr(uint32_t c) {
    return (c >= 0x0430 && c <= 0x044F) || c == 0x0451;
}

static void skip_ws(Lexer *L) {
    for (;;) {
        uint32_t cp;
        size_t adv;
        if (!peek_cp(L, &cp, &adv)) break;
        if (cp == 0xFEFF) {
            advance_cp(L);
            continue;
        }
        if (cp == ' ' || cp == '\t' || cp == '\r' || cp == '\n') {
            advance_cp(L);
            continue;
        }
        if (cp == '/' && L->pos + 1 < L->len && L->src[L->pos + 1] == '/') {
            while (peek_cp(L, &cp, &adv) && cp != '\n') advance_cp(L);
            continue;
        }
        break;
    }
}

static void push_tok(Lexer *L, Token t) {
    if (L->ntoks >= L->cap) {
        L->cap = L->cap ? L->cap * 2 : 256;
        L->toks = (Token *)realloc(L->toks, (size_t)L->cap * sizeof(Token));
    }
    L->toks[L->ntoks++] = t;
}

static char *dup_range(Lexer *L, size_t start, size_t end) {
    size_t n = end - start;
    char *s = (char *)malloc(n + 1);
    memcpy(s, L->src + start, n);
    s[n] = 0;
    return s;
}

static int classify_keyword(const char *w) {
    if (!strcmp(w, "если")) return T_KW_IF;
    if (!strcmp(w, "иначе")) return T_KW_ELSE;
    if (!strcmp(w, "пока")) return T_KW_WHILE;
    if (!strcmp(w, "для")) return T_KW_FOR;
    if (!strcmp(w, "в")) return T_KW_IN;
    if (!strcmp(w, "возврат")) return T_KW_RETURN;
    if (!strcmp(w, "прервать")) return T_KW_BREAK;
    if (!strcmp(w, "продолжить")) return T_KW_CONTINUE;
    if (!strcmp(w, "функция")) return T_KW_FN;
    if (!strcmp(w, "тип")) return T_KW_TYPE;
    if (!strcmp(w, "модуль")) return T_KW_MOD;
    if (!strcmp(w, "импорт")) return T_KW_IMPORT;
    if (!strcmp(w, "печать")) return T_KW_PRINT;
    if (!strcmp(w, "печать_строки")) return T_KW_PRINT_STR;
    if (!strcmp(w, "печать_число")) return T_KW_PRINT_NUM;
    if (!strcmp(w, "истина")) return T_KW_TRUE;
    if (!strcmp(w, "ложь")) return T_KW_FALSE;
    if (!strcmp(w, "и")) return T_KW_AND;
    if (!strcmp(w, "или")) return T_KW_OR;
    if (!strcmp(w, "не")) return T_KW_NOT;
    return -1;
}

static int classify_word(char *w) {
    int kw = classify_keyword(w);
    if (kw >= 0) return kw;

    /* decode to codepoints */
    uint32_t cps[64];
    int n = 0;
    size_t i = 0;
    size_t len = strlen(w);
    bool has_under_digit = false;
    while (i < len && n < 64) {
        size_t adv;
        uint32_t cp;
        if (!utf8_decode((const unsigned char *)w + i, len - i, &adv, &cp)) break;
        if (cp == '_' || (cp >= '0' && cp <= '9')) has_under_digit = true;
        cps[n++] = cp;
        i += adv;
    }
    if (n == 0 || has_under_digit) return T_WORD;

    bool all_upperish = true;
    bool any_upper = false;
    for (int k = 0; k < n; k++) {
        if (is_lower_cyr(cps[k]) && !is_sign(cps[k])) all_upperish = false;
        if (is_upper_cyr(cps[k])) any_upper = true;
    }
    if (!all_upperish || !any_upper) return T_WORD;

    bool all_skel = true;
    for (int k = 0; k < n; k++)
        if (!is_skeleton(cps[k])) all_skel = false;
    if (n == 3 && all_skel) return T_ROOT;

    if (n == 4 && is_cons(cps[0]) && is_vowel(cps[1]) && is_cons(cps[2]) && is_vowel(cps[3]))
        return T_TEMPLATE;

    int skel = 0;
    bool mod = false;
    for (int k = 0; k < n; k++) {
        if (is_skeleton(cps[k])) skel++;
        if (is_full_vowel(cps[k])) mod = true;
    }
    if (mod && skel == 3 && n >= 4 && n <= 6) return T_TEMPLATE;
    return T_WORD;
}

static int lex_all(Lexer *L) {
    for (;;) {
        skip_ws(L);
        int line = L->line, col = L->col;
        uint32_t cp;
        size_t adv;
        if (!peek_cp(L, &cp, &adv)) {
            Token t = {0};
            t.kind = T_EOF;
            t.line = line;
            t.col = col;
            push_tok(L, t);
            break;
        }

        Token t;
        memset(&t, 0, sizeof(t));
        t.line = line;
        t.col = col;

        if (cp == '>') {
            advance_cp(L);
            if (peek_cp(L, &cp, &adv) && cp == '>') {
                advance_cp(L);
                if (peek_cp(L, &cp, &adv) && cp == '>') {
                    advance_cp(L);
                    t.kind = T_NET_IMPULSE;
                } else
                    t.kind = T_IMPULSE;
            } else if (peek_cp(L, &cp, &adv) && cp == '=') {
                advance_cp(L);
                t.kind = T_GE;
            } else
                t.kind = T_GT;
        } else if (cp == '<') {
            advance_cp(L);
            if (peek_cp(L, &cp, &adv) && cp == '=') {
                advance_cp(L);
                t.kind = T_LE;
            } else
                t.kind = T_LT;
        } else if (cp == '|') {
            advance_cp(L);
            if (!(peek_cp(L, &cp, &adv) && cp == '|')) {
                fprintf(stderr, "Ошибка лексики [%d:%d]: Ожидалось '||'\n", line, col);
                return 1;
            }
            advance_cp(L);
            t.kind = T_CHOIR;
        } else if (cp == '!') {
            advance_cp(L);
            if (peek_cp(L, &cp, &adv) && cp == '=') {
                advance_cp(L);
                t.kind = T_NE;
            } else
                t.kind = T_FIX;
        } else if (cp == '~') {
            advance_cp(L);
            t.kind = T_FLOW;
        } else if (cp == '=') {
            advance_cp(L);
            if (peek_cp(L, &cp, &adv) && cp == '=') {
                advance_cp(L);
                t.kind = T_EQEQ;
            } else
                t.kind = T_EQ;
        } else if (cp == ':') {
            advance_cp(L);
            t.kind = T_COLON;
        } else if (cp == '-') {
            advance_cp(L);
            t.kind = T_DASH;
        } else if (cp == '+') {
            advance_cp(L);
            t.kind = T_PLUS;
        } else if (cp == '*') {
            advance_cp(L);
            t.kind = T_STAR;
        } else if (cp == '/') {
            advance_cp(L);
            t.kind = T_SLASH;
        } else if (cp == '(') {
            advance_cp(L);
            t.kind = T_LPAREN;
        } else if (cp == ')') {
            advance_cp(L);
            t.kind = T_RPAREN;
        } else if (cp == '{') {
            advance_cp(L);
            t.kind = T_LBRACE;
        } else if (cp == '}') {
            advance_cp(L);
            t.kind = T_RBRACE;
        } else if (cp == ',') {
            advance_cp(L);
            t.kind = T_COMMA;
        } else if (cp == '"') {
            advance_cp(L);
            size_t start = L->pos;
            for (;;) {
                if (!peek_cp(L, &cp, &adv)) {
                    fprintf(stderr, "Ошибка лексики [%d:%d]: Незакрытая строка\n", line, col);
                    return 1;
                }
                if (cp == '"') {
                    t.sval = dup_range(L, start, L->pos);
                    advance_cp(L);
                    t.kind = T_STRING;
                    break;
                }
                if (cp == '\n') {
                    fprintf(stderr, "Ошибка лексики [%d:%d]: Незакрытая строка\n", line, col);
                    return 1;
                }
                advance_cp(L);
            }
        } else if (cp == '#') {
            advance_cp(L);
            size_t start = L->pos;
            while (peek_cp(L, &cp, &adv) &&
                   ((cp >= 'a' && cp <= 'z') || (cp >= 'A' && cp <= 'Z') || cp == '_')) {
                advance_cp(L);
            }
            t.sval = dup_range(L, start, L->pos);
            if (!t.sval[0]) {
                fprintf(stderr, "Ошибка лексики [%d:%d]: Директива без имени\n", line, col);
                return 1;
            }
            if (strcmp(t.sval, "include") && strcmp(t.sval, "define") && strcmp(t.sval, "link") &&
                strcmp(t.sval, "runtime")) {
                fprintf(stderr, "Ошибка лексики [%d:%d]: Неизвестная директива '#%s'\n", line, col,
                        t.sval);
                return 1;
            }
            t.kind = T_DIRECTIVE;
        } else if (cp >= '0' && cp <= '9') {
            size_t start = L->pos;
            while (peek_cp(L, &cp, &adv) && cp >= '0' && cp <= '9') advance_cp(L);
            bool is_float = false;
            if (peek_cp(L, &cp, &adv) && cp == '.') {
                size_t save = L->pos;
                int scol = L->col;
                advance_cp(L);
                if (peek_cp(L, &cp, &adv) && cp >= '0' && cp <= '9') {
                    is_float = true;
                    while (peek_cp(L, &cp, &adv) && cp >= '0' && cp <= '9') advance_cp(L);
                } else {
                    L->pos = save;
                    L->col = scol;
                }
            }
            char *num = dup_range(L, start, L->pos);
            if (is_float) {
                t.kind = T_FLOAT;
                t.fval = atof(num);
            } else {
                t.kind = T_INT;
                t.ival = atoll(num);
            }
            free(num);
        } else if (is_russian(cp)) {
            size_t start = L->pos;
            while (peek_cp(L, &cp, &adv)) {
                if (cp < 128 && isalpha((int)cp)) {
                    fprintf(stderr,
                            "Ошибка лексики [%d:%d]: Недопустимый символ: латиница не разрешена\n",
                            line, col);
                    return 1;
                }
                if (is_russian(cp) || cp == '_')
                    advance_cp(L);
                else
                    break;
            }
            t.sval = dup_range(L, start, L->pos);
            t.kind = classify_word(t.sval);
        } else if (cp < 128 && isalpha((int)cp)) {
            fprintf(stderr, "Ошибка лексики [%d:%d]: Недопустимый символ: латиница не разрешена\n",
                    line, col);
            return 1;
        } else {
            fprintf(stderr, "Ошибка лексики [%d:%d]: Неизвестный символ\n", line, col);
            return 1;
        }
        push_tok(L, t);
    }
    return 0;
}

static void print_tok(const Token *t) {
    printf("  [%d:%d] ", t->line, t->col);
    switch (t->kind) {
        case T_EOF: printf("КонецФайла"); break;
        case T_IMPULSE: printf(">>"); break;
        case T_NET_IMPULSE: printf(">>>"); break;
        case T_CHOIR: printf("||"); break;
        case T_FIX: printf("!"); break;
        case T_FLOW: printf("~"); break;
        case T_EQ: printf("="); break;
        case T_EQEQ: printf("=="); break;
        case T_NE: printf("!="); break;
        case T_GT: printf(">"); break;
        case T_LT: printf("<"); break;
        case T_GE: printf(">="); break;
        case T_LE: printf("<="); break;
        case T_COLON: printf(":"); break;
        case T_DASH: printf("-"); break;
        case T_PLUS: printf("+"); break;
        case T_STAR: printf("*"); break;
        case T_SLASH: printf("/"); break;
        case T_LPAREN: printf("("); break;
        case T_RPAREN: printf(")"); break;
        case T_LBRACE: printf("{"); break;
        case T_RBRACE: printf("}"); break;
        case T_COMMA: printf(","); break;
        case T_ROOT: printf("Корень(%s)", t->sval ? t->sval : ""); break;
        case T_TEMPLATE: printf("Шаблон(%s)", t->sval ? t->sval : ""); break;
        case T_WORD: printf("Слово(%s)", t->sval ? t->sval : ""); break;
        case T_INT: printf("Целое(%lld)", (long long)t->ival); break;
        case T_FLOAT: printf("Вещественное(%g)", t->fval); break;
        case T_STRING: printf("Строка(\"%s\")", t->sval ? t->sval : ""); break;
        case T_DIRECTIVE: printf("Директива(%s)", t->sval ? t->sval : ""); break;
        case T_KW_IF: printf("если"); break;
        case T_KW_ELSE: printf("иначе"); break;
        case T_KW_WHILE: printf("пока"); break;
        case T_KW_FOR: printf("для"); break;
        case T_KW_IN: printf("в"); break;
        case T_KW_RETURN: printf("возврат"); break;
        case T_KW_BREAK: printf("прервать"); break;
        case T_KW_CONTINUE: printf("продолжить"); break;
        case T_KW_FN: printf("функция"); break;
        case T_KW_TYPE: printf("тип"); break;
        case T_KW_MOD: printf("модуль"); break;
        case T_KW_IMPORT: printf("импорт"); break;
        case T_KW_PRINT: printf("печать"); break;
        case T_KW_PRINT_STR: printf("печать_строки"); break;
        case T_KW_PRINT_NUM: printf("печать_число"); break;
        case T_KW_TRUE: printf("истина"); break;
        case T_KW_FALSE: printf("ложь"); break;
        case T_KW_AND: printf("и"); break;
        case T_KW_OR: printf("или"); break;
        case T_KW_NOT: printf("не"); break;
        default: printf("?"); break;
    }
    printf("\n");
}

#ifdef ZIP_LEXER_MAIN
int main(int argc, char **argv) {
    const char *path = argc > 1 ? argv[1] : NULL;
    unsigned char *buf = NULL;
    size_t n = 0;
    if (!path) {
        fprintf(stderr, "Использование: %s <файл.zip>\n", argv[0]);
        return 1;
    }
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

    Lexer L = {0};
    L.src = buf;
    L.len = n;
    L.line = 1;
    L.col = 1;
    L.cap = 256;
    L.toks = (Token *)malloc((size_t)L.cap * sizeof(Token));

    if (lex_all(&L) != 0) {
        free(buf);
        free(L.toks);
        return 1;
    }

    printf("=== ZIP-лексер ===\n");
    printf("Файл: %s\n", path);
    printf("Токенов: %d\n", L.ntoks);
    for (int i = 0; i < L.ntoks; i++) print_tok(&L.toks[i]);

    for (int i = 0; i < L.ntoks; i++) free(L.toks[i].sval);
    free(L.toks);
    free(buf);
    return 0;
}
#endif
