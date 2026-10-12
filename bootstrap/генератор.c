#include "zipb.h"

typedef struct {
    char *names[ZIP_MAX_STRINGS];
    int n;
} StrTab;

typedef struct {
    char *names[ZIP_MAX_LOCALS];
    char *types[ZIP_MAX_LOCALS]; /* имя структуры или NULL */
    int n;
} Locals;

#define ZIP_MAX_STRUCTS 64
#define ZIP_MAX_FIELDS 32

typedef struct {
    char *name;
    char *fields[ZIP_MAX_FIELDS];
    char *ftypes[ZIP_MAX_FIELDS];
    int nfields;
} StructDef;

typedef struct {
    StructDef defs[ZIP_MAX_STRUCTS];
    int n;
} StructTab;

typedef struct {
    char *names[ZIP_MAX_FUNCS];
    Node *fns[ZIP_MAX_FUNCS]; /* NULL = внешняя/builtin */
    int arity_hint[ZIP_MAX_FUNCS];
    int n;
} FnTab;

typedef struct {
    Buf *bytes;
    uint32_t ipc; /* instruction index for jumps */
} Code;

static StrTab g_str;
static FnTab g_fn;
static StructTab g_st;

static void str_init(void) { g_str.n = 0; }
static void struct_init(void) { g_st.n = 0; }

static StructDef *struct_find(const char *name) {
    if (!name) return NULL;
    for (int i = 0; i < g_st.n; i++)
        if (strcmp(g_st.defs[i].name, name) == 0) return &g_st.defs[i];
    return NULL;
}

static int field_index(const char *stype, const char *field) {
    StructDef *d = struct_find(stype);
    if (!d || !field) return -1;
    for (int i = 0; i < d->nfields; i++)
        if (strcmp(d->fields[i], field) == 0) return i;
    return -1;
}

static const char *field_type(const char *stype, const char *field) {
    StructDef *d = struct_find(stype);
    if (!d || !field) return NULL;
    for (int i = 0; i < d->nfields; i++)
        if (strcmp(d->fields[i], field) == 0) return d->ftypes[i];
    return NULL;
}

static void collect_structs(Node *ast) {
    struct_init();
    if (!ast) return;
    for (int i = 0; i < ast->n; i++) {
        Node *k = ast->kids[i];
        if (k->type != NODE_STRUCT || !k->name || g_st.n >= ZIP_MAX_STRUCTS)
            continue;
        StructDef *d = &g_st.defs[g_st.n++];
        d->name = zip_strdup(k->name);
        d->nfields = 0;
        for (int f = 0; f < k->n && d->nfields < ZIP_MAX_FIELDS; f++) {
            Node *field = k->kids[f];
            if (field->type != NODE_FIELD || !field->name) continue;
            d->fields[d->nfields] = zip_strdup(field->name);
            d->ftypes[d->nfields] = zip_strdup(field->type_name ? field->type_name : "Целое");
            d->nfields++;
        }
    }
}

static const char *local_type_get(Locals *L, const char *name) {
    for (int i = 0; i < L->n; i++)
        if (strcmp(L->names[i], name) == 0) return L->types[i];
    return NULL;
}

static void local_type_set(Locals *L, const char *name, const char *ty) {
    for (int i = 0; i < L->n; i++) {
        if (strcmp(L->names[i], name) == 0) {
            free(L->types[i]);
            L->types[i] = ty ? zip_strdup(ty) : NULL;
            return;
        }
    }
}

static const char *expr_struct_type(Node *n, Locals *loc) {
    if (!n) return NULL;
    if (n->type == NODE_STRUCT_LIT) return n->name;
    if (n->type == NODE_IDENT) return local_type_get(loc, n->name ? n->name : "_");
    if (n->type == NODE_FIELD_ACCESS && n->n > 0)
        return field_type(expr_struct_type(n->kids[0], loc), n->name);
    return NULL;
}
static uint32_t str_add(const char *s) {
    if (!s) s = "";
    for (int i = 0; i < g_str.n; i++)
        if (strcmp(g_str.names[i], s) == 0) return (uint32_t)i;
    if (g_str.n >= ZIP_MAX_STRINGS) return 0;
    g_str.names[g_str.n] = zip_strdup(s);
    return (uint32_t)g_str.n++;
}

