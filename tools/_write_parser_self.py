# -*- coding: utf-8 -*-
"""Generate zip-self parser specs + парсер_rt.c (UTF-8 safe)."""
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SELF = ROOT / "zip-self"

дерево = r"""// ============================================
// ZIP-SELF: AST (дерево разбора)
// ============================================
// Спецификация узлов; исполнение — парсер_rt.c
#runtime("parser_ast")
"""

парсер = r"""// ============================================
// ZIP-SELF: ПАРСЕР НА САМОМ ZIP
// ============================================
// Спецификация рекурсивного спуска; исполнение — парсер_rt.c
// (лексер ZIP + каскад приоритетов + поток >> / || / ! / ~)
#runtime("parser_zip")
"""

простая = "// Простая программа\nСАПАР(1, 2) >> ЧАТАВ!\n"

(SELF / "дерево.zip").write_text(дерево, encoding="utf-8", newline="\n")
(SELF / "дерево_ast.zip").write_text(дерево, encoding="utf-8", newline="\n")
(SELF / "парсер.zip").write_text(парсер, encoding="utf-8", newline="\n")
(SELF / "parser.zip").write_text(парсер, encoding="utf-8", newline="\n")
(ROOT / "examples" / "простая.zip").write_text(простая, encoding="utf-8", newline="\n")
(ROOT / "examples" / "simple.zip").write_text(простая, encoding="utf-8", newline="\n")
print("specs ok")

