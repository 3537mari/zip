#include "zipb.h"

typedef struct {
    const unsigned char *data;
    size_t len;
    size_t pos;
    int line;
    int col;
} Lexer;

static int is_cyr_start(const unsigned char *b, size_t rem) {
    if (rem < 2) return 0;
    return (b[0] == 0xD0 || b[0] == 0xD1) && b[1] >= 0x80 && b[1] <= 0xBF;
}

static unsigned char cur(Lexer *L) {
    return L->pos < L->len ? L->data[L->pos] : 0;
}

static void adv(Lexer *L) {
    if (L->pos >= L->len) return;
    if (L->data[L->pos] == '\n') { L->line++; L->col = 1; }
    else L->col++;
    L->pos++;
}

static void skip(Lexer *L) {
    for (;;) {
        unsigned char c = cur(L);
        if (c == ' ' || c == '\t' || c == '\n' || c == '\r') { adv(L); continue; }
        if (c == '/' && L->pos + 1 < L->len && L->data[L->pos + 1] == '/') {
            while (L->pos < L->len && cur(L) != '\n') adv(L);
            continue;
        }
        if (c == '/' && L->pos + 1 < L->len && L->data[L->pos + 1] == '*') {
            adv(L); adv(L);
            while (L->pos + 1 < L->len) {
                if (cur(L) == '*' && L->data[L->pos + 1] == '/') { adv(L); adv(L); break; }
                adv(L);
            }
            continue;
        }
        break;
    }
}

static Token tok_num(Lexer *L) {
    size_t start = L->pos;
    int dot = 0;
    while (L->pos < L->len) {
        unsigned char c = cur(L);
        if (isdigit(c)) adv(L);
        else if (c == '.' && !dot && L->pos + 1 < L->len && isdigit(L->data[L->pos + 1])) {
            dot = 1; adv(L);
        } else break;
    }
    char buf[96];
    size_t n = L->pos - start;
    if (n >= sizeof(buf)) n = sizeof(buf) - 1;
    memcpy(buf, L->data + start, n);
    buf[n] = 0;
    Token t = {0};
    t.line = L->line; t.col = L->col;
    if (dot) { t.type = TK_FLOAT; t.f64 = atof(buf); }
    else { t.type = TK_INT; t.i64 = atoll(buf); }
    return t;
}

static Token tok_str(Lexer *L) {
    adv(L); /* " */
    size_t start = L->pos;
    while (L->pos < L->len && cur(L) != '"') {
        if (cur(L) == '\\') adv(L);
        adv(L);
    }
    size_t n = L->pos - start;
    char *s = (char *)zip_alloc(n + 1);
    memcpy(s, L->data + start, n);
    s[n] = 0;
    if (cur(L) == '"') adv(L);
    Token t = {0};
    t.type = TK_STRING;
    t.str = s;
    t.line = L->line; t.col = L->col;
    return t;
}

static TokenType kw(const char *w) {
    if (!strcmp(w, "если")) return TK_IF;
    if (!strcmp(w, "иначе")) return TK_ELSE;
    if (!strcmp(w, "пока")) return TK_WHILE;
    if (!strcmp(w, "для")) return TK_FOR;
    if (!strcmp(w, "в")) return TK_IN;
    if (!strcmp(w, "возврат")) return TK_RETURN;
    if (!strcmp(w, "прервать")) return TK_BREAK;
    if (!strcmp(w, "продолжить")) return TK_CONTINUE;
    if (!strcmp(w, "функция")) return TK_FUNCTION;
    if (!strcmp(w, "тип")) return TK_TYPE;
    if (!strcmp(w, "агент")) return TK_AGENT;
    if (!strcmp(w, "истина")) return TK_TRUE;
    if (!strcmp(w, "ложь")) return TK_FALSE;
    if (!strcmp(w, "и")) return TK_AND;
    if (!strcmp(w, "или")) return TK_OR;
    if (!strcmp(w, "не")) return TK_NOT;
    if (!strcmp(w, "ничто")) return TK_NOTHING;
    if (!strcmp(w, "нечто")) return TK_SOMETHING;
    if (!strcmp(w, "успех")) return TK_SUCCESS;
    if (!strcmp(w, "ошибка")) return TK_ERROR;
    return TK_IDENT;
}

