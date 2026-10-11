/* ZIP-SELF: эмиттер байт-кода ZIP v3 (.zipb) — зеркало zipc/src/байткод_эмиттер_v3.rs */
#define _CRT_SECURE_NO_WARNINGS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

/* Лексер + парсер + IR */
#include "промежуток_rt.c"

/* ===== Буфер байт-кода ===== */
typedef struct {
    uint8_t *data;
    int len;
    int cap;
} ByteBuf;

static void bb_init(ByteBuf *b, int cap) {
    b->cap = cap > 0 ? cap : 4096;
    b->len = 0;
    b->data = (uint8_t *)malloc((size_t)b->cap);
}

static void bb_reserve(ByteBuf *b, int need) {
    if (b->len + need <= b->cap) return;
    int nc = b->cap;
    while (nc < b->len + need) nc *= 2;
    b->data = (uint8_t *)realloc(b->data, (size_t)nc);
    b->cap = nc;
}

static void bb_u8(ByteBuf *b, uint8_t v) {
    bb_reserve(b, 1);
    b->data[b->len++] = v;
}

static void bb_u16(ByteBuf *b, uint16_t v) {
    bb_u8(b, (uint8_t)(v & 0xFF));
    bb_u8(b, (uint8_t)((v >> 8) & 0xFF));
}

static void bb_u32(ByteBuf *b, uint32_t v) {
    bb_u8(b, (uint8_t)(v & 0xFF));
    bb_u8(b, (uint8_t)((v >> 8) & 0xFF));
    bb_u8(b, (uint8_t)((v >> 16) & 0xFF));
    bb_u8(b, (uint8_t)((v >> 24) & 0xFF));
}

static void bb_i64(ByteBuf *b, int64_t v) {
    uint64_t u = (uint64_t)v;
    for (int i = 0; i < 8; i++)
        bb_u8(b, (uint8_t)((u >> (i * 8)) & 0xFF));
}

static void bb_f64(ByteBuf *b, double v) {
    uint64_t u;
    memcpy(&u, &v, 8);
    for (int i = 0; i < 8; i++)
        bb_u8(b, (uint8_t)((u >> (i * 8)) & 0xFF));
}

static void bb_bytes(ByteBuf *b, const void *p, int n) {
    bb_reserve(b, n);
    memcpy(b->data + b->len, p, (size_t)n);
    b->len += n;
}

/* ===== Таблица строк ===== */
typedef struct {
    char **items;
    int n;
    int cap;
} StrTab;

static void st_init(StrTab *t) {
    t->cap = 64;
    t->n = 0;
    t->items = (char **)calloc((size_t)t->cap, sizeof(char *));
}

static uint32_t st_add(StrTab *t, const char *s) {
    if (!s) s = "";
    for (int i = 0; i < t->n; i++)
        if (strcmp(t->items[i], s) == 0) return (uint32_t)i;
    if (t->n >= t->cap) {
        t->cap *= 2;
        t->items = (char **)realloc(t->items, (size_t)t->cap * sizeof(char *));
    }
    size_t len = strlen(s);
    char *c = (char *)malloc(len + 1);
    memcpy(c, s, len + 1);
    t->items[t->n] = c;
    return (uint32_t)t->n++;
}

static void st_free(StrTab *t) {
    for (int i = 0; i < t->n; i++) free(t->items[i]);
    free(t->items);
}

/* Промежуточные опкоды эмиттера (до резолва меток) */
enum {
    OP_RAW = 1,
    OP_LABEL = 2,
    OP_JUMP = 3,
    OP_JUMP_IF = 4
};

typedef struct {
    int kind; /* OP_* */
    uint8_t *bytes;
    int nbytes;
    int label_id;
} EmOp;

typedef struct {
    EmOp *ops;
    int n;
    int cap;
} EmOps;

static void ops_init(EmOps *o) {
    o->cap = 256;
    o->n = 0;
    o->ops = (EmOp *)calloc((size_t)o->cap, sizeof(EmOp));
}

static void ops_push_raw(EmOps *o, const uint8_t *b, int n) {
    if (o->n >= o->cap) {
        o->cap *= 2;
        o->ops = (EmOp *)realloc(o->ops, (size_t)o->cap * sizeof(EmOp));
    }
    EmOp *e = &o->ops[o->n++];
    e->kind = OP_RAW;
    e->nbytes = n;
    e->bytes = (uint8_t *)malloc((size_t)n);
    memcpy(e->bytes, b, (size_t)n);
    e->label_id = 0;
}

