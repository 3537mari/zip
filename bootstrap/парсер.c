#include "zipb.h"

typedef struct {
    Token *toks;
    size_t n;
    size_t i;
} Parser;

static Token *peek(Parser *P) {
    return P->i < P->n ? &P->toks[P->i] : NULL;
}

static Token *advance(Parser *P) {
    Token *t = peek(P);
    if (P->i < P->n) P->i++;
    return t;
}

static int check(Parser *P, TokenType ty) {
    Token *t = peek(P);
    return t && t->type == ty;
}

static int match(Parser *P, TokenType ty) {
    if (check(P, ty)) { advance(P); return 1; }
    return 0;
}

static Node *parse_expr(Parser *P);
static Node *parse_stmt(Parser *P);
static Node *parse_block(Parser *P);

static Node *parse_primary(Parser *P) {
    Token *t = peek(P);
    if (!t) return NULL;
    if (t->type == TK_INT) {
        Node *n = node_new(NODE_CONST_I64);
        n->i64 = t->i64;
        advance(P);
        return n;
    }
    if (t->type == TK_FLOAT) {
        Node *n = node_new(NODE_CONST_F64);
        n->f64 = t->f64;
        advance(P);
        return n;
    }
    if (t->type == TK_STRING) {
        Node *n = node_new(NODE_CONST_STR);
        n->str = zip_strdup(t->str);
        advance(P);
        return n;
    }
    if (t->type == TK_TRUE || t->type == TK_FALSE) {
        Node *n = node_new(NODE_CONST_I64);
        n->i64 = t->type == TK_TRUE ? 1 : 0;
        advance(P);
        return n;
    }
    if (t->type == TK_IDENT || t->type == TK_ROOT) {
        char *name = zip_strdup(t->str);
        advance(P);
        if (match(P, TK_LPAREN)) {
            Node *call = node_new(NODE_CALL);
            call->name = name;
            if (!check(P, TK_RPAREN)) {
                do {
                    Node *arg = parse_expr(P);
                    if (arg) node_add(call, arg);
                } while (match(P, TK_COMMA));
            }
            match(P, TK_RPAREN);
            /* builtins */
            if (!strcmp(name, "ПечатьСтроки") || !strcmp(name, "печать_строки")) {
                call->type = NODE_PRINT_STR;
            } else if (!strcmp(name, "ПечатьЧисло") || !strcmp(name, "печать_число")) {
                call->type = NODE_PRINT_I64;
            }
            return call;
        }
        Node *id = node_new(NODE_IDENT);
        id->name = name;
        return id;
    }
    if (match(P, TK_LPAREN)) {
        Node *e = parse_expr(P);
        match(P, TK_RPAREN);
        return e;
    }
    advance(P);
    return NULL;
}

static Node *parse_unary(Parser *P) {
    if (match(P, TK_MINUS)) {
        Node *z = node_new(NODE_CONST_I64);
        z->i64 = 0;
        Node *r = parse_unary(P);
        Node *b = node_new(NODE_BINOP);
        b->op = TK_MINUS;
        node_add(b, z);
        if (r) node_add(b, r);
        return b;
    }
    if (match(P, TK_NOT)) return parse_unary(P);
    return parse_primary(P);
}

static Node *parse_mul(Parser *P) {
    Node *left = parse_unary(P);
    while (check(P, TK_STAR) || check(P, TK_SLASH) || check(P, TK_PERCENT)) {
        TokenType op = peek(P)->type;
        advance(P);
        Node *right = parse_unary(P);
        Node *b = node_new(NODE_BINOP);
        b->op = (int)op;
        if (left) node_add(b, left);
        if (right) node_add(b, right);
        left = b;
    }
    return left;
}

static Node *parse_add(Parser *P) {
    Node *left = parse_mul(P);
    while (check(P, TK_PLUS) || check(P, TK_MINUS)) {
        TokenType op = peek(P)->type;
        advance(P);
        Node *right = parse_mul(P);
        Node *b = node_new(NODE_BINOP);
        b->op = (int)op;
        if (left) node_add(b, left);
        if (right) node_add(b, right);
        left = b;
    }
    return left;
}

static Node *parse_cmp(Parser *P) {
    Node *left = parse_add(P);
    while (check(P, TK_LT) || check(P, TK_LE) || check(P, TK_GT) || check(P, TK_GE)
           || check(P, TK_EQEQ) || check(P, TK_NE)) {
        TokenType op = peek(P)->type;
        advance(P);
        Node *right = parse_add(P);
        Node *b = node_new(NODE_BINOP);
        b->op = (int)op;
        if (left) node_add(b, left);
        if (right) node_add(b, right);
        left = b;
    }
    return left;
}

