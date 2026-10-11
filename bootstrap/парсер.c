#include "zipb.h"

typedef struct {
    Token *toks;
    size_t n, i;
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
static Node *parse_function(Parser *P);

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
        Node *n = node_new(NODE_CONST_BOOL);
        n->i64 = t->type == TK_TRUE ? 1 : 0;
        advance(P);
        return n;
    }
    if (t->type == TK_NOTHING) {
        Node *n = node_new(NODE_CONST_I64);
        n->i64 = 0;
        n->name = zip_strdup("ничто");
        advance(P);
        return n;
    }
    if (t->type == TK_IDENT || t->type == TK_ROOT || t->type == TK_TEMPLATE
        || ((t->type == TK_AND || t->type == TK_OR || t->type == TK_NOT || t->type == TK_IN)
            && t->str)) {
        char *name = zip_strdup(t->str);
        advance(P);
        if (match(P, TK_LPAREN)) {
            Node *call = node_new(NODE_CALL);
            call->name = name;
            if (!check(P, TK_RPAREN)) {
                do {
                    Node *a = parse_expr(P);
                    if (a) node_add(call, a);
                } while (match(P, TK_COMMA));
            }
            match(P, TK_RPAREN);
            call->arity = call->n;
            if (zip_streq(name, "ПечатьСтроки") || zip_streq(name, "печать_строки"))
                call->type = NODE_PRINT_STR;
            else if (zip_streq(name, "ПечатьЧисло") || zip_streq(name, "печать_число"))
                call->type = NODE_PRINT_I64;
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
    return NULL;
}

static Node *parse_unary(Parser *P) {
    if (match(P, TK_MINUS)) {
        Node *u = node_new(NODE_UNARY);
        u->op = TK_MINUS;
        Node *r = parse_unary(P);
        if (r) node_add(u, r);
        return u;
    }
    if (match(P, TK_NOT)) {
        Node *u = node_new(NODE_UNARY);
        u->op = TK_NOT;
        Node *r = parse_unary(P);
        if (r) node_add(u, r);
        return u;
    }
    if (match(P, TK_PERFECT)) {
        Node *u = node_new(NODE_PERFECT);
        Node *r = parse_unary(P);
        if (r) node_add(u, r);
        return u;
    }
    return parse_primary(P);
}

static Node *parse_mul(Parser *P) {
    Node *left = parse_unary(P);
    while (check(P, TK_STAR) || check(P, TK_SLASH) || check(P, TK_PERCENT)) {
        int op = (int)peek(P)->type;
        advance(P);
        Node *right = parse_unary(P);
        Node *b = node_new(NODE_BINOP);
        b->op = op;
        if (left) node_add(b, left);
        if (right) node_add(b, right);
        left = b;
    }
    return left;
}

static Node *parse_add(Parser *P) {
    Node *left = parse_mul(P);
    while (check(P, TK_PLUS) || check(P, TK_MINUS)) {
        int op = (int)peek(P)->type;
        advance(P);
        Node *right = parse_mul(P);
        Node *b = node_new(NODE_BINOP);
        b->op = op;
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
        int op = (int)peek(P)->type;
        advance(P);
        Node *right = parse_add(P);
        Node *b = node_new(NODE_BINOP);
        b->op = op;
        if (left) node_add(b, left);
        if (right) node_add(b, right);
        left = b;
    }
    return left;
}

static Node *parse_and(Parser *P) {
    Node *left = parse_cmp(P);
    while (match(P, TK_AND)) {
        Node *right = parse_cmp(P);
        Node *b = node_new(NODE_BINOP);
        b->op = TK_AND;
        if (left) node_add(b, left);
        if (right) node_add(b, right);
        left = b;
    }
    return left;
}

static Node *parse_or(Parser *P) {
    Node *left = parse_and(P);
    while (match(P, TK_OR)) {
        Node *right = parse_and(P);
        Node *b = node_new(NODE_BINOP);
        b->op = TK_OR;
        if (left) node_add(b, left);
        if (right) node_add(b, right);
        left = b;
    }
    return left;
}

/* >> impulse */
static Node *parse_pulse(Parser *P) {
    Node *left = parse_or(P);
    while (match(P, TK_IMPULSE) || match(P, TK_NET)) {
        Node *right = parse_or(P);
        Node *b = node_new(NODE_PULSE);
        if (left) node_add(b, left);
        if (right) node_add(b, right);
        left = b;
    }
    return left;
}

/* || choir */
static Node *parse_choir(Parser *P) {
    Node *left = parse_pulse(P);
    while (match(P, TK_CHOIR)) {
        Node *right = parse_pulse(P);
        Node *b = node_new(NODE_CHOIR);
        if (left) node_add(b, left);
        if (right) node_add(b, right);
        left = b;
    }
    return left;
}

static Node *parse_expr(Parser *P) {
    return parse_choir(P);
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

static Node *parse_if(Parser *P) {
    advance(P); /* если */
    Node *n = node_new(NODE_IF);
    if (match(P, TK_LPAREN)) {
        Node *c = parse_expr(P);
        if (c) node_add(n, c);
        match(P, TK_RPAREN);
    } else {
        Node *c = parse_expr(P);
        if (c) node_add(n, c);
    }
    Node *thenb = check(P, TK_LBRACE) ? parse_block(P) : parse_stmt(P);
    if (thenb) node_add(n, thenb);
    if (match(P, TK_ELSE)) {
        Node *elseb = check(P, TK_LBRACE) ? parse_block(P) : parse_stmt(P);
        if (elseb) node_add(n, elseb);
    }
    return n;
}

static Node *parse_while(Parser *P) {
    advance(P);
    Node *n = node_new(NODE_WHILE);
    if (match(P, TK_LPAREN)) {
        Node *c = parse_expr(P);
        if (c) node_add(n, c);
        match(P, TK_RPAREN);
    } else {
        Node *c = parse_expr(P);
        if (c) node_add(n, c);
    }
    Node *body = check(P, TK_LBRACE) ? parse_block(P) : parse_stmt(P);
    if (body) node_add(n, body);
    return n;
}

static Node *parse_for(Parser *P) {
    advance(P); /* для */
    Node *n = node_new(NODE_FOR);
    /* «и» — часто счётчик цикла, но лексер даёт TK_AND */
    if (check(P, TK_IDENT) || check(P, TK_ROOT) || check(P, TK_TEMPLATE)
        || check(P, TK_AND) || check(P, TK_OR) || check(P, TK_NOT) || check(P, TK_IN)) {
        Token *tv = peek(P);
        n->name = zip_strdup(tv->str ? tv->str : "и");
        advance(P);
    }
    match(P, TK_IN);
    /* range: a .. b */
    Node *lo = parse_expr(P);
    if (lo) node_add(n, lo);
    match(P, TK_RANGE);
    Node *hi = parse_expr(P);
    if (hi) node_add(n, hi);
    Node *body = check(P, TK_LBRACE) ? parse_block(P) : parse_stmt(P);
    if (body) node_add(n, body);
    return n;
}

static Node *parse_stmt(Parser *P) {
    if (check(P, TK_RETURN)) {
        advance(P);
        Node *n = node_new(NODE_RETURN);
        if (!check(P, TK_RBRACE) && !check(P, TK_EOF) && !check(P, TK_ELSE)) {
            if (match(P, TK_LPAREN)) {
                Node *e = parse_expr(P);
                if (e) node_add(n, e);
                match(P, TK_RPAREN);
            } else {
                Node *e = parse_expr(P);
                if (e) node_add(n, e);
            }
        }
        return n;
    }
    if (check(P, TK_BREAK)) { advance(P); return node_new(NODE_BREAK); }
    if (check(P, TK_CONTINUE)) { advance(P); return node_new(NODE_CONTINUE); }
    if (check(P, TK_IF)) return parse_if(P);
    if (check(P, TK_WHILE)) return parse_while(P);
    if (check(P, TK_FOR)) return parse_for(P);

    if ((check(P, TK_IDENT) || check(P, TK_ROOT) || check(P, TK_TEMPLATE)
         || check(P, TK_AND) || check(P, TK_OR) || check(P, TK_NOT) || check(P, TK_IN))
        && P->i + 1 < P->n && P->toks[P->i + 1].type == TK_EQ) {
        Node *n = node_new(NODE_ASSIGN);
        n->name = zip_strdup(peek(P)->str ? peek(P)->str : "_");
        advance(P);
        advance(P);
        Node *e = parse_expr(P);
        if (e) node_add(n, e);
        return n;
    }
    return parse_expr(P);
}

static Node *parse_function(Parser *P) {
    advance(P); /* функция */
    Node *fn = node_new(NODE_FUNCTION);
    if (check(P, TK_IDENT) || check(P, TK_ROOT) || check(P, TK_TEMPLATE)) {
        fn->name = zip_strdup(peek(P)->str);
        advance(P);
    } else {
        fn->name = zip_strdup("главная");
    }
    if (match(P, TK_LPAREN)) {
        while (!check(P, TK_RPAREN) && !check(P, TK_EOF)) {
            if (check(P, TK_IDENT) || check(P, TK_ROOT) || check(P, TK_TEMPLATE)
                || check(P, TK_AND) || check(P, TK_OR)) {
                Node *p = node_new(NODE_PARAM);
                p->name = zip_strdup(peek(P)->str ? peek(P)->str : "_");
                advance(P);
                if (match(P, TK_COLON)) {
                    if (check(P, TK_IDENT) || check(P, TK_ROOT) || check(P, TK_TEMPLATE)) {
                        p->type_name = zip_strdup(peek(P)->str);
                        advance(P);
                    }
                }
                node_add(fn, p);
                fn->arity++;
            }
            if (!match(P, TK_COMMA)) break;
        }
        match(P, TK_RPAREN);
    }
    if (match(P, TK_ARROW)) {
        if (check(P, TK_IDENT) || check(P, TK_ROOT)) {
            fn->type_name = zip_strdup(peek(P)->str);
            advance(P);
        }
    }
    Node *body = parse_block(P);
    if (body) node_add(fn, body);
    return fn;
}

Node *parse(Token *toks, size_t n) {
    Parser P = { toks, n, 0 };
    Node *prog = node_new(NODE_PROGRAM);
    Node *toplevel = node_new(NODE_BLOCK);
    int has_fn = 0;

    while (peek(&P) && !check(&P, TK_EOF)) {
        if (check(&P, TK_AT)) {
            Node *a = node_new(NODE_ANNOT);
            advance(&P);
            if (check(&P, TK_IDENT) || check(&P, TK_ROOT)) {
                a->name = zip_strdup(peek(&P)->str);
                advance(&P);
            }
            if (match(&P, TK_LPAREN)) {
                while (!check(&P, TK_RPAREN) && !check(&P, TK_EOF)) advance(&P);
                match(&P, TK_RPAREN);
            }
            node_add(prog, a);
            continue;
        }
        if (check(&P, TK_HASH)) {
            Node *d = node_new(NODE_DIRECTIVE);
            advance(&P);
            if (check(&P, TK_IDENT) || check(&P, TK_ROOT) || check(&P, TK_TEMPLATE)) {
                d->name = zip_strdup(peek(&P)->str);
                advance(&P);
            }
            if (match(&P, TK_LPAREN)) {
                if (check(&P, TK_STRING)) {
                    d->str = zip_strdup(peek(&P)->str);
                    advance(&P);
                }
                while (!check(&P, TK_RPAREN) && !check(&P, TK_EOF)) advance(&P);
                match(&P, TK_RPAREN);
            }
            node_add(prog, d);
            continue;
        }
        if (check(&P, TK_FUNCTION)) {
            Node *fn = parse_function(&P);
            if (fn) { node_add(prog, fn); has_fn = 1; }
            continue;
        }
        Node *s = parse_stmt(&P);
        if (s) node_add(toplevel, s);
        else advance(&P);
    }

    if (toplevel->n > 0) {
        Node *fn = node_new(NODE_FUNCTION);
        fn->name = zip_strdup("главная");
        node_add(fn, toplevel);
        node_add(prog, fn);
    } else {
        node_free(toplevel);
        if (!has_fn) {
            Node *fn = node_new(NODE_FUNCTION);
            fn->name = zip_strdup("главная");
            Node *body = node_new(NODE_BLOCK);
            Node *ret = node_new(NODE_RETURN);
            Node *z = node_new(NODE_CONST_I64);
            z->i64 = 0;
            node_add(ret, z);
            node_add(body, ret);
            node_add(fn, body);
            node_add(prog, fn);
        }
    }
    return prog;
}