static void ops_push_u8(EmOps *o, uint8_t op) { ops_push_raw(o, &op, 1); }

static void ops_push_op_u32(EmOps *o, uint8_t op, uint32_t v) {
    uint8_t buf[5];
    buf[0] = op;
    buf[1] = (uint8_t)(v & 0xFF);
    buf[2] = (uint8_t)((v >> 8) & 0xFF);
    buf[3] = (uint8_t)((v >> 16) & 0xFF);
    buf[4] = (uint8_t)((v >> 24) & 0xFF);
    ops_push_raw(o, buf, 5);
}

static void ops_push_op_u16(EmOps *o, uint8_t op, uint16_t v) {
    uint8_t buf[3];
    buf[0] = op;
    buf[1] = (uint8_t)(v & 0xFF);
    buf[2] = (uint8_t)((v >> 8) & 0xFF);
    ops_push_raw(o, buf, 3);
}

static void ops_push_push_i64(EmOps *o, int64_t v) {
    uint8_t buf[9];
    buf[0] = 0x10;
    uint64_t u = (uint64_t)v;
    for (int i = 0; i < 8; i++) buf[1 + i] = (uint8_t)((u >> (i * 8)) & 0xFF);
    ops_push_raw(o, buf, 9);
}

static void ops_push_push_f64(EmOps *o, double v) {
    uint8_t buf[9];
    buf[0] = 0x11;
    uint64_t u;
    memcpy(&u, &v, 8);
    for (int i = 0; i < 8; i++) buf[1 + i] = (uint8_t)((u >> (i * 8)) & 0xFF);
    ops_push_raw(o, buf, 9);
}

static void ops_push_label(EmOps *o, int id) {
    if (o->n >= o->cap) {
        o->cap *= 2;
        o->ops = (EmOp *)realloc(o->ops, (size_t)o->cap * sizeof(EmOp));
    }
    EmOp *e = &o->ops[o->n++];
    e->kind = OP_LABEL;
    e->bytes = NULL;
    e->nbytes = 0;
    e->label_id = id;
}

static void ops_push_jump(EmOps *o, int kind, int label) {
    if (o->n >= o->cap) {
        o->cap *= 2;
        o->ops = (EmOp *)realloc(o->ops, (size_t)o->cap * sizeof(EmOp));
    }
    EmOp *e = &o->ops[o->n++];
    e->kind = kind;
    e->bytes = NULL;
    e->nbytes = 0;
    e->label_id = label;
}

static void ops_free(EmOps *o) {
    for (int i = 0; i < o->n; i++) free(o->ops[i].bytes);
    free(o->ops);
}

static int fn_index_of(Gen *G, int id) {
    for (int i = 0; i < G->nfuncs; i++)
        if (G->funcs[i].id == id) return i;
    return 0;
}

static int max_reg_fn(IrFunc *F) {
    int maxr = 0;
    for (int i = 0; i < F->nparams; i++)
        if (F->params[i] > maxr) maxr = F->params[i];
    for (int i = 0; i < F->ninsns; i++) {
        Insn *in = &F->body[i];
        int regs[8];
        int nr = 0;
        switch (in->kind) {
            case IR_CONST_I:
            case IR_CONST_F:
            case IR_CONST_S:
            case IR_ALLOC:
                regs[nr++] = in->dst;
                break;
            case IR_MOVE:
            case IR_IMPULSE:
                regs[nr++] = in->dst;
                regs[nr++] = in->a;
                break;
            case IR_ADD:
            case IR_SUB:
            case IR_MUL:
            case IR_DIV:
            case IR_CMP:
                regs[nr++] = in->dst;
                regs[nr++] = in->a;
                regs[nr++] = in->b;
                break;
            case IR_LOAD:
            case IR_STORE:
            case IR_WAIT:
            case IR_FIX:
            case IR_PRINT:
                regs[nr++] = in->a;
                if (in->kind == IR_LOAD || in->kind == IR_STORE) regs[nr++] = in->dst;
                break;
            case IR_BRANCH:
                regs[nr++] = in->a;
                break;
            case IR_CALL:
            case IR_EXTERN:
            case IR_START_CHOIR:
                for (int j = 0; j < in->nargs; j++) regs[nr++] = in->args[j];
                if (in->has_result) regs[nr++] = in->result;
                break;
            case IR_RET:
                if (in->has_result) regs[nr++] = in->result;
                break;
            default:
                break;
        }
        for (int j = 0; j < nr; j++)
            if (regs[j] > maxr) maxr = regs[j];
    }
    return maxr;
}

