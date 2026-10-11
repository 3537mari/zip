#include "zipb.h"

typedef struct {
    char **names;
    int n;
    int cap;
} StrTab;

static StrTab stab;

static void stab_init(void) {
    stab.names = NULL;
    stab.n = 0;
    stab.cap = 0;
}

static uint32_t stab_add(const char *s) {
    if (!s) s = "";
    for (int i = 0; i < stab.n; i++)
        if (!strcmp(stab.names[i], s)) return (uint32_t)i;
    if (stab.n >= stab.cap) {
        stab.cap = stab.cap ? stab.cap * 2 : 8;
        stab.names = (char **)zip_realloc(stab.names, stab.cap * sizeof(char *));
    }
    stab.names[stab.n] = zip_strdup(s);
    return (uint32_t)stab.n++;
}

static void emit_expr(Node *n, Buf *b);

static Opcode binop_op(int tok) {
    switch (tok) {
    case TK_PLUS: return OP_ADD;
    case TK_MINUS: return OP_SUB;
    case TK_STAR: return OP_MUL;
    case TK_SLASH: return OP_DIV;
    case TK_PERCENT: return OP_MOD;
    case TK_EQEQ: return OP_EQ;
    case TK_NE: return OP_NE;
    case TK_LT: return OP_LT;
    case TK_LE: return OP_LE;
    case TK_GT: return OP_GT;
    case TK_GE: return OP_GE;
    default: return OP_NOP;
    }
}

static void emit_expr(Node *n, Buf *b) {
    if (!n) return;
    switch (n->type) {
    case NODE_CONST_I64:
        buf_byte(b, OP_PUSH_I64);
        buf_i64(b, n->i64);
        break;
    case NODE_CONST_F64:
        buf_byte(b, OP_PUSH_F64);
        /* store bits as i64 payload for simplicity — write double LE */
        {
            union { double d; int64_t i; } u;
            u.d = n->f64;
            buf_i64(b, u.i);
        }
        break;
    case NODE_CONST_STR: {
        uint32_t idx = stab_add(n->str);
        buf_byte(b, OP_PUSH_STR);
        buf_u32(b, idx);
        break;
    }
    case NODE_BINOP:
        if (n->n > 0) emit_expr(n->kids[0], b);
        if (n->n > 1) emit_expr(n->kids[1], b);
        buf_byte(b, (uint8_t)binop_op(n->op));
        break;
    case NODE_PRINT_STR:
        if (n->n > 0) emit_expr(n->kids[0], b);
        else { buf_byte(b, OP_PUSH_STR); buf_u32(b, stab_add("")); }
        buf_byte(b, OP_PRINTLN);
        break;
    case NODE_PRINT_I64:
        if (n->n > 0) emit_expr(n->kids[0], b);
        else { buf_byte(b, OP_PUSH_I64); buf_i64(b, 0); }
        buf_byte(b, OP_PRINT_I64);
        break;
    case NODE_CALL:
        for (int i = 0; i < n->n; i++) emit_expr(n->kids[i], b);
        /* unresolved call → nop for MVP bootstrap */
        break;
    case NODE_IDENT:
        /* load local 0 placeholder */
        buf_byte(b, OP_LOAD);
        buf_u32(b, 0);
        break;
    default:
        for (int i = 0; i < n->n; i++) emit_expr(n->kids[i], b);
        break;
    }
}

static void emit_stmt(Node *n, Buf *b) {
    if (!n) return;
    switch (n->type) {
    case NODE_RETURN:
        if (n->n > 0) emit_expr(n->kids[0], b);
        else { buf_byte(b, OP_PUSH_I64); buf_i64(b, 0); }
        buf_byte(b, OP_RET);
        break;
    case NODE_ASSIGN:
        if (n->n > 0) emit_expr(n->kids[0], b);
        buf_byte(b, OP_STORE);
        buf_u32(b, 0);
        break;
    case NODE_IF:
        /* MVP: emit then-branch only */
        if (n->n > 0) emit_expr(n->kids[0], b);
        if (n->n > 1) emit_stmt(n->kids[1], b);
        break;
    case NODE_WHILE:
        if (n->n > 1) emit_stmt(n->kids[1], b);
        break;
    case NODE_BLOCK:
        for (int i = 0; i < n->n; i++) emit_stmt(n->kids[i], b);
        break;
    case NODE_PRINT_STR:
    case NODE_PRINT_I64:
    case NODE_CALL:
    case NODE_BINOP:
    case NODE_CONST_I64:
    case NODE_CONST_STR:
        emit_expr(n, b);
        break;
    default:
        for (int i = 0; i < n->n; i++) emit_stmt(n->kids[i], b);
        break;
    }
}

static void emit_fn_body(Node *fn, Buf *code) {
    if (fn->n > 0) emit_stmt(fn->kids[0], code);
    /* ensure RET */
    if (code->len == 0 || code->data[code->len - 1] != OP_RET) {
        buf_byte(code, OP_PUSH_I64);
        buf_i64(code, 0);
        buf_byte(code, OP_RET);
    }
}

Buf *codegen(Node *ast) {
    stab_init();
    Buf *out = buf_new();
    Buf *code = buf_new();

    /* Collect functions */
    Node *fns[64];
    int nfn = 0;
    if (ast) {
        for (int i = 0; i < ast->n && nfn < 64; i++) {
            if (ast->kids[i]->type == NODE_FUNCTION)
                fns[nfn++] = ast->kids[i];
        }
    }
    if (nfn == 0) {
        /* empty module: one empty main */
        buf_byte(code, OP_PUSH_I64);
        buf_i64(code, 0);
        buf_byte(code, OP_RET);
        nfn = 1;
        fns[0] = NULL;
    } else {
        for (int i = 0; i < nfn; i++) emit_fn_body(fns[i], code);
    }

    /* Header: ZIPB | ver u16 | flags u16 | nfuncs u32 | str_off u32 */
    uint32_t header_size = 4 + 2 + 2 + 4 + 4;
    /* Function table: for each: name_idx u32, code_off u32, code_len u32, locals u16, argc u16 = 16 bytes */
    uint32_t ftab_size = (uint32_t)(nfn * 16);
    uint32_t code_off = header_size + ftab_size;
    uint32_t str_off = code_off + (uint32_t)code->len;

    buf_bytes(out, (const uint8_t *)ZIPB_MAGIC, 4);
    buf_u16(out, (uint16_t)ZIPB_VERSION);
    buf_u16(out, 0);
    buf_u32(out, (uint32_t)nfn);
    buf_u32(out, str_off);

    for (int i = 0; i < nfn; i++) {
        const char *name = (fns[i] && fns[i]->name) ? fns[i]->name : "главная";
        uint32_t ni = stab_add(name);
        buf_u32(out, ni);
        buf_u32(out, code_off); /* all share one blob for MVP */
        buf_u32(out, (uint32_t)code->len);
        buf_u16(out, 16); /* locals */
        buf_u16(out, 0);  /* argc */
    }

    buf_bytes(out, code->data, code->len);

    /* string table: count + (len+bytes)* */
    buf_u32(out, (uint32_t)stab.n);
    for (int i = 0; i < stab.n; i++) {
        const char *s = stab.names[i] ? stab.names[i] : "";
        uint32_t L = (uint32_t)strlen(s);
        buf_u32(out, L);
        buf_bytes(out, (const uint8_t *)s, L);
    }

    buf_free(code);
    for (int i = 0; i < stab.n; i++) free(stab.names[i]);
    free(stab.names);
    return out;
}
