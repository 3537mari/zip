#include "zipb.h"

typedef struct {
    const unsigned char *data;
    size_t len, pos;
    int line, col;
} Lexer;

static int is_cyr(const unsigned char *b, size_t rem) {
    return rem >= 2 && (b[0] == 0xD0 || b[0] == 0xD1) && b[1] >= 0x80 && b[1] <= 0xBF;
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
    adv(L);
    Buf *tmp = buf_new();
    while (L->pos < L->len && cur(L) != '"') {
        if (cur(L) == '\\' && L->pos + 1 < L->len) {
            adv(L);
            unsigned char e = cur(L);
            if (e == 'n') buf_byte(tmp, '\n');
            else if (e == 't') buf_byte(tmp, '\t');
            else if (e == 'r') buf_byte(tmp, '\r');
            else if (e == '\\') buf_byte(tmp, '\\');
            else if (e == '"') buf_byte(tmp, '"');
            else buf_byte(tmp, e);
            adv(L);
        } else {
            buf_byte(tmp, cur(L));
            adv(L);
        }
    }
    if (cur(L) == '"') adv(L);
    char *s = (char *)zip_alloc(tmp->len + 1);
    memcpy(s, tmp->data, tmp->len);
    s[tmp->len] = 0;
    buf_free(tmp);
    Token t = {0};
    t.type = TK_STRING;
    t.str = s;
    t.line = L->line; t.col = L->col;
    return t;
}

static Token tok_word(Lexer *L) {
    size_t start = L->pos;
    int cyr_chars = 0;
    while (L->pos < L->len) {
        const unsigned char *b = L->data + L->pos;
        if (is_cyr(b, L->len - L->pos)) { adv(L); adv(L); cyr_chars++; }
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
    t.type = keyword_lookup(w);
    t.line = L->line; t.col = L->col;
    /* 3 Cyrillic letters, not a keyword → root */
    if (t.type == TK_IDENT && cyr_chars == 3 && n == 6)
        t.type = TK_ROOT;
    /* 5 Cyrillic letters pattern like САПАР → template */
    if (t.type == TK_IDENT && cyr_chars == 5 && n == 10)
        t.type = TK_TEMPLATE;
    return t;
}

Token *lex(const char *src, size_t *out_n) {
    Lexer L = { (const unsigned char *)src, strlen(src), 0, 1, 1 };
    size_t cap = 512;
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
            else if (cur(&L) == '=') { /* shouldn't */ t.type = TK_IMPULSE; }
            else t.type = TK_IMPULSE;
        } else if (c == '|' && L.pos + 1 < L.len && L.data[L.pos + 1] == '|') {
            adv(&L); adv(&L); t.type = TK_CHOIR;
        } else if (c == '=' && L.pos + 1 < L.len && L.data[L.pos + 1] == '=') {
            adv(&L); adv(&L); t.type = TK_EQEQ;
        } else if (c == '!' && L.pos + 1 < L.len && L.data[L.pos + 1] == '=') {
            adv(&L); adv(&L); t.type = TK_NE;
        } else if (c == '!' ) {
            adv(&L); t.type = TK_PERFECT;
        } else if (c == '<' && L.pos + 1 < L.len && L.data[L.pos + 1] == '=') {
            adv(&L); adv(&L); t.type = TK_LE;
        } else if (c == '>' && L.pos + 1 < L.len && L.data[L.pos + 1] == '=') {
            adv(&L); adv(&L); t.type = TK_GE;
        } else if (c == '-' && L.pos + 1 < L.len && L.data[L.pos + 1] == '>') {
            adv(&L); adv(&L); t.type = TK_ARROW;
        } else if (c == '.' && L.pos + 1 < L.len && L.data[L.pos + 1] == '.') {
            adv(&L); adv(&L); t.type = TK_RANGE;
        } else if (c == '.') {
            adv(&L); t.type = TK_DOT;
        } else if (c == '~') { adv(&L); t.type = TK_STREAM; }
        else if (c == '=') { adv(&L); t.type = TK_EQ; }
        else if (c == ':') { adv(&L); t.type = TK_COLON; }
        else if (c == '(') { adv(&L); t.type = TK_LPAREN; }
        else if (c == ')') { adv(&L); t.type = TK_RPAREN; }
        else if (c == '{') { adv(&L); t.type = TK_LBRACE; }
        else if (c == '}') { adv(&L); t.type = TK_RBRACE; }
        else if (c == '[') { adv(&L); t.type = TK_LBRACK; }
        else if (c == ']') { adv(&L); t.type = TK_RBRACK; }
        else if (c == ';') { adv(&L); t.type = TK_SEMI; }
        else if (c == ',') { adv(&L); t.type = TK_COMMA; }
        else if (c == '+') { adv(&L); t.type = TK_PLUS; }
        else if (c == '-') { adv(&L); t.type = TK_MINUS; }
        else if (c == '*') { adv(&L); t.type = TK_STAR; }
        else if (c == '/') { adv(&L); t.type = TK_SLASH; }
        else if (c == '%') { adv(&L); t.type = TK_PERCENT; }
        else if (c == '<') { adv(&L); t.type = TK_LT; }
        else if (c == '>') { adv(&L); t.type = TK_GT; }
        else if (c == '&') { adv(&L); t.type = TK_AMP; }
        else if (c == '@') { adv(&L); t.type = TK_AT; }
        else if (c == '#') { adv(&L); t.type = TK_HASH; }
        else if (c == '"') t = tok_str(&L);
        else if (isdigit(c)) t = tok_num(&L);
        else if (is_cyr(L.data + L.pos, L.len - L.pos)
                 || (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_') {
            t = tok_word(&L);
        } else {
            fprintf(stderr, "лексер: неизвестный 0x%02X @%d:%d\n", c, L.line, L.col);
            tokens_free(toks, n);
            return NULL;
        }

        if (n >= cap) {
            cap *= 2;
            toks = (Token *)zip_realloc(toks, cap * sizeof(Token));
        }
        toks[n++] = t;
    }
    if (n >= cap) toks = (Token *)zip_realloc(toks, (n + 1) * sizeof(Token));
    toks[n].type = TK_EOF;
    n++;
    *out_n = n;
    return toks;
}

void tokens_free(Token *t, size_t n) {
    if (!t) return;
    for (size_t i = 0; i < n; i++) free(t[i].str);
    free(t);
}