static void emit_insn(EmOps *ops, Gen *G, StrTab *st, const Insn *in) {
    switch (in->kind) {
        case IR_LABEL:
            ops_push_label(ops, in->label);
            break;
        case IR_CONST_I:
            ops_push_push_i64(ops, in->ival);
            ops_push_op_u32(ops, 0x41, (uint32_t)in->dst); /* Store */
            break;
        case IR_CONST_F:
            ops_push_push_f64(ops, in->fval);
            ops_push_op_u32(ops, 0x41, (uint32_t)in->dst);
            break;
        case IR_CONST_S: {
            uint32_t idx = st_add(st, in->sval);
            ops_push_op_u32(ops, 0x12, idx); /* PushStr */
            ops_push_op_u32(ops, 0x41, (uint32_t)in->dst);
            break;
        }
        case IR_MOVE:
            ops_push_op_u32(ops, 0x40, (uint32_t)in->a); /* Load */
            ops_push_op_u32(ops, 0x41, (uint32_t)in->dst);
            break;
        case IR_IMPULSE: {
            /* Pulse: u16 from, u16 to */
            uint8_t buf[5];
            buf[0] = 0x70;
            uint16_t from = (uint16_t)in->a;
            uint16_t to = (uint16_t)in->dst;
            buf[1] = (uint8_t)(from & 0xFF);
            buf[2] = (uint8_t)((from >> 8) & 0xFF);
            buf[3] = (uint8_t)(to & 0xFF);
            buf[4] = (uint8_t)((to >> 8) & 0xFF);
            ops_push_raw(ops, buf, 5);
            break;
        }
        case IR_ADD:
            ops_push_op_u32(ops, 0x40, (uint32_t)in->a);
            ops_push_op_u32(ops, 0x40, (uint32_t)in->b);
            ops_push_u8(ops, 0x20);
            ops_push_op_u32(ops, 0x41, (uint32_t)in->dst);
            break;
        case IR_SUB:
            ops_push_op_u32(ops, 0x40, (uint32_t)in->a);
            ops_push_op_u32(ops, 0x40, (uint32_t)in->b);
            ops_push_u8(ops, 0x21);
            ops_push_op_u32(ops, 0x41, (uint32_t)in->dst);
            break;
        case IR_MUL:
            ops_push_op_u32(ops, 0x40, (uint32_t)in->a);
            ops_push_op_u32(ops, 0x40, (uint32_t)in->b);
            ops_push_u8(ops, 0x22);
            ops_push_op_u32(ops, 0x41, (uint32_t)in->dst);
            break;
        case IR_DIV:
            ops_push_op_u32(ops, 0x40, (uint32_t)in->a);
            ops_push_op_u32(ops, 0x40, (uint32_t)in->b);
            ops_push_u8(ops, 0x23);
            ops_push_op_u32(ops, 0x41, (uint32_t)in->dst);
            break;
        case IR_CMP: {
            ops_push_op_u32(ops, 0x40, (uint32_t)in->a);
            ops_push_op_u32(ops, 0x40, (uint32_t)in->b);
            uint8_t op = 0x31; /* Ne default */
            if (strcmp(in->op, "==") == 0) op = 0x30;
            else if (strcmp(in->op, "!=") == 0) op = 0x31;
            else if (strcmp(in->op, "<") == 0) op = 0x32;
            else if (strcmp(in->op, "<=") == 0) op = 0x33;
            else if (strcmp(in->op, ">") == 0) op = 0x34;
            else if (strcmp(in->op, ">=") == 0) op = 0x35;
            ops_push_u8(ops, op);
            ops_push_op_u32(ops, 0x41, (uint32_t)in->dst);
            break;
        }
        case IR_PRINT: {
            ops_push_op_u32(ops, 0x40, (uint32_t)in->a);
            uint8_t op = 0x90; /* Print */
            if (in->print_mode == 1) op = 0x91;      /* PrintLn */
            else if (in->print_mode == 2) op = 0x92; /* PrintI64 */
            ops_push_u8(ops, op);
            break;
        }
        case IR_FIX:
            ops_push_op_u16(ops, 0x71, (uint16_t)in->a); /* PerfectLock */
            break;
        case IR_START_CHOIR:
            for (int i = 0; i < in->nargs; i++)
                ops_push_op_u32(ops, 0x40, (uint32_t)in->args[i]);
            ops_push_op_u32(ops, 0x72, (uint32_t)fn_index_of(G, in->func));
            break;
        case IR_JOIN_CHOIR:
            ops_push_u8(ops, 0x73);
            break;
        case IR_CALL:
            for (int i = 0; i < in->nargs; i++)
                ops_push_op_u32(ops, 0x40, (uint32_t)in->args[i]);
            ops_push_op_u32(ops, 0x60, (uint32_t)fn_index_of(G, in->func));
            if (in->has_result)
                ops_push_op_u32(ops, 0x41, (uint32_t)in->result);
            break;
        case IR_GOTO:
            ops_push_jump(ops, OP_JUMP, in->label);
            break;
        case IR_BRANCH:
            ops_push_op_u32(ops, 0x40, (uint32_t)in->a);
            ops_push_jump(ops, OP_JUMP_IF, in->label);
            ops_push_jump(ops, OP_JUMP, in->label_f);
            break;
        case IR_RET:
            if (in->has_result)
                ops_push_op_u32(ops, 0x40, (uint32_t)in->result);
            else
                ops_push_push_i64(ops, 0);
            ops_push_u8(ops, 0x61); /* Ret */
            break;
        case IR_EXTERN: {
            uint32_t li = st_add(st, in->lib);
            uint32_t fi = st_add(st, in->fname);
            uint8_t buf[9];
            buf[0] = 0x62;
            buf[1] = (uint8_t)(li & 0xFF);
            buf[2] = (uint8_t)((li >> 8) & 0xFF);
            buf[3] = (uint8_t)((li >> 16) & 0xFF);
            buf[4] = (uint8_t)((li >> 24) & 0xFF);
            buf[5] = (uint8_t)(fi & 0xFF);
            buf[6] = (uint8_t)((fi >> 8) & 0xFF);
            buf[7] = (uint8_t)((fi >> 16) & 0xFF);
            buf[8] = (uint8_t)((fi >> 24) & 0xFF);
            ops_push_raw(ops, buf, 9);
            break;
        }
        case IR_WAIT:
        case IR_LOAD:
        case IR_STORE:
        case IR_ALLOC:
        default:
            ops_push_u8(ops, 0x00); /* Nop */
            break;
    }
}