# --- C runtime (large) ---
c = r'''/* ZIP-SELF: parser runtime — mirror of zipc/src/парсер.rs + дерево.rs Debug */
#define _CRT_SECURE_NO_WARNINGS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdarg.h>

/* Embed lexer (no main) */
#include "лексика_rt.c"

/* ===== AST ===== */
typedef enum {
    N_PROG = 0,
    N_ROOT,
    N_VERB,
    N_FIX,
    N_ASYNC,
    N_CHOIR,
    N_INT,
    N_FLOAT,
    N_STR,
    N_IDENT,
    N_BLOCK,
    N_IF,
    N_WHILE,
    N_FOR,
    N_FN,
    N_RETURN,
    N_BREAK,
    N_CONTINUE,
    N_BINOP,
    N_CMP,
    N_ASSIGN,
    N_CALL,
    N_TRUE,
    N_FALSE,
    N_AND,
    N_OR,
    N_NOT,
    N_NEG,
    N_EXT,
    N_RUNTIME
} NKind;

typedef struct Node Node;
struct Node {
    NKind kind;
    char *name;       /* глагол / имя / корень / runtime */
    char *name2;      /* библиотека for EXT, or unused */
    char *op;         /* operator string */
    int64_t ival;
    double fval;
    char *sval;
    Node **kids;
    int nkids;
    int has_next;     /* VERB: last kid is следующий */
    char **pair_a;    /* ROOT templates left */
    char **pair_b;
    int npairs;
    char **pnames;
    char **ptypes;
    int nparams;
    char *ret_type;   /* NULL = None */
    int has_ret_type;
    int has_else;     /* IF: last kid is else */
    int has_return_val;
};

typedef struct {
    Token *toks;
    int ntoks;
    int pos;
} Parser;

static Node *node_new(NKind k) {
    Node *n = (Node *)calloc(1, sizeof(Node));
    n->kind = k;
    return n;
}

static void node_add(Node *n, Node *c) {
    if (!c) return;
    n->kids = (Node **)realloc(n->kids, (size_t)(n->nkids + 1) * sizeof(Node *));
    n->kids[n->nkids++] = c;
}

static char *xstrdup(const char *s) {
    if (!s) return NULL;
    size_t n = strlen(s);
    char *p = (char *)malloc(n + 1);
    memcpy(p, s, n + 1);
    return p;
}

static Token *Pcur(Parser *P) {
    if (P->pos >= P->ntoks) return &P->toks[P->ntoks - 1];
    return &P->toks[P->pos];
}

static Token *Plook(Parser *P, int off) {
    int i = P->pos + off;
    if (i < 0 || i >= P->ntoks) return &P->toks[P->ntoks - 1];
    return &P->toks[i];
}

static int Pkind(Parser *P) { return Pcur(P)->kind; }

static int Pis(Parser *P, int k) { return Pkind(P) == k; }

static int Pis_any_name(Parser *P) {
    int k = Pkind(P);
    return k == T_ROOT || k == T_TEMPLATE || k == T_WORD ||
           (k >= T_KW_IF && k <= T_KW_NOT && k != T_KW_TRUE && k != T_KW_FALSE);
}

static int is_kw_name_kind(int k) {
    return k == T_KW_IF || k == T_KW_ELSE || k == T_KW_WHILE || k == T_KW_FOR ||
           k == T_KW_IN || k == T_KW_RETURN || k == T_KW_BREAK || k == T_KW_CONTINUE ||
           k == T_KW_FN || k == T_KW_TYPE || k == T_KW_MOD || k == T_KW_IMPORT ||
           k == T_KW_PRINT || k == T_KW_PRINT_STR || k == T_KW_PRINT_NUM ||
           k == T_KW_AND || k == T_KW_OR || k == T_KW_NOT;
}

static const char *kw_text(int k) {
    switch (k) {
        case T_KW_IF: return "если";
        case T_KW_ELSE: return "иначе";
        case T_KW_WHILE: return "пока";
        case T_KW_FOR: return "для";
        case T_KW_IN: return "в";
        case T_KW_RETURN: return "возврат";
        case T_KW_BREAK: return "прервать";
        case T_KW_CONTINUE: return "продолжить";
        case T_KW_FN: return "функция";
        case T_KW_TYPE: return "тип";
        case T_KW_MOD: return "модуль";
        case T_KW_IMPORT: return "импорт";
        case T_KW_PRINT: return "печать";
        case T_KW_PRINT_STR: return "печать_строки";
        case T_KW_PRINT_NUM: return "печать_число";
        case T_KW_AND: return "и";
        case T_KW_OR: return "или";
        case T_KW_NOT: return "не";
        default: return "?";
    }
}

static Token Peat(Parser *P) {
    Token t = *Pcur(P);
    if (t.kind != T_EOF && P->pos < P->ntoks) P->pos++;
    return t;
}

static int Pexpect(Parser *P, int k) {
    if (Pis(P, k)) { Peat(P); return 1; }
    return 0;
}

static void perr(Parser *P, const char *msg) {
    Token *t = Pcur(P);
    fprintf(stderr, "Ошибка парсера [%d:%d]: %s\n", t->line, t->col, msg);
}

/* forward decls */
static Node *parse_stmt(Parser *P);
static Node *parse_choir(Parser *P);
static Node *parse_impulse(Parser *P);
static Node *parse_postfix(Parser *P);
static Node *parse_expr(Parser *P);
static Node *parse_expr_arg(Parser *P);
static Node *parse_primary(Parser *P, int as_arg);
static Node *parse_block(Parser *P);
static Node *parse_arg(Parser *P);
static Node *attach_next(Node *left, Node *right);

static char *parse_ident_name(Parser *P) {
    Token t = Peat(P);
    if (t.kind == T_ROOT || t.kind == T_TEMPLATE || t.kind == T_WORD)
        return xstrdup(t.sval ? t.sval : "");
    if (is_kw_name_kind(t.kind))
        return xstrdup(kw_text(t.kind));
    perr(P, "ожидался идентификатор");
    return xstrdup("?");
}

static const char *verb_name(Node *n) {
    if (!n) return NULL;
    if (n->kind == N_VERB || n->kind == N_CALL || n->kind == N_IDENT)
        return n->name;
    return NULL;
}

static Node *attach_next(Node *left, Node *right) {
    if (!left) return right;
    if (left->kind == N_VERB && !left->has_next) {
        node_add(left, right);
        left->has_next = 1;
        return left;
    }
    if (left->kind == N_CALL) {
        /* convert to verb with next */
        left->kind = N_VERB;
        node_add(left, right);
        left->has_next = 1;
        return left;
    }
    if (left->kind == N_IDENT) {
        Node *v = node_new(N_VERB);
        v->name = left->name;
        left->name = NULL;
        free(left);
        node_add(v, right);
        v->has_next = 1;
        return v;
    }
    perr(NULL, ">> ожидает глагол слева");
    node_add(left, right);
    return left;
}

static int looks_like_flow(Parser *P) {
    int i = P->pos;
    int depth = 0;
    while (i < P->ntoks) {
        int k = P->toks[i].kind;
        if (k == T_RPAREN && depth == 0) break;
        if (k == T_LPAREN) depth++;
        else if (k == T_RPAREN) depth = depth > 0 ? depth - 1 : 0;
        else if (depth == 0 &&
                 (k == T_IMPULSE || k == T_NET_IMPULSE || k == T_CHOIR || k == T_FLOW))
            return 1;
        else if (k == T_EOF) break;
        i++;
    }
    return 0;
}

static Node *parse_arg_list(Parser *P, const char *verb) {
    (void)verb;
    Node *dummy = node_new(N_BLOCK); /* temporary holder */
    if (Pis(P, T_RPAREN)) {
        Peat(P);
        return dummy;
    }
    for (;;) {
        node_add(dummy, parse_arg(P));
        if (Pis(P, T_COMMA)) {
            Peat(P);
            continue;
        }
        break;
    }
    if (!Pexpect(P, T_RPAREN)) perr(P, "ожидалась )");
    return dummy;
}

static Node *finish_name_or_call(Parser *P, char *name, int kind_was, int as_arg) {
    if (Pis(P, T_LPAREN)) {
        Peat(P);
        Node *args = parse_arg_list(P, name);
        Node *n;
        if (kind_was == T_WORD || is_kw_name_kind(kind_was)) {
            n = node_new(N_CALL);
            n->name = name;
        } else {
            n = node_new(N_VERB);
            n->name = name;
        }
        for (int i = 0; i < args->nkids; i++) node_add(n, args->kids[i]);
        free(args->kids);
        free(args);
        return n;
    }
    if (as_arg || kind_was == T_WORD || is_kw_name_kind(kind_was)) {
        Node *n = node_new(N_IDENT);
        n->name = name;
        return n;
    }
    Node *n = node_new(N_VERB);
    n->name = name;
    return n;
}

static Node *parse_primary(Parser *P, int as_arg) {
    int k = Pkind(P);
    if (k == T_KW_TRUE) {
        Peat(P);
        return node_new(N_TRUE);
    }
    if (k == T_KW_FALSE) {
        Peat(P);
        return node_new(N_FALSE);
    }
    if (k == T_INT) {
        Token t = Peat(P);
        Node *n = node_new(N_INT);
        n->ival = t.ival;
        return n;
    }
    if (k == T_FLOAT) {
        Token t = Peat(P);
        Node *n = node_new(N_FLOAT);
        n->fval = t.fval;
        return n;
    }
    if (k == T_STRING) {
        Token t = Peat(P);
        Node *n = node_new(N_STR);
        n->sval = xstrdup(t.sval ? t.sval : "");
        return n;
    }
    if (k == T_LBRACE) return parse_block(P);
    if (k == T_LPAREN) {
        Peat(P);
        Node *inner = looks_like_flow(P) ? parse_choir(P) : parse_expr(P);
        if (!Pexpect(P, T_RPAREN)) perr(P, "ожидалась )");
        return inner;
    }
    if (k == T_ROOT || k == T_TEMPLATE || k == T_WORD) {
        int kind_was = k;
        char *name = parse_ident_name(P);
        return finish_name_or_call(P, name, kind_was, as_arg);
    }
    if (is_kw_name_kind(k)) {
        int kind_was = k;
        char *name = parse_ident_name(P);
        return finish_name_or_call(P, name, kind_was, as_arg);
    }
    perr(P, "неожиданный токен в выражении");
    Peat(P);
    return node_new(N_INT);
}

static Node *parse_unary(Parser *P, int as_arg) {
    if (Pis(P, T_DASH)) {
        Peat(P);
        Node *n = node_new(N_NEG);
        node_add(n, parse_unary(P, as_arg));
        return n;
    }
    return parse_primary(P, as_arg);
}

static Node *parse_mul(Parser *P, int as_arg) {
    Node *left = parse_unary(P, as_arg);
    for (;;) {
        const char *op = NULL;
        if (Pis(P, T_STAR)) op = "*";
        else if (Pis(P, T_SLASH)) op = "/";
        else break;
        Peat(P);
        Node *n = node_new(N_BINOP);
        n->op = xstrdup(op);
        node_add(n, left);
        node_add(n, parse_unary(P, as_arg));
        left = n;
    }
    return left;
}

static Node *parse_add(Parser *P, int as_arg) {
    Node *left = parse_mul(P, as_arg);
    for (;;) {
        const char *op = NULL;
        if (Pis(P, T_PLUS)) op = "+";
        else if (Pis(P, T_DASH)) op = "-";
        else break;
        Peat(P);
        Node *n = node_new(N_BINOP);
        n->op = xstrdup(op);
        node_add(n, left);
        node_add(n, parse_mul(P, as_arg));
        left = n;
    }
    return left;
}

static Node *parse_cmp(Parser *P, int as_arg) {
    Node *left = parse_add(P, as_arg);
    const char *op = NULL;
    if (Pis(P, T_EQEQ)) op = "==";
    else if (Pis(P, T_NE)) op = "!=";
    else if (Pis(P, T_GT)) op = ">";
    else if (Pis(P, T_LT)) op = "<";
    else if (Pis(P, T_GE)) op = ">=";
    else if (Pis(P, T_LE)) op = "<=";
    if (!op) return left;
    Peat(P);
    Node *n = node_new(N_CMP);
    n->op = xstrdup(op);
    node_add(n, left);
    node_add(n, parse_add(P, as_arg));
    return n;
}

static Node *parse_not(Parser *P, int as_arg) {
    if (Pis(P, T_KW_NOT)) {
        Peat(P);
        Node *n = node_new(N_NOT);
        node_add(n, parse_not(P, as_arg));
        return n;
    }
    return parse_cmp(P, as_arg);
}

static Node *parse_and(Parser *P, int as_arg) {
    Node *left = parse_not(P, as_arg);
    while (Pis(P, T_KW_AND)) {
        Peat(P);
        Node *n = node_new(N_AND);
        node_add(n, left);
        node_add(n, parse_not(P, as_arg));
        left = n;
    }
    return left;
}

static Node *parse_or(Parser *P, int as_arg) {
    Node *left = parse_and(P, as_arg);
    while (Pis(P, T_KW_OR)) {
        Peat(P);
        Node *n = node_new(N_OR);
        node_add(n, left);
        node_add(n, parse_and(P, as_arg));
        left = n;
    }
    return left;
}

static Node *parse_expr(Parser *P) { return parse_or(P, 0); }
static Node *parse_expr_arg(Parser *P) { return parse_or(P, 1); }

static Node *parse_postfix_common(Parser *P, Node *node, int arg_mode) {
    if (Pis(P, T_FIX)) {
        Peat(P);
        const char *vn = verb_name(node);
        if (!vn) {
            perr(P, "! ожидает глагол");
            return node;
        }
        Node *fix = node_new(N_FIX);
        fix->name = xstrdup(vn);
        if (node->kind == N_VERB || node->kind == N_CALL) {
            /* данные = Блок(аргументы) — without next */
            Node *blk = node_new(N_BLOCK);
            int nargs = node->nkids - (node->has_next ? 1 : 0);
            for (int i = 0; i < nargs; i++) node_add(blk, node->kids[i]);
            /* leak old node shell but keep kids transferred */
            node->nkids = 0;
            free(node->kids);
            free(node->name);
            free(node);
            node_add(fix, blk);
        } else if (arg_mode && node->kind == N_IDENT) {
            free(node->name);
            free(node);
            node_add(fix, node_new(N_BLOCK));
        } else {
            node_add(fix, node);
        }
        return fix;
    }
    if (Pis(P, T_FLOW)) {
        Peat(P);
        const char *vn = verb_name(node);
        if (!vn) {
            perr(P, "~ ожидает глагол");
            return node;
        }
        Node *a = node_new(N_ASYNC);
        a->name = xstrdup(vn);
        /* free shell of verb/ident */
        if (node->kind == N_VERB || node->kind == N_CALL || node->kind == N_IDENT) {
            for (int i = 0; i < node->nkids; i++) {
                /* drop args of bare verb before ~ */
            }
            free(node->name);
            free(node->kids);
            free(node);
        }
        if (Pis(P, T_LBRACE))
            node_add(a, parse_block(P));
        else
            node_add(a, node_new(N_BLOCK));
        return a;
    }
    return node;
}

static Node *parse_postfix(Parser *P) {
    return parse_postfix_common(P, parse_expr(P), 0);
}

static Node *parse_postfix_arg(Parser *P) {
    return parse_postfix_common(P, parse_expr_arg(P), 1);
}

static Node *parse_arg(Parser *P) {
    Node *node = parse_postfix_arg(P);
    if (Pis(P, T_IMPULSE) || Pis(P, T_NET_IMPULSE)) {
        Peat(P);
        return attach_next(node, parse_impulse(P));
    }
    if (Pis(P, T_CHOIR)) {
        Node *ch = node_new(N_CHOIR);
        node_add(ch, node);
        while (Pis(P, T_CHOIR)) {
            Peat(P);
            node_add(ch, parse_impulse(P));
        }
        return ch;
    }
    return node;
}

static Node *parse_impulse(Parser *P) {
    Node *left = parse_postfix(P);
    if (Pis(P, T_IMPULSE) || Pis(P, T_NET_IMPULSE)) {
        Peat(P);
        return attach_next(left, parse_impulse(P));
    }
    return left;
}

static Node *parse_choir(Parser *P) {
    Node *first = parse_impulse(P);
    if (!Pis(P, T_CHOIR)) return first;
    Node *ch = node_new(N_CHOIR);
    node_add(ch, first);
    while (Pis(P, T_CHOIR)) {
        Peat(P);
        node_add(ch, parse_impulse(P));
    }
    return ch;
}

static Node *parse_block(Parser *P) {
    if (!Pexpect(P, T_LBRACE)) perr(P, "ожидалась {");
    Node *b = node_new(N_BLOCK);
    while (!Pis(P, T_RBRACE) && !Pis(P, T_EOF)) node_add(b, parse_stmt(P));
    if (!Pexpect(P, T_RBRACE)) perr(P, "ожидалась }");
    return b;
}

static Node *parse_block_or_stmt(Parser *P) {
    if (Pis(P, T_LBRACE)) return parse_block(P);
    return parse_stmt(P);
}

static Node *parse_root_decl(Parser *P) {
    Token t = Peat(P);
    Node *n = node_new(N_ROOT);
    n->name = xstrdup(t.sval ? t.sval : "");
    if (!Pexpect(P, T_EQ)) perr(P, "ожидался =");
    if (!Pexpect(P, T_LBRACE)) perr(P, "ожидалась {");
    while (!Pis(P, T_RBRACE) && !Pis(P, T_EOF)) {
        char *left = parse_ident_name(P);
        if (Pis(P, T_DASH)) {
            Peat(P);
            char *right = parse_ident_name(P);
            size_t ln = strlen(left) + 1 + strlen(right) + 1;
            char *comb = (char *)malloc(ln);
            snprintf(comb, ln, "%s-%s", left, right);
            free(left);
            free(right);
            left = comb;
        }
        if (!Pexpect(P, T_COLON)) perr(P, "ожидалось :");
        char *word = parse_ident_name(P);
        n->pair_a = (char **)realloc(n->pair_a, (size_t)(n->npairs + 1) * sizeof(char *));
        n->pair_b = (char **)realloc(n->pair_b, (size_t)(n->npairs + 1) * sizeof(char *));
        n->pair_a[n->npairs] = left;
        n->pair_b[n->npairs] = word;
        n->npairs++;
        if (Pis(P, T_COMMA)) Peat(P);
        else break;
    }
    if (!Pexpect(P, T_RBRACE)) perr(P, "ожидалась }");
    return n;
}

static Node *parse_if(Parser *P) {
    Peat(P); /* если */
    Node *n = node_new(N_IF);
    node_add(n, parse_expr(P));
    node_add(n, parse_block_or_stmt(P));
    if (Pis(P, T_KW_ELSE)) {
        Peat(P);
        node_add(n, parse_block_or_stmt(P));
        n->has_else = 1;
    }
    return n;
}

static Node *parse_while(Parser *P) {
    Peat(P);
    Node *n = node_new(N_WHILE);
    node_add(n, parse_expr(P));
    node_add(n, parse_block_or_stmt(P));
    return n;
}

static Node *parse_for(Parser *P) {
    Peat(P);
    Node *n = node_new(N_FOR);
    n->name = parse_ident_name(P);
    if (!Pexpect(P, T_KW_IN)) perr(P, "ожидалось в");
    node_add(n, parse_expr(P));
    node_add(n, parse_block_or_stmt(P));
    return n;
}

static Node *parse_fn(Parser *P) {
    Peat(P);
    Node *n = node_new(N_FN);
    n->name = parse_ident_name(P);
    if (!Pexpect(P, T_LPAREN)) perr(P, "ожидалась (");
    if (!Pis(P, T_RPAREN)) {
        for (;;) {
            char *pn = parse_ident_name(P);
            char *pt = xstrdup("");
            if (Pis(P, T_COLON)) {
                Peat(P);
                free(pt);
                pt = parse_ident_name(P);
            }
            n->pnames = (char **)realloc(n->pnames, (size_t)(n->nparams + 1) * sizeof(char *));
            n->ptypes = (char **)realloc(n->ptypes, (size_t)(n->nparams + 1) * sizeof(char *));
            n->pnames[n->nparams] = pn;
            n->ptypes[n->nparams] = pt;
            n->nparams++;
            if (Pis(P, T_COMMA)) {
                Peat(P);
                continue;
            }
            break;
        }
    }
    if (!Pexpect(P, T_RPAREN)) perr(P, "ожидалась )");
    if (Pis(P, T_COLON)) {
        Peat(P);
        n->ret_type = parse_ident_name(P);
        n->has_ret_type = 1;
    }
    node_add(n, parse_block(P));
    return n;
}

static Node *parse_return(Parser *P) {
    Peat(P);
    Node *n = node_new(N_RETURN);
    if (Pis(P, T_LPAREN)) {
        Peat(P);
        if (Pis(P, T_RPAREN)) {
            Peat(P);
            return n;
        }
        node_add(n, parse_expr(P));
        n->has_return_val = 1;
        if (!Pexpect(P, T_RPAREN)) perr(P, "ожидалась )");
        return n;
    }
    if (Pis(P, T_RBRACE) || Pis(P, T_EOF) || Pis(P, T_KW_ELSE) || Pis(P, T_KW_BREAK) ||
        Pis(P, T_KW_CONTINUE))
        return n;
    if (!(Pis(P, T_KW_IF) || Pis(P, T_KW_WHILE) || Pis(P, T_KW_FOR) || Pis(P, T_KW_FN) ||
          Pis(P, T_KW_RETURN))) {
        node_add(n, parse_expr(P));
        n->has_return_val = 1;
    }
    return n;
}

static Node *parse_runtime(Parser *P) {
    Peat(P); /* #runtime */
    if (!Pexpect(P, T_LPAREN)) perr(P, "ожидалась (");
    if (!Pis(P, T_STRING)) {
        perr(P, "ожидалась строка");
        return node_new(N_RUNTIME);
    }
    Token t = Peat(P);
    Node *n = node_new(N_RUNTIME);
    n->name = xstrdup(t.sval ? t.sval : "");
    if (!Pexpect(P, T_RPAREN)) perr(P, "ожидалась )");
    return n;
}

static Node *parse_link(Parser *P) {
    Peat(P);
    if (!Pexpect(P, T_LPAREN)) perr(P, "ожидалась (");
    if (!Pis(P, T_STRING)) {
        perr(P, "ожидалась строка");
        return node_new(N_EXT);
    }
    Token t = Peat(P);
    Node *n = node_new(N_EXT);
    n->name2 = xstrdup(t.sval ? t.sval : ""); /* lib */
    if (!Pexpect(P, T_RPAREN)) perr(P, "ожидалась )");
    n->name = parse_ident_name(P);
    if (!Pexpect(P, T_LPAREN)) perr(P, "ожидалась (");
    Node *args = parse_arg_list(P, n->name);
    for (int i = 0; i < args->nkids; i++) node_add(n, args->kids[i]);
    free(args->kids);
    free(args);
    return n;
}

static Node *parse_assign_or_flow(Parser *P) {
    if ((Pis(P, T_WORD) || Pis(P, T_ROOT) || Pis(P, T_TEMPLATE)) && Plook(P, 1)->kind == T_EQ) {
        char *name = parse_ident_name(P);
        Peat(P); /* = */
        Node *n = node_new(N_ASSIGN);
        n->name = name;
        node_add(n, parse_expr(P));
        return n;
    }
    return parse_choir(P);
}

static Node *parse_stmt(Parser *P) {
    int k = Pkind(P);
    if (k == T_DIRECTIVE && Pcur(P)->sval && strcmp(Pcur(P)->sval, "link") == 0)
        return parse_link(P);
    if (k == T_DIRECTIVE && Pcur(P)->sval && strcmp(Pcur(P)->sval, "runtime") == 0)
        return parse_runtime(P);
    if (k == T_ROOT && Plook(P, 1)->kind == T_EQ) return parse_root_decl(P);
    if (k == T_KW_IF) return parse_if(P);
    if (k == T_KW_WHILE) return parse_while(P);
    if (k == T_KW_FOR) return parse_for(P);
    if (k == T_KW_FN) return parse_fn(P);
    if (k == T_KW_RETURN) return parse_return(P);
    if (k == T_KW_BREAK) {
        Peat(P);
        return node_new(N_BREAK);
    }
    if (k == T_KW_CONTINUE) {
        Peat(P);
        return node_new(N_CONTINUE);
    }
    return parse_assign_or_flow(P);
}

static Node *parse_program(Parser *P) {
    Node *prog = node_new(N_PROG);
    while (!Pis(P, T_EOF)) node_add(prog, parse_stmt(P));
    return prog;
}

/* ===== Debug print (Rust {:#?}) ===== */
static void pad(int d) {
    for (int i = 0; i < d; i++) putchar(' ');
}

static void print_node(Node *n, int ind);

static void print_list(Node **arr, int n, int ind, int from, int to) {
    (void)n;
    if (from >= to) {
        printf("[]");
        return;
    }
    printf("[\n");
    for (int i = from; i < to; i++) {
        print_node(arr[i], ind + 4);
        printf(",\n");
    }
    pad(ind);
    printf("]");
}

static void print_node(Node *n, int ind) {
    if (!n) {
        pad(ind);
        printf("None");
        return;
    }
    switch (n->kind) {
        case N_PROG:
            pad(ind);
            printf("Программа {\n");
            pad(ind + 4);
            printf("объявления: ");
            print_list(n->kids, n->nkids, ind + 4, 0, n->nkids);
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_ROOT:
            pad(ind);
            printf("ОбъявлениеКорня {\n");
            pad(ind + 4);
            printf("корень: \"%s\",\n", n->name ? n->name : "");
            pad(ind + 4);
            printf("шаблоны: [\n");
            for (int i = 0; i < n->npairs; i++) {
                pad(ind + 8);
                printf("(\n");
                pad(ind + 12);
                printf("\"%s\",\n", n->pair_a[i]);
                pad(ind + 12);
                printf("\"%s\",\n", n->pair_b[i]);
                pad(ind + 8);
                printf("),\n");
            }
            pad(ind + 4);
            printf("],\n");
            pad(ind);
            printf("}");
            break;
        case N_VERB: {
            int nargs = n->nkids - (n->has_next ? 1 : 0);
            pad(ind);
            printf("ВызовГлагола {\n");
            pad(ind + 4);
            printf("глагол: \"%s\",\n", n->name ? n->name : "");
            pad(ind + 4);
            printf("аргументы: ");
            print_list(n->kids, n->nkids, ind + 4, 0, nargs);
            printf(",\n");
            pad(ind + 4);
            printf("следующий: ");
            if (n->has_next) {
                printf("Some(\n");
                print_node(n->kids[nargs], ind + 8);
                printf(",\n");
                pad(ind + 4);
                printf(")");
            } else {
                printf("None");
            }
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        }
        case N_FIX:
            pad(ind);
            printf("Фиксация {\n");
            pad(ind + 4);
            printf("глагол: \"%s\",\n", n->name ? n->name : "");
            pad(ind + 4);
            printf("данные: ");
            if (n->nkids > 0) {
                /* inline first line of child without extra pad for tuple-like */
                /* Rust: данные: Блок( — child printed at same indent level as field value */
                /* Actually {:#?} prints nested with indent. Field value starts after ": " */
                /* For Box, it prints the inner directly */
                Node *d = n->kids[0];
                if (d->kind == N_BLOCK) {
                    printf("Блок(\n");
                    pad(ind + 8);
                    if (d->nkids == 0)
                        printf("[],\n");
                    else {
                        printf("[\n");
                        for (int i = 0; i < d->nkids; i++) {
                            print_node(d->kids[i], ind + 12);
                            printf(",\n");
                        }
                        pad(ind + 8);
                        printf("],\n");
                    }
                    pad(ind + 4);
                    printf(")");
                } else {
                    printf("\n");
                    print_node(d, ind + 8);
                }
            }
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_ASYNC:
            pad(ind);
            printf("АсинхронныйПоток {\n");
            pad(ind + 4);
            printf("глагол: \"%s\",\n", n->name ? n->name : "");
            pad(ind + 4);
            printf("тело: ");
            if (n->nkids > 0 && n->kids[0]->kind == N_BLOCK) {
                Node *d = n->kids[0];
                printf("Блок(\n");
                pad(ind + 8);
                if (d->nkids == 0)
                    printf("[],\n");
                else {
                    printf("[\n");
                    for (int i = 0; i < d->nkids; i++) {
                        print_node(d->kids[i], ind + 12);
                        printf(",\n");
                    }
                    pad(ind + 8);
                    printf("],\n");
                }
                pad(ind + 4);
                printf(")");
            } else if (n->nkids > 0) {
                printf("\n");
                print_node(n->kids[0], ind + 8);
            }
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_CHOIR:
            pad(ind);
            printf("Хор {\n");
            pad(ind + 4);
            printf("ветки: ");
            print_list(n->kids, n->nkids, ind + 4, 0, n->nkids);
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_INT:
            pad(ind);
            printf("Целое(\n");
            pad(ind + 4);
            printf("%lld,\n", (long long)n->ival);
            pad(ind);
            printf(")");
            break;
        case N_FLOAT:
            pad(ind);
            printf("Вещественное(\n");
            pad(ind + 4);
            printf("%g,\n", n->fval);
            pad(ind);
            printf(")");
            break;
        case N_STR:
            pad(ind);
            printf("Строка(\n");
            pad(ind + 4);
            printf("\"%s\",\n", n->sval ? n->sval : "");
            pad(ind);
            printf(")");
            break;
        case N_IDENT:
            pad(ind);
            printf("Идентификатор(\n");
            pad(ind + 4);
            printf("\"%s\",\n", n->name ? n->name : "");
            pad(ind);
            printf(")");
            break;
        case N_BLOCK:
            pad(ind);
            printf("Блок(\n");
            pad(ind + 4);
            if (n->nkids == 0)
                printf("[],\n");
            else {
                printf("[\n");
                for (int i = 0; i < n->nkids; i++) {
                    print_node(n->kids[i], ind + 8);
                    printf(",\n");
                }
                pad(ind + 4);
                printf("],\n");
            }
            pad(ind);
            printf(")");
            break;
        case N_IF:
            pad(ind);
            printf("Если {\n");
            pad(ind + 4);
            printf("условие: ");
            /* Box prints as nested */
            printf("\n");
            print_node(n->kids[0], ind + 8);
            printf(",\n");
            pad(ind + 4);
            printf("тогда: \n");
            print_node(n->kids[1], ind + 8);
            printf(",\n");
            pad(ind + 4);
            printf("иначе: ");
            if (n->has_else) {
                printf("Some(\n");
                print_node(n->kids[2], ind + 8);
                printf(",\n");
                pad(ind + 4);
                printf(")");
            } else
                printf("None");
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_WHILE:
            pad(ind);
            printf("Пока {\n");
            pad(ind + 4);
            printf("условие: \n");
            print_node(n->kids[0], ind + 8);
            printf(",\n");
            pad(ind + 4);
            printf("тело: \n");
            print_node(n->kids[1], ind + 8);
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_FOR:
            pad(ind);
            printf("Для {\n");
            pad(ind + 4);
            printf("переменная: \"%s\",\n", n->name ? n->name : "");
            pad(ind + 4);
            printf("коллекция: \n");
            print_node(n->kids[0], ind + 8);
            printf(",\n");
            pad(ind + 4);
            printf("тело: \n");
            print_node(n->kids[1], ind + 8);
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_FN:
            pad(ind);
            printf("ОпределениеФункции {\n");
            pad(ind + 4);
            printf("имя: \"%s\",\n", n->name ? n->name : "");
            pad(ind + 4);
            printf("параметры: [\n");
            for (int i = 0; i < n->nparams; i++) {
                pad(ind + 8);
                printf("(\n");
                pad(ind + 12);
                printf("\"%s\",\n", n->pnames[i]);
                pad(ind + 12);
                printf("\"%s\",\n", n->ptypes[i]);
                pad(ind + 8);
                printf("),\n");
            }
            pad(ind + 4);
            printf("],\n");
            pad(ind + 4);
            printf("тип_возврата: ");
            if (n->has_ret_type) {
                printf("Some(\n");
                pad(ind + 8);
                printf("\"%s\",\n", n->ret_type);
                pad(ind + 4);
                printf(")");
            } else
                printf("None");
            printf(",\n");
            pad(ind + 4);
            printf("тело: \n");
            print_node(n->kids[0], ind + 8);
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_RETURN:
            pad(ind);
            printf("Возврат(");
            if (n->has_return_val) {
                printf("\n");
                pad(ind + 4);
                printf("Some(\n");
                print_node(n->kids[0], ind + 8);
                printf(",\n");
                pad(ind + 4);
                printf("),\n");
                pad(ind);
                printf(")");
            } else {
                printf("\n");
                pad(ind + 4);
                printf("None,\n");
                pad(ind);
                printf(")");
            }
            break;
        case N_BREAK:
            pad(ind);
            printf("Прервать");
            break;
        case N_CONTINUE:
            pad(ind);
            printf("Продолжить");
            break;
        case N_BINOP:
            pad(ind);
            printf("БинарнаяОперация {\n");
            pad(ind + 4);
            printf("оператор: \"%s\",\n", n->op ? n->op : "");
            pad(ind + 4);
            printf("левый: \n");
            print_node(n->kids[0], ind + 8);
            printf(",\n");
            pad(ind + 4);
            printf("правый: \n");
            print_node(n->kids[1], ind + 8);
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_CMP:
            pad(ind);
            printf("Сравнение {\n");
            pad(ind + 4);
            printf("оператор: \"%s\",\n", n->op ? n->op : "");
            pad(ind + 4);
            printf("левый: \n");
            print_node(n->kids[0], ind + 8);
            printf(",\n");
            pad(ind + 4);
            printf("правый: \n");
            print_node(n->kids[1], ind + 8);
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_ASSIGN:
            pad(ind);
            printf("Присваивание {\n");
            pad(ind + 4);
            printf("имя: \"%s\",\n", n->name ? n->name : "");
            pad(ind + 4);
            printf("значение: \n");
            print_node(n->kids[0], ind + 8);
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_CALL:
            pad(ind);
            printf("ВызовФункции {\n");
            pad(ind + 4);
            printf("имя: \"%s\",\n", n->name ? n->name : "");
            pad(ind + 4);
            printf("аргументы: ");
            print_list(n->kids, n->nkids, ind + 4, 0, n->nkids);
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_TRUE:
            pad(ind);
            printf("Истина");
            break;
        case N_FALSE:
            pad(ind);
            printf("Ложь");
            break;
        case N_AND:
            pad(ind);
            printf("ЛогическоеИ(\n");
            print_node(n->kids[0], ind + 4);
            printf(",\n");
            print_node(n->kids[1], ind + 4);
            printf(",\n");
            pad(ind);
            printf(")");
            break;
        case N_OR:
            pad(ind);
            printf("ЛогическоеИли(\n");
            print_node(n->kids[0], ind + 4);
            printf(",\n");
            print_node(n->kids[1], ind + 4);
            printf(",\n");
            pad(ind);
            printf(")");
            break;
        case N_NOT:
            pad(ind);
            printf("ЛогическоеНе(\n");
            print_node(n->kids[0], ind + 4);
            printf(",\n");
            pad(ind);
            printf(")");
            break;
        case N_NEG:
            pad(ind);
            printf("УнарныйМинус(\n");
            print_node(n->kids[0], ind + 4);
            printf(",\n");
            pad(ind);
            printf(")");
            break;
        case N_EXT:
            pad(ind);
            printf("ВнешнийВызов {\n");
            pad(ind + 4);
            printf("библиотека: \"%s\",\n", n->name2 ? n->name2 : "");
            pad(ind + 4);
            printf("функция: \"%s\",\n", n->name ? n->name : "");
            pad(ind + 4);
            printf("аргументы: ");
            print_list(n->kids, n->nkids, ind + 4, 0, n->nkids);
            printf(",\n");
            pad(ind);
            printf("}");
            break;
        case N_RUNTIME:
            pad(ind);
            printf("Runtime {\n");
            pad(ind + 4);
            printf("имя: \"%s\",\n", n->name ? n->name : "");
            pad(ind);
            printf("}");
            break;
        default:
            pad(ind);
            printf("?");
            break;
    }
}

#ifdef ZIP_PARSER_MAIN
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif
int main(int argc, char **argv) {
#ifdef _WIN32
    _setmode(_fileno(stdout), _O_BINARY);
    _setmode(_fileno(stderr), _O_BINARY);
#endif
    if (argc < 2) {
        fprintf(stderr, "Использование: %s <файл.zip>\n", argv[0]);
        return 1;
    }
    const char *path = argv[1];
    FILE *f = fopen(path, "rb");
    if (!f) {
        perror(path);
        return 1;
    }
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    unsigned char *buf = (unsigned char *)malloc((size_t)sz + 1);
    size_t n = fread(buf, 1, (size_t)sz, f);
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

    Parser P = {0};
    P.toks = L.toks;
    P.ntoks = L.ntoks;
    P.pos = 0;
    Node *ast = parse_program(&P);
    print_node(ast, 0);
    printf("\n");

    for (int i = 0; i < L.ntoks; i++) free(L.toks[i].sval);
    free(L.toks);
    free(buf);
    return 0;
}
#endif
'''

(SELF / "парсер_rt.c").write_text(c, encoding="utf-8", newline="\n")
# ASCII alias path for tools that struggle with Cyrillic
(SELF / "parser_rt.c").write_text(c, encoding="utf-8", newline="\n")
print("парсер_rt.c", len(c), "bytes")