static uint32_t local_get(Locals *L, const char *name) {
    for (int i = 0; i < L->n; i++)
        if (strcmp(L->names[i], name) == 0) return (uint32_t)i;
    if (L->n >= ZIP_MAX_LOCALS) return 0;
    L->names[L->n] = zip_strdup(name);
    L->types[L->n] = NULL;
    return (uint32_t)L->n++;
}

static void fn_init(void) { g_fn.n = 0; }
static int fn_find(const char *name) {
    for (int i = 0; i < g_fn.n; i++)
        if (strcmp(g_fn.names[i], name) == 0) return i;
    return -1;
}

static void emit_expr(Node *n, Code *C, Locals *loc);
static void emit_stmt(Node *n, Code *C, Locals *loc);

static void op0(Code *C, uint8_t op) {
    buf_byte(C->bytes, op);
    C->ipc++;
}
static void op_i64(Code *C, uint8_t op, int64_t v) {
    buf_byte(C->bytes, op);
    buf_i64(C->bytes, v);
    C->ipc++;
}
static void op_f64(Code *C, uint8_t op, double v) {
    buf_byte(C->bytes, op);
    buf_f64(C->bytes, v);
    C->ipc++;
}
static void op_u32(Code *C, uint8_t op, uint32_t v) {
    buf_byte(C->bytes, op);
    buf_u32(C->bytes, v);
    C->ipc++;
}
static void op_u16x2(Code *C, uint8_t op, uint16_t a, uint16_t b) {
    buf_byte(C->bytes, op);
    buf_u16(C->bytes, a);
    buf_u16(C->bytes, b);
    C->ipc++;
}
static void op_u16(Code *C, uint8_t op, uint16_t a) {
    buf_byte(C->bytes, op);
    buf_u16(C->bytes, a);
    C->ipc++;
}
/* emit jump opcode + placeholder; returns byte offset of u32 patch */
static size_t emit_jump(Code *C, uint8_t op) {
    buf_byte(C->bytes, op);
    size_t at = C->bytes->len;
    buf_u32(C->bytes, 0);
    C->ipc++;
    return at;
}