static void resolve_and_encode(EmOps *ops, ByteBuf *code) {
    /* Индекс инструкции (не байт) для меток — как в Rust-эмиттере */
    int label_at[4096];
    for (int i = 0; i < 4096; i++) label_at[i] = 0;
    int idx = 0;
    for (int i = 0; i < ops->n; i++) {
        if (ops->ops[i].kind == OP_LABEL) {
            int id = ops->ops[i].label_id;
            if (id >= 0 && id < 4096) label_at[id] = idx;
        } else {
            idx++;
        }
    }
    for (int i = 0; i < ops->n; i++) {
        EmOp *e = &ops->ops[i];
        if (e->kind == OP_LABEL) continue;
        if (e->kind == OP_RAW) {
            bb_bytes(code, e->bytes, e->nbytes);
        } else if (e->kind == OP_JUMP || e->kind == OP_JUMP_IF) {
            uint8_t op = e->kind == OP_JUMP ? 0x50 : 0x51;
            uint32_t a = 0;
            if (e->label_id >= 0 && e->label_id < 4096)
                a = (uint32_t)label_at[e->label_id];
            bb_u8(code, op);
            bb_u32(code, a);
        }
    }
}

static int emit_zipb(Gen *G, ByteBuf *out) {
    StrTab st;
    st_init(&st);

    /* Имена функций в таблицу строк */
    for (int i = 0; i < G->nfuncs; i++)
        st_add(&st, G->funcs[i].name);

    /* Код каждой функции */
    ByteBuf *codes = (ByteBuf *)calloc((size_t)G->nfuncs, sizeof(ByteBuf));
    uint16_t *arities = (uint16_t *)calloc((size_t)G->nfuncs, sizeof(uint16_t));
    uint16_t *locals = (uint16_t *)calloc((size_t)G->nfuncs, sizeof(uint16_t));
    uint32_t *flags = (uint32_t *)calloc((size_t)G->nfuncs, sizeof(uint32_t));
    uint32_t *name_idx = (uint32_t *)calloc((size_t)G->nfuncs, sizeof(uint32_t));

    for (int fi = 0; fi < G->nfuncs; fi++) {
        IrFunc *F = &G->funcs[fi];
        name_idx[fi] = st_add(&st, F->name);
        arities[fi] = (uint16_t)F->nparams;
        int mr = max_reg_fn(F);
        int loc = F->nparams + 32;
        if (loc < 16) loc = 16;
        if (mr + 1 > loc) loc = mr + 1;
        locals[fi] = (uint16_t)loc;
        flags[fi] = (uint32_t)(F->async_f | (F->perfect_f << 1));

        EmOps ops;
        ops_init(&ops);
        for (int i = 0; i < F->ninsns; i++)
            emit_insn(&ops, G, &st, &F->body[i]);

        int has_ret = 0;
        for (int i = 0; i < ops.n; i++) {
            if (ops.ops[i].kind == OP_RAW && ops.ops[i].nbytes == 1 &&
                ops.ops[i].bytes[0] == 0x61)
                has_ret = 1;
        }
        if (!has_ret) {
            ops_push_push_i64(&ops, 0);
            ops_push_u8(&ops, 0x61);
        }

        bb_init(&codes[fi], 256);
        resolve_and_encode(&ops, &codes[fi]);
        ops_free(&ops);
    }

    /* Заголовок ZIPB */
    bb_init(out, 1024);
    bb_u8(out, 'Z');
    bb_u8(out, 'I');
    bb_u8(out, 'P');
    bb_u8(out, 'B');
    bb_u16(out, 3);                         /* версия */
    bb_u16(out, 0);                         /* флаги */
    bb_u32(out, (uint32_t)G->nfuncs);       /* кол-во функций */
    bb_u32(out, 16);                        /* смещение строк */

    /* Таблица строк */
    bb_u32(out, (uint32_t)st.n);
    for (int i = 0; i < st.n; i++) {
        uint32_t len = (uint32_t)strlen(st.items[i]);
        bb_u32(out, len);
        bb_bytes(out, st.items[i], (int)len);
    }

    /* Функции */
    for (int fi = 0; fi < G->nfuncs; fi++) {
        bb_u32(out, name_idx[fi]);
        bb_u16(out, arities[fi]);
        bb_u16(out, locals[fi]);
        bb_u32(out, flags[fi]);
        bb_u32(out, (uint32_t)codes[fi].len);
        bb_bytes(out, codes[fi].data, codes[fi].len);
        free(codes[fi].data);
    }

    free(codes);
    free(arities);
    free(locals);
    free(flags);
    free(name_idx);
    st_free(&st);
    return 0;
}