static Node *parse_expr(Parser *P) {
    return parse_cmp(P);
}

static Node *parse_block(Parser *P) {
    Node *block = node_new(NODE_BLOCK);
    match(P, TK_LBRACE);
    while (peek(P) && !check(P, TK_RBRACE) && !check(P, TK_EOF)) {
        Node *s = parse_stmt(P);
        if (s) node_add(block, s);
        else advance(P);
    }
    match(P, TK_RBRACE);
    return block;
}

static Node *parse_stmt(Parser *P) {
    if (check(P, TK_RETURN)) {
        advance(P);
        Node *n = node_new(NODE_RETURN);
        if (!check(P, TK_RBRACE) && !check(P, TK_EOF)) {
            Node *e = parse_expr(P);
            if (e) node_add(n, e);
        }
        return n;
    }
    if (check(P, TK_IF)) {
        advance(P);
        Node *n = node_new(NODE_IF);
        Node *cond = parse_expr(P);
        if (cond) node_add(n, cond);
        Node *thenb = parse_block(P);
        if (thenb) node_add(n, thenb);
        if (match(P, TK_ELSE)) {
            Node *elseb = parse_block(P);
            if (elseb) node_add(n, elseb);
        }
        return n;
    }
    if (check(P, TK_WHILE)) {
        advance(P);
        Node *n = node_new(NODE_WHILE);
        Node *cond = parse_expr(P);
        if (cond) node_add(n, cond);
        Node *body = parse_block(P);
        if (body) node_add(n, body);
        return n;
    }
    if ((check(P, TK_IDENT) || check(P, TK_ROOT)) && P->i + 1 < P->n
        && P->toks[P->i + 1].type == TK_EQ) {
        Node *n = node_new(NODE_ASSIGN);
        n->name = zip_strdup(peek(P)->str);
        advance(P);
        advance(P); /* = */
        Node *e = parse_expr(P);
        if (e) node_add(n, e);
        return n;
    }
    /* expression statement / call */
    return parse_expr(P);
}

static Node *parse_function(Parser *P) {
    advance(P); /* функция */
    Node *fn = node_new(NODE_FUNCTION);
    if (check(P, TK_IDENT) || check(P, TK_ROOT)) {
        fn->name = zip_strdup(peek(P)->str);
        advance(P);
    } else {
        fn->name = zip_strdup("главная");
    }
    if (match(P, TK_LPAREN)) {
        while (!check(P, TK_RPAREN) && !check(P, TK_EOF)) advance(P);
        match(P, TK_RPAREN);
    }
    Node *body = parse_block(P);
    if (body) node_add(fn, body);
    return fn;
}

Node *parse(Token *toks, size_t n) {
    Parser P = { toks, n, 0 };
    Node *prog = node_new(NODE_PROGRAM);
    Node *main_body = node_new(NODE_BLOCK);
    int has_fn = 0;

    while (peek(&P) && !check(&P, TK_EOF)) {
        if (check(&P, TK_AT) || check(&P, TK_HASH)) {
            /* skip annotation / directive line-ish */
            advance(&P);
            while (peek(&P) && !check(&P, TK_FUNCTION) && !check(&P, TK_EOF)
                   && peek(&P)->type != TK_IDENT && peek(&P)->type != TK_ROOT
                   && peek(&P)->type != TK_IF && peek(&P)->type != TK_WHILE) {
                if (check(&P, TK_LBRACE)) { parse_block(&P); break; }
                advance(&P);
            }
            continue;
        }
        if (check(&P, TK_FUNCTION)) {
            Node *fn = parse_function(&P);
            if (fn) { node_add(prog, fn); has_fn = 1; }
            continue;
        }
        Node *s = parse_stmt(&P);
        if (s) node_add(main_body, s);
        else advance(&P);
    }

    if (!has_fn && main_body->n > 0) {
        Node *fn = node_new(NODE_FUNCTION);
        fn->name = zip_strdup("главная");
        node_add(fn, main_body);
        node_add(prog, fn);
    } else if (main_body->n == 0) {
        node_free(main_body);
    } else {
        /* top-level stmts + functions: wrap leftovers into главная */
        Node *fn = node_new(NODE_FUNCTION);
        fn->name = zip_strdup("главная");
        node_add(fn, main_body);
        node_add(prog, fn);
    }
    return prog;
}
