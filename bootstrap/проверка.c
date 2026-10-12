#include "zipb.h"

/* AST validation / diagnostics for bootstrap compiler. */

static int g_errors;

static void err(const char *msg) {
    fprintf(stderr, "проверка: %s\n", msg);
    g_errors++;
}

static void walk(Node *n, int depth) {
    if (!n) return;
    if (depth > 256) { err("слишком глубокий AST"); return; }
    switch (n->type) {
    case NODE_FUNCTION:
        if (!n->name || !n->name[0]) err("функция без имени");
        break;
    case NODE_ASSIGN:
        if (!n->name) err("присваивание без имени");
        if (n->n < 1) err("присваивание без RHS");
        break;
    case NODE_CALL:
        if (!n->name) err("вызов без имени");
        break;
    case NODE_BINOP:
        if (n->n < 2) err("бинарная операция без операндов");
        break;
    case NODE_IF:
        if (n->n < 2) err("если без условия/тела");
        break;
    case NODE_WHILE:
        if (n->n < 2) err("пока без условия/тела");
        break;
    case NODE_FOR:
        if (n->n < 3) err("для: ожидается lo..hi и тело");
        break;
    case NODE_RETURN:
        break;
    default:
        break;
    }
    for (int i = 0; i < n->n; i++) walk(n->kids[i], depth + 1);
}

int validate_ast(Node *ast) {
    g_errors = 0;
    if (!ast) { err("пустой AST"); return 1; }
    if (ast->type != NODE_PROGRAM) err("корень не PROGRAM");
    int fns = 0;
    for (int i = 0; i < ast->n; i++) {
        if (ast->kids[i]->type == NODE_FUNCTION) fns++;
        walk(ast->kids[i], 0);
    }
    if (fns == 0) err("нет функций");
    return g_errors;
}

void print_ast(Node *n, int indent) {
    if (!n) return;
    for (int i = 0; i < indent; i++) putchar(' ');
    const char *tn = "?";
    switch (n->type) {
    case NODE_PROGRAM: tn = "PROGRAM"; break;
    case NODE_FUNCTION: tn = "FUNCTION"; break;
    case NODE_PARAM: tn = "PARAM"; break;
    case NODE_CALL: tn = "CALL"; break;
    case NODE_CONST_I64: tn = "I64"; break;
    case NODE_CONST_F64: tn = "F64"; break;
    case NODE_CONST_STR: tn = "STR"; break;
    case NODE_CONST_BOOL: tn = "BOOL"; break;
    case NODE_IDENT: tn = "IDENT"; break;
    case NODE_BINOP: tn = "BINOP"; break;
    case NODE_UNARY: tn = "UNARY"; break;
    case NODE_ASSIGN: tn = "ASSIGN"; break;
    case NODE_IF: tn = "IF"; break;
    case NODE_WHILE: tn = "WHILE"; break;
    case NODE_FOR: tn = "FOR"; break;
    case NODE_RETURN: tn = "RETURN"; break;
    case NODE_BLOCK: tn = "BLOCK"; break;
    case NODE_PRINT_STR: tn = "PRINT_STR"; break;
    case NODE_PRINT_I64: tn = "PRINT_I64"; break;
    case NODE_PULSE: tn = "PULSE"; break;
    case NODE_CHOIR: tn = "CHOIR"; break;
    case NODE_ANNOT: tn = "ANNOT"; break;
    case NODE_DIRECTIVE: tn = "DIRECTIVE"; break;
    case NODE_STRUCT: tn = "STRUCT"; break;
    case NODE_FIELD: tn = "FIELD"; break;
    case NODE_STRUCT_LIT: tn = "STRUCT_LIT"; break;
    case NODE_FIELD_ACCESS: tn = "FIELD_ACCESS"; break;
    case NODE_FIELD_STORE: tn = "FIELD_STORE"; break;
    case NODE_ARRAY_TYPE: tn = "ARRAY_TYPE"; break;
    case NODE_ARRAY_LIT: tn = "ARRAY_LIT"; break;
    case NODE_INDEX_ACCESS: tn = "INDEX_ACCESS"; break;
    case NODE_INDEX_STORE: tn = "INDEX_STORE"; break;
    case NODE_ADDR_OF: tn = "ADDR_OF"; break;
    case NODE_DEREF: tn = "DEREF"; break;
    case NODE_ALLOC: tn = "ALLOC"; break;
    case NODE_FREE: tn = "FREE"; break;
    case NODE_STORE_PTR: tn = "STORE_PTR"; break;
    default: break;
    }
    printf("%s", tn);
    if (n->name) printf(" name=%s", n->name);
    if (n->type == NODE_CONST_I64) printf(" val=%lld", (long long)n->i64);
    if (n->type == NODE_CONST_STR && n->str) printf(" \"%s\"", n->str);
    if (n->type == NODE_BINOP) printf(" op=%s", token_name((TokenType)n->op));
    printf("\n");
    for (int i = 0; i < n->n; i++) print_ast(n->kids[i], indent + 2);
}