static int write_file_bin(const char *path, const void *data, int len) {
    FILE *f = fopen(path, "wb");
    if (!f) return -1;
    if (fwrite(data, 1, (size_t)len, f) != (size_t)len) {
        fclose(f);
        return -1;
    }
    fclose(f);
    return 0;
}

static int file_exists_p(const char *p) {
    FILE *f = fopen(p, "rb");
    if (!f) return 0;
    fclose(f);
    return 1;
}

static int find_zip_self_c(const char *fname, const char *hint, char *out, int outn) {
    if (hint) {
        char dir[1024];
        strncpy(dir, hint, sizeof(dir) - 1);
        dir[sizeof(dir) - 1] = 0;
        char *cut = strrchr(dir, '/');
        char *cut2 = strrchr(dir, '\\');
        if (cut2 && (!cut || cut2 > cut)) cut = cut2;
        if (cut) {
            *cut = 0;
            snprintf(out, (size_t)outn, "%s/%s", dir, fname);
            if (file_exists_p(out)) return 1;
            snprintf(out, (size_t)outn, "%s/../zip-self/%s", dir, fname);
            if (file_exists_p(out)) return 1;
        }
    }
    const char *prefixes[] = {".", "..", "zip-self", "../zip-self", "../../zip-self", NULL};
    for (int i = 0; prefixes[i]; i++) {
        snprintf(out, (size_t)outn, "%s/%s", prefixes[i], fname);
        if (file_exists_p(out)) return 1;
    }
    return 0;
}