static Token tok_word(Lexer *L) {
    size_t start = L->pos;
    while (L->pos < L->len) {
        const unsigned char *b = L->data + L->pos;
        if (is_cyr_start(b, L->len - L->pos)) { adv(L); adv(L); }
        else if ((cur(L) >= 'a' && cur(L) <= 'z') || (cur(L) >= 'A' && cur(L) <= 'Z')
                 || cur(L) == '_' || isdigit(cur(L))) {
            adv(L);
        } else break;
    }
    size_t n = L->pos - start;
    char *w = (char *)zip_alloc(n + 1);
    memcpy(w, L->data + start, n);
    w[n] = 0;
    Token t = {0};
    t.str = w;
    t.type = kw(w);
    t.line = L->line; t.col = L->col;
    /* 3 Cyrillic letters (6 UTF-8 bytes) → root candidate */
    if (t.type == TK_IDENT && n == 6 && is_cyr_start((const unsigned char *)w, n))
        t.type = TK_ROOT;
    return t;
}

Token *lex(const char *src, size_t *out_n) {
    Lexer L = { (const unsigned char *)src, strlen(src), 0, 1, 1 };
    size_t cap = 256;
    Token *toks = (Token *)zip_alloc(cap * sizeof(Token));
    size_t n = 0;

    while (L.pos < L.len) {
        skip(&L);
        if (L.pos >= L.len) break;
        Token t = {0};
        t.line = L.line; t.col = L.col;
        unsigned char c = cur(&L);

        if (c == '>' && L.pos + 1 < L.len && L.data[L.pos + 1] == '>') {
            adv(&L); adv(&L);
            if (cur(&L) == '>') { adv(&L); t.type = TK_NET; }
            else t.type = TK_IMPULSE;
        } else if (c == '|' && L.pos + 1 < L.len && L.data[L.pos + 1] == '|') {
            adv(&L); adv(&L); t.type = TK_CHOIR;
        } else if (c == '!' && !(L.pos + 1 < L.len && L.data[L.pos + 1] == '=')) {
            adv(&L); t.type = TK_PERFECT;
        } else if (c == '~') { adv(&L); t.type = TK_STREAM; }
        else if (c == '=' && L.pos + 1 < L.len && L.data[L.pos + 1] == '=') {
            adv(&L); adv(&L); t.type = TK_EQEQ;
        } else if (c == '!' && L.pos + 1 < L.len && L.data[L.pos + 1] == '=') {
            adv(&L); adv(&L); t.type = TK_NE;
        } else if (c == '<' && L.pos + 1 < L.len && L.data[L.pos + 1] == '=') {
            adv(&L); adv(&L); t.type = TK_LE;
        } else if (c == '>' && L.pos + 1 < L.len && L.data[L.pos + 1] == '=') {
            adv(&L); adv(&L); t.type = TK_GE;
        } else if (c == '-' && L.pos + 1 < L.len && L.data[L.pos + 1] == '>') {
            adv(&L); adv(&L); t.type = TK_ARROW;
        } else if (c == '.' && L.pos + 1 < L.len && L.data[L.pos + 1] == '.') {
            adv(&L); adv(&L); t.type = TK_RANGE;
        } else if (c == '=') { adv(&L); t.type = TK_EQ; }
        else if (c == ':') { adv(&L); t.type = TK_COLON; }
        else if (c == '(') { adv(&L); t.type = TK_LPAREN; }
        else if (c == ')') { adv(&L); t.type = TK_RPAREN; }
        else if (c == '{') { adv(&L); t.type = TK_LBRACE; }
        else if (c == '}') { adv(&L); t.type = TK_RBRACE; }
        else if (c == ',') { adv(&L); t.type = TK_COMMA; }
        else if (c == '+') { adv(&L); t.type = TK_PLUS; }
        else if (c == '-') { adv(&L); t.type = TK_MINUS; }
        else if (c == '*') { adv(&L); t.type = TK_STAR; }
        else if (c == '/') { adv(&L); t.type = TK_SLASH; }
        else if (c == '%') { adv(&L); t.type = TK_PERCENT; }
        else if (c == '<') { adv(&L); t.type = TK_LT; }
        else if (c == '>') { adv(&L); t.type = TK_GT; }
        else if (c == '@') { adv(&L); t.type = TK_AT; }
        else if (c == '#') { adv(&L); t.type = TK_HASH; }
        else if (c == '"') t = tok_str(&L);
        else if (isdigit(c)) t = tok_num(&L);
        else if (is_cyr_start(L.data + L.pos, L.len - L.pos)
                 || (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_') {
            t = tok_word(&L);
        } else {
            fprintf(stderr, "лексер: неизвестный байт 0x%02X строка %d столбец %d\n",
                    c, L.line, L.col);
            free(toks);
            return NULL;
        }

        if (n >= cap) {
            cap *= 2;
            toks = (Token *)zip_realloc(toks, cap * sizeof(Token));
        }
        toks[n++] = t;
    }
    if (n >= cap) toks = (Token *)zip_realloc(toks, (cap + 1) * sizeof(Token));
    toks[n].type = TK_EOF;
    n++;
    *out_n = n;
    return toks;
}