static Opcode binop_to_op(int tok) {
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

static void emit_expr(Node *n, Code *C, Locals *loc) {
    if (!n) return;
    switch (n->type) {
    case NODE_CONST_I64:
        op_i64(C, OP_PUSH_I64, n->i64);
        break;
    case NODE_CONST_F64:
        op_f64(C, OP_PUSH_F64, n->f64);
        break;
    case NODE_CONST_STR:
        op_u32(C, OP_PUSH_STR, str_add(n->str));
        break;
    case NODE_CONST_BOOL:
        op0(C, n->i64 ? OP_PUSH_TRUE : OP_PUSH_FALSE);
        break;
    case NODE_IDENT:
        op_u32(C, OP_LOAD, local_get(loc, n->name ? n->name : "_"));
        break;
    case NODE_UNARY:
        if (n->op == TK_MINUS) {
            op_i64(C, OP_PUSH_I64, 0);
            if (n->n > 0) emit_expr(n->kids[0], C, loc);
            op0(C, OP_SUB);
        } else if (n->op == TK_NOT) {
            if (n->n > 0) emit_expr(n->kids[0], C, loc);
            op_i64(C, OP_PUSH_I64, 0);
            op0(C, OP_EQ);
        } else if (n->n > 0) emit_expr(n->kids[0], C, loc);
        break;
    case NODE_BINOP:
        if (n->op == TK_AND) {
            if (n->n > 0) emit_expr(n->kids[0], C, loc);
            size_t jfalse = emit_jump(C, OP_JUMP_IF_NOT);
            if (n->n > 1) emit_expr(n->kids[1], C, loc);
            size_t jend = emit_jump(C, OP_JUMP);
            buf_patch_u32(C->bytes, jfalse, C->ipc);
            op_i64(C, OP_PUSH_I64, 0);
            buf_patch_u32(C->bytes, jend, C->ipc);
        } else if (n->op == TK_OR) {
            if (n->n > 0) emit_expr(n->kids[0], C, loc);
            size_t jtrue = emit_jump(C, OP_JUMP_IF);
            if (n->n > 1) emit_expr(n->kids[1], C, loc);
            size_t jend = emit_jump(C, OP_JUMP);
            buf_patch_u32(C->bytes, jtrue, C->ipc);
            op_i64(C, OP_PUSH_I64, 1);
            buf_patch_u32(C->bytes, jend, C->ipc);
        } else {
            if (n->n > 0) emit_expr(n->kids[0], C, loc);
            if (n->n > 1) emit_expr(n->kids[1], C, loc);
            op0(C, (uint8_t)binop_to_op(n->op));
        }
        break;
    case NODE_PULSE:
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        if (n->n > 1) emit_expr(n->kids[1], C, loc);
        op_u16x2(C, OP_PULSE, 0, 1);
        break;
    case NODE_CHOIR:
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        if (n->n > 1) emit_expr(n->kids[1], C, loc);
        op0(C, OP_CHOIR_JOIN);
        break;
    case NODE_PERFECT:
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        op_u16(C, OP_PERFECT_LOCK, 0);
        break;
    case NODE_PRINT_STR:
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        else op_u32(C, OP_PUSH_STR, str_add(""));
        op0(C, OP_PRINTLN);
        break;
    case NODE_PRINT_I64:
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        else op_i64(C, OP_PUSH_I64, 0);
        op0(C, OP_PRINT_I64);
        break;
    case NODE_CALL: {
        /* Builtin: длина_массива(arr) */
        if (n->name && (zip_streq(n->name, "длина_массива") || zip_streq(n->name, "длинаМассива"))) {
            if (n->n > 0) emit_expr(n->kids[0], C, loc);
            else op_i64(C, OP_PUSH_I64, 0);
            op0(C, OP_ARRAY_LEN);
            break;
        }
        for (int i = 0; i < n->n; i++) emit_expr(n->kids[i], C, loc);
        int fi = n->name ? fn_find(n->name) : -1;
        if (fi >= 0) {
            op_u32(C, OP_CALL, (uint32_t)fi);
        } else {
            for (int i = 0; i < n->n; i++) op0(C, OP_POP);
            op_i64(C, OP_PUSH_I64, 0);
        }
        break;
    }
    case NODE_ARRAY_LIT: {
        int64_t repeat = n->i64;
        if (repeat > 0 && n->n > 0) {
            /* [знач; N] */
            op_u32(C, OP_NEW_ARRAY, (uint32_t)repeat);
            for (int64_t i = 0; i < repeat; i++) {
                op0(C, OP_DUP);
                emit_expr(n->kids[0], C, loc);
                op_i64(C, OP_PUSH_I64, i);
                op0(C, OP_STORE_INDEX);
            }
        } else {
            op_u32(C, OP_NEW_ARRAY, (uint32_t)(n->n > 0 ? n->n : 0));
            for (int i = 0; i < n->n; i++) {
                op0(C, OP_DUP);
                emit_expr(n->kids[i], C, loc);
                op_i64(C, OP_PUSH_I64, i);
                op0(C, OP_STORE_INDEX);
            }
        }
        break;
    }
    case NODE_INDEX_ACCESS: {
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        else op_i64(C, OP_PUSH_I64, 0);
        if (n->n > 1) emit_expr(n->kids[1], C, loc);
        else op_i64(C, OP_PUSH_I64, 0);
        op0(C, OP_LOAD_INDEX);
        break;
    }
    case NODE_STRUCT_LIT: {
        StructDef *d = struct_find(n->name);
        int nfields = d ? d->nfields : n->n;
        if (nfields < 0) nfields = 0;
        op_u32(C, OP_NEW_STRUCT, (uint32_t)nfields);
        if (d) {
            for (int i = 0; i < d->nfields; i++) {
                Node *val = NULL;
                for (int k = 0; k < n->n; k++) {
                    Node *f = n->kids[k];
                    if (f->type == NODE_FIELD && f->name
                        && strcmp(f->name, d->fields[i]) == 0 && f->n > 0) {
                        val = f->kids[0];
                        break;
                    }
                }
                op0(C, OP_DUP);
                if (val) emit_expr(val, C, loc);
                else op_i64(C, OP_PUSH_I64, 0);
                op_u32(C, OP_STORE_FIELD, (uint32_t)i);
            }
        } else {
            for (int i = 0; i < n->n; i++) {
                Node *f = n->kids[i];
                op0(C, OP_DUP);
                if (f->n > 0) emit_expr(f->kids[0], C, loc);
                else op_i64(C, OP_PUSH_I64, 0);
                op_u32(C, OP_STORE_FIELD, (uint32_t)i);
            }
        }
        break;
    }
    case NODE_FIELD_ACCESS: {
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        else op_i64(C, OP_PUSH_I64, 0);
        {
            const char *st = n->n > 0 ? expr_struct_type(n->kids[0], loc) : NULL;
            int idx = field_index(st, n->name);
            if (idx < 0) idx = 0;
            op_u32(C, OP_LOAD_FIELD, (uint32_t)idx);
        }
        break;
    }
    default:
        for (int i = 0; i < n->n; i++) emit_expr(n->kids[i], C, loc);
        break;
    }
}

static void emit_stmt(Node *n, Code *C, Locals *loc) {
    if (!n) return;
    switch (n->type) {
    case NODE_RETURN:
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        else op_i64(C, OP_PUSH_I64, 0);
        op0(C, OP_RET);
        break;
    case NODE_ASSIGN:
        if (n->n > 0) {
            emit_expr(n->kids[0], C, loc);
            local_get(loc, n->name ? n->name : "_");
            local_type_set(loc, n->name ? n->name : "_", expr_struct_type(n->kids[0], loc));
        }
        op_u32(C, OP_STORE, local_get(loc, n->name ? n->name : "_"));
        break;
    case NODE_FIELD_STORE: {
        /* объект, значение — STORE_FIELD индекс */
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        else op_i64(C, OP_PUSH_I64, 0);
        if (n->n > 1) emit_expr(n->kids[1], C, loc);
        else op_i64(C, OP_PUSH_I64, 0);
        {
            const char *st = n->n > 0 ? expr_struct_type(n->kids[0], loc) : NULL;
            int idx = field_index(st, n->name);
            if (idx < 0) idx = 0;
            op_u32(C, OP_STORE_FIELD, (uint32_t)idx);
        }
        break;
    }
    case NODE_INDEX_STORE: {
        /* стек: массив, значение, индекс */
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        else op_i64(C, OP_PUSH_I64, 0);
        if (n->n > 2) emit_expr(n->kids[2], C, loc);
        else op_i64(C, OP_PUSH_I64, 0);
        if (n->n > 1) emit_expr(n->kids[1], C, loc);
        else op_i64(C, OP_PUSH_I64, 0);
        op0(C, OP_STORE_INDEX);
        break;
    }
    case NODE_IF: {
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        size_t jelse = emit_jump(C, OP_JUMP_IF_NOT);
        if (n->n > 1) emit_stmt(n->kids[1], C, loc);
        size_t jend = emit_jump(C, OP_JUMP);
        buf_patch_u32(C->bytes, jelse, C->ipc);
        if (n->n > 2) emit_stmt(n->kids[2], C, loc);
        buf_patch_u32(C->bytes, jend, C->ipc);
        break;
    }
    case NODE_WHILE: {
        uint32_t loop = C->ipc;
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        size_t jend = emit_jump(C, OP_JUMP_IF_NOT);
        if (n->n > 1) emit_stmt(n->kids[1], C, loc);
        size_t jback = emit_jump(C, OP_JUMP);
        buf_patch_u32(C->bytes, jback, loop);
        buf_patch_u32(C->bytes, jend, C->ipc);
        break;
    }
    case NODE_FOR: {
        const char *var = n->name ? n->name : "и";
        uint32_t slot = local_get(loc, var);
        if (n->n > 0) emit_expr(n->kids[0], C, loc);
        op_u32(C, OP_STORE, slot);
        uint32_t loop = C->ipc;
        op_u32(C, OP_LOAD, slot);
        if (n->n > 1) emit_expr(n->kids[1], C, loc);
        op0(C, OP_LT);
        size_t jend = emit_jump(C, OP_JUMP_IF_NOT);
        if (n->n > 2) emit_stmt(n->kids[2], C, loc);
        op_u32(C, OP_LOAD, slot);
        op_i64(C, OP_PUSH_I64, 1);
        op0(C, OP_ADD);
        op_u32(C, OP_STORE, slot);
        size_t jback = emit_jump(C, OP_JUMP);
        buf_patch_u32(C->bytes, jback, loop);
        buf_patch_u32(C->bytes, jend, C->ipc);
        break;
    }
    case NODE_BLOCK:
        for (int i = 0; i < n->n; i++) emit_stmt(n->kids[i], C, loc);
        break;
    case NODE_BREAK:
    case NODE_CONTINUE:
        break;
    case NODE_PRINT_STR:
    case NODE_PRINT_I64:
    case NODE_CALL:
    case NODE_BINOP:
    case NODE_CONST_I64:
    case NODE_CONST_STR:
    case NODE_PULSE:
    case NODE_CHOIR:
    case NODE_STRUCT_LIT:
    case NODE_FIELD_ACCESS:
    case NODE_ARRAY_LIT:
    case NODE_INDEX_ACCESS:
        emit_expr(n, C, loc);
        break;
    default:
        for (int i = 0; i < n->n; i++) emit_stmt(n->kids[i], C, loc);
        break;
    }
}

static void collect_externals(Node *n) {
    if (!n) return;
    if (n->type == NODE_CALL && n->name && fn_find(n->name) < 0 && g_fn.n < ZIP_MAX_FUNCS) {
        g_fn.names[g_fn.n] = zip_strdup(n->name);
        g_fn.fns[g_fn.n] = NULL;
        g_fn.arity_hint[g_fn.n] = n->n;
        g_fn.n++;
        str_add(n->name);
    }
    for (int i = 0; i < n->n; i++) collect_externals(n->kids[i]);
}

static void collect_fns(Node *ast) {
    fn_init();
    if (!ast) return;
    for (int i = 0; i < ast->n; i++) {
        Node *k = ast->kids[i];
        if (k->type == NODE_FUNCTION && k->name && g_fn.n < ZIP_MAX_FUNCS) {
            g_fn.names[g_fn.n] = zip_strdup(k->name);
            g_fn.fns[g_fn.n] = k;
            g_fn.arity_hint[g_fn.n] = k->arity;
            g_fn.n++;
            str_add(k->name);
        }
    }
    collect_externals(ast);
}

static void emit_function(Node *fn, Code *C, uint16_t *out_locals, uint16_t *out_arity, int arity_hint) {
    Locals loc = {0};
    int arity = 0;
    Node *body = NULL;
    if (!fn) {
        /* Внешняя/builtin: тело не исполняется — VM перехватит по имени */
        *out_arity = (uint16_t)(arity_hint < 0 ? 0 : arity_hint);
        *out_locals = 16;
        op_i64(C, OP_PUSH_I64, 0);
        op0(C, OP_RET);
        return;
    }
    for (int i = 0; i < fn->n; i++) {
        if (fn->kids[i]->type == NODE_PARAM) {
            local_get(&loc, fn->kids[i]->name ? fn->kids[i]->name : "_");
            arity++;
        } else if (fn->kids[i]->type == NODE_BLOCK) {
            body = fn->kids[i];
        }
    }
    *out_arity = (uint16_t)arity;
    if (body) emit_stmt(body, C, &loc);
    if (C->ipc == 0 || C->bytes->data[C->bytes->len - 1] != OP_RET) {
        op_i64(C, OP_PUSH_I64, 0);
        op0(C, OP_RET);
    }
    *out_locals = (uint16_t)(loc.n < 16 ? 16 : loc.n + 4);
    for (int i = 0; i < loc.n; i++) {
        free(loc.names[i]);
        free(loc.types[i]);
    }
}

Buf *codegen(Node *ast) {
    str_init();
    collect_structs(ast);
    collect_fns(ast);

    Buf *codes[ZIP_MAX_FUNCS];
    uint16_t locals[ZIP_MAX_FUNCS];
    uint16_t arity[ZIP_MAX_FUNCS];
    int nfn = g_fn.n;

    if (nfn == 0) {
        codes[0] = buf_new();
        Code C = { codes[0], 0 };
        op_i64(&C, OP_PUSH_I64, 0);
        op0(&C, OP_RET);
        locals[0] = 16;
        arity[0] = 0;
        str_add("главная");
        nfn = 1;
        g_fn.names[0] = zip_strdup("главная");
        g_fn.n = 1;
    } else {
        for (int i = 0; i < nfn; i++) {
            codes[i] = buf_new();
            Code C = { codes[i], 0 };
            emit_function(g_fn.fns[i], &C, &locals[i], &arity[i], g_fn.arity_hint[i]);
        }
    }

    /* ZIPB = header | strings | functions  (как сериализовать_v3) */
    Buf *out = buf_new();
    buf_bytes(out, ZIPB_MAGIC, 4);
    buf_u16(out, ZIPB_VERSION);
    buf_u16(out, 0);
    buf_u32(out, (uint32_t)nfn);
    buf_u32(out, 16);

    buf_u32(out, (uint32_t)g_str.n);
    for (int i = 0; i < g_str.n; i++) {
        const char *s = g_str.names[i] ? g_str.names[i] : "";
        uint32_t L = (uint32_t)strlen(s);
        buf_u32(out, L);
        buf_bytes(out, s, L);
    }

    for (int i = 0; i < nfn; i++) {
        uint32_t ni = 0;
        const char *nm = g_fn.names[i] ? g_fn.names[i] : "главная";
        for (int s = 0; s < g_str.n; s++) {
            if (strcmp(g_str.names[s], nm) == 0) { ni = (uint32_t)s; break; }
        }
        buf_u32(out, ni);
        buf_u16(out, arity[i]);
        buf_u16(out, locals[i]);
        buf_u32(out, 0);
        buf_u32(out, (uint32_t)codes[i]->len);
        buf_bytes(out, codes[i]->data, codes[i]->len);
        buf_free(codes[i]);
    }

    for (int i = 0; i < g_str.n; i++) free(g_str.names[i]);
    for (int i = 0; i < g_fn.n; i++) free(g_fn.names[i]);
    for (int i = 0; i < g_st.n; i++) {
        free(g_st.defs[i].name);
        for (int f = 0; f < g_st.defs[i].nfields; f++) {
            free(g_st.defs[i].fields[f]);
            free(g_st.defs[i].ftypes[f]);
        }
    }
    return out;
}