static const char *find_runtime_ast(Node *n) {
    if (!n) return NULL;
    if (n->kind == N_RUNTIME && n->name) return n->name;
    if (n->kind == N_PROG || n->kind == N_BLOCK) {
        for (int i = 0; i < n->nkids; i++) {
            const char *r = find_runtime_ast(n->kids[i]);
            if (r) return r;
        }
    }
    return NULL;
}

static int map_rt(const char *name, const char **c_file, const char **macro) {
    if (!name) return 0;
    if (strcmp(name, "компилятор_rt") == 0 || strcmp(name, "compiler_rt") == 0) {
        /* ASCII filename — надёжнее fopen на Windows */
        *c_file = "compiler_rt.c";
        *macro = "ZIP_COMPILER_MAIN";
        return 1;
    }
    if (strcmp(name, "байткод_эмиттер_rt") == 0 || strcmp(name, "bytecode_emitter_rt") == 0) {
        *c_file = "bytecode_emitter_rt.c";
        *macro = "ZIP_BAJTKOD_MAIN";
        return 1;
    }
    if (strcmp(name, "native_rt") == 0 || strcmp(name, "нативный") == 0 ||
        strcmp(name, "нативный_rt") == 0) {
        *c_file = "нативный_rt.c";
        *macro = "ZIP_NATIVE_MAIN";
        return 1;
    }
    if (strcmp(name, "lexer_zip") == 0) {
        *c_file = "лексика_rt.c";
        *macro = "ZIP_LEXER_MAIN";
        return 1;
    }
    if (strcmp(name, "parser_zip") == 0 || strcmp(name, "parser_rt") == 0) {
        *c_file = "парсер_rt.c";
        *macro = "ZIP_PARSER_MAIN";
        return 1;
    }
    if (strcmp(name, "ir_zip") == 0 || strcmp(name, "промежуток_rt") == 0) {
        *c_file = "промежуток_rt.c";
        *macro = "ZIP_IR_MAIN";
        return 1;
    }
    return 0;
}

static int compile_runtime_clang(const char *c_path, const char *macro, const char *exe_path) {
    char cmd[4096];
#ifdef _WIN32
    snprintf(cmd, sizeof(cmd),
             "clang \"%s\" -O3 -march=native -D%s -Wl,/Brepro -o \"%s\"", c_path, macro, exe_path);
#else
    snprintf(cmd, sizeof(cmd), "clang \"%s\" -O3 -march=native -D%s -o \"%s\"", c_path, macro,
             exe_path);
#endif
    printf("  clang runtime: %s -D%s → %s\n", c_path, macro, exe_path);
    return system(cmd);
}

static int ends_with_ci(const char *s, const char *suf) {
    size_t n = strlen(s), m = strlen(suf);
    if (n < m) return 0;
    for (size_t i = 0; i < m; i++) {
        char a = s[n - m + i], b = suf[i];
        if (a >= 'A' && a <= 'Z') a = (char)(a - 'A' + 'a');
        if (b >= 'A' && b <= 'Z') b = (char)(b - 'A' + 'a');
        if (a != b) return 0;
    }
    return 1;
}

/* Компиляция .zip → .zipb (или #runtime → exe при самосборке) */
static int compile_zip_to_zipb(const char *in_path, const char *out_path) {
    FILE *f = fopen(in_path, "rb");
    if (!f) {
        perror(in_path);
        return 1;
    }
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    unsigned char *src = (unsigned char *)malloc((size_t)sz + 1);
    size_t n = fread(src, 1, (size_t)sz, f);
    src[n] = 0;
    fclose(f);

    printf("Компиляция:\n%s\n", in_path);
    printf("%d\n", (int)n);

    Lexer L = {0};
    L.src = src;
    L.len = n;
    L.line = 1;
    L.col = 1;
    L.cap = 256;
    L.toks = (Token *)malloc((size_t)L.cap * sizeof(Token));
    if (lex_all(&L) != 0) {
        free(src);
        free(L.toks);
        return 1;
    }
    printf("%d\n", L.ntoks);

    Parser P = {0};
    P.toks = L.toks;
    P.ntoks = L.ntoks;
    P.pos = 0;
    Node *ast = parse_program(&P);

    /* #runtime → самосборка компилятора через clang (как selfc) */
    {
        const char *rt = find_runtime_ast(ast);
        const char *cfile = NULL;
        const char *macro = NULL;
        if (rt && map_rt(rt, &cfile, &macro)) {
            /* Если выход — .zipb, всё равно собираем exe рядом (bootstrap) */
            char exe_path[512];
            strncpy(exe_path, out_path, sizeof(exe_path) - 1);
            exe_path[sizeof(exe_path) - 1] = 0;
            if (ends_with_ci(exe_path, ".zipb")) {
                size_t el = strlen(exe_path);
                exe_path[el - 5] = 0; /* strip .zipb */
            }
#ifdef _WIN32
            {
                size_t el = strlen(exe_path);
                if (el < 4 || !ends_with_ci(exe_path, ".exe"))
                    strncat(exe_path, ".exe", sizeof(exe_path) - el - 1);
            }
#endif
            char cpath[1024];
            if (!find_zip_self_c(cfile, in_path, cpath, sizeof(cpath))) {
                fprintf(stderr, "runtime «%s»: не найден %s\n", rt, cfile);
                return 1;
            }
            int rc = compile_runtime_clang(cpath, macro, exe_path);
            if (rc != 0) {
                fprintf(stderr, "Ошибка clang runtime (код %d)\n", rc);
                return 1;
            }
            printf("✓ Runtime «%s» → %s\n", rt, exe_path);
            /* Дополнительно: пустой валидный ZIPB-маркер для цепочки .zipb */
            if (ends_with_ci(out_path, ".zipb")) {
                /* Скомпилируем минимальный модуль-заглушку: главная → 0 */
                Gen G;
                gen_init(&G);
                IrFunc mainf;
                memset(&mainf, 0, sizeof(mainf));
                mainf.id = 0;
                strncpy(mainf.name, "главная", 127);
                mainf.cap = 8;
                mainf.body = (Insn *)calloc(8, sizeof(Insn));
                mainf.nparams = 0;
                Insn c0 = {0};
                c0.kind = IR_CONST_I;
                c0.dst = 0;
                c0.ival = 0;
                push(&mainf, c0);
                Insn ret = {0};
                ret.kind = IR_RET;
                ret.has_result = 1;
                ret.result = 0;
                push(&mainf, ret);
                G.funcs[G.nfuncs++] = mainf;
                ByteBuf out;
                emit_zipb(&G, &out);
                write_file_bin(out_path, out.data, out.len);
                free(out.data);
                free(mainf.body);
                printf("Готово (маркер ZIPB):\n%s\n", out_path);
            }
            for (int i = 0; i < L.ntoks; i++) free(L.toks[i].sval);
            free(L.toks);
            free(src);
            return 0;
        }
    }

    Gen G;
    generate(&G, ast);

    ByteBuf out;
    emit_zipb(&G, &out);

    char zipb_path[512];
    strncpy(zipb_path, out_path, sizeof(zipb_path) - 1);
    zipb_path[sizeof(zipb_path) - 1] = 0;
    if (!ends_with_ci(zipb_path, ".zipb"))
        strncat(zipb_path, ".zipb", sizeof(zipb_path) - strlen(zipb_path) - 1);

    if (write_file_bin(zipb_path, out.data, out.len) != 0) {
        perror(zipb_path);
        free(out.data);
        return 1;
    }
    printf("Готово:\n%s\n", zipb_path);
    printf("Функций: %d\n", G.nfuncs);

    free(out.data);
    for (int i = 0; i < L.ntoks; i++) free(L.toks[i].sval);
    free(L.toks);
    free(src);
    return 0;
}

#if defined(ZIP_BAJTKOD_MAIN) || defined(ZIP_COMPILER_MAIN)
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif
int main(int argc, char **argv) {
#ifdef _WIN32
    _setmode(_fileno(stdout), _O_BINARY);
    _setmode(_fileno(stderr), _O_BINARY);
#endif
    if (argc < 3) {
        fprintf(stderr, "Использование: %s <вход.zip> <выход.zipb>\n", argv[0]);
        return 1;
    }
    return compile_zip_to_zipb(argv[1], argv[2]);
}
#endif
