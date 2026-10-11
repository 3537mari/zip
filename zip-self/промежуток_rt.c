/* ZIP-SELF: IR generator runtime — mirror of zipc/src/промежуток.rs */
#define _CRT_SECURE_NO_WARNINGS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

/* Embed parser + lexer (no mains) */
#include "парсер_rt.c"

#define MAX_ARGS 32
#define MAX_FUNCS 64
#define MAX_INSNS 4096
#define MAX_VARS 256

typedef enum {
    IR_ALLOC = 1,
    IR_LOAD = 2,
    IR_STORE = 3,
    IR_MOVE = 4,
    IR_ADD = 5,
    IR_SUB = 6,
    IR_MUL = 7,
    IR_DIV = 8,
    IR_CMP = 9,
    IR_IMPULSE = 10,
    IR_WAIT = 11,
    IR_BRANCH = 12,
    IR_START_CHOIR = 20,
    IR_JOIN_CHOIR = 21,
    IR_FIX = 30,
    IR_EXTERN = 40,
    IR_GOTO = 50,
    IR_CALL = 51,
    IR_RET = 52,
    IR_LABEL = 53,
    IR_CONST_I = 60,
    IR_CONST_F = 61,
    IR_CONST_S = 62,
    IR_PRINT = 63
} IrKind;

typedef enum {
    TY_I64 = 1,
    TY_F64 = 2,
    TY_PTR = 3,
    TY_TENSOR = 4,
    TY_PERFECT = 5,
    TY_IMPERFECT = 6
} Ty;

typedef struct {
    IrKind kind;
    int dst, a, b;
    int64_t ival;
    double fval;
    char sval[256];
    char op[8];
    int label, label_f;
    int func;
    char lib[128];
    char fname[128];
    int nargs;
    int args[MAX_ARGS];
    int has_result;
    int result;
    int print_mode; /* 0=plain 1=str 2=num */
} Insn;

typedef struct {
    int id;
    char name[128];
    int nparams;
    int params[16];
    Ty ret_ty;
    Insn *body;
    int ninsns;
    int cap;
    int async_f;
    int perfect_f;
} IrFunc;

typedef struct {
    char name[128];
    int reg;
} VarSlot;

typedef struct {
    IrFunc funcs[MAX_FUNCS];
    int nfuncs;
    int next_reg;
    int next_label;
    int next_fn;
    /* name -> fn id */
    char fn_names[MAX_FUNCS][128];
    int fn_ids[MAX_FUNCS];
    int n_fn_names;
    int reg_ty[4096];
    VarSlot vars[MAX_VARS];
    int nvars;
} Gen;

static void gen_init(Gen *G) {
    memset(G, 0, sizeof(*G));
}

static int new_reg(Gen *G, Ty ty) {
    int r = G->next_reg++;
    if (r < 4096) G->reg_ty[r] = (int)ty;
    return r;
}

static int new_label(Gen *G) { return G->next_label++; }

static int new_fn_id(Gen *G) { return G->next_fn++; }

static IrFunc *fn_by_id(Gen *G, int id) {
    for (int i = 0; i < G->nfuncs; i++)
        if (G->funcs[i].id == id) return &G->funcs[i];
    return NULL;
}

static void fn_push_insn(IrFunc *F, Insn in) {
    if (F->ninsns >= F->cap) {
        int nc = F->cap ? F->cap * 2 : 64;
        F->body = (Insn *)realloc(F->body, (size_t)nc * sizeof(Insn));
        F->cap = nc;
    }
    F->body[F->ninsns++] = in;
}

static void body_push(Insn *body, int *n, int *cap, Insn in) {
    if (*n >= *cap) {
        int nc = *cap ? *cap * 2 : 64;
        /* body is always heap in our usage via IrFunc */
        (void)nc;
    }
    /* use through IrFunc only — wrapper below */
    (void)body;
    (void)n;
    (void)cap;
    (void)in;
}

static int lookup_fn(Gen *G, const char *name) {
    for (int i = 0; i < G->n_fn_names; i++)
        if (strcmp(G->fn_names[i], name) == 0) return G->fn_ids[i];
    return -1;
}

static void bind_fn(Gen *G, const char *name, int id) {
    strncpy(G->fn_names[G->n_fn_names], name, 127);
    G->fn_ids[G->n_fn_names] = id;
    G->n_fn_names++;
}

static int ensure_verb(Gen *G, const char *name, int async_f, int perfect_f) {
    int existing = lookup_fn(G, name);
    if (existing >= 0) {
        IrFunc *F = fn_by_id(G, existing);
        if (F) {
            F->async_f |= async_f;
            F->perfect_f |= perfect_f;
            if (perfect_f) F->ret_ty = TY_PERFECT;
        }
        return existing;
    }
    int id = new_fn_id(G);
    int param = G->next_reg++;
    if (param < 4096) G->reg_ty[param] = TY_IMPERFECT;

    IrFunc *F = &G->funcs[G->nfuncs++];
    memset(F, 0, sizeof(*F));
    F->id = id;
    strncpy(F->name, name, 127);
    F->nparams = 1;
    F->params[0] = param;
    F->ret_ty = perfect_f ? TY_PERFECT : TY_IMPERFECT;
    F->async_f = async_f;
    F->perfect_f = perfect_f;
    F->cap = 16;
    F->body = (Insn *)calloc((size_t)F->cap, sizeof(Insn));

    Insn wait = {0};
    wait.kind = IR_WAIT;
    wait.a = param;
    fn_push_insn(F, wait);
    if (perfect_f) {
        Insn fx = {0};
        fx.kind = IR_FIX;
        fx.a = param;
        fn_push_insn(F, fx);
    }
    Insn ret = {0};
    ret.kind = IR_RET;
    ret.has_result = 1;
    ret.result = param;
    fn_push_insn(F, ret);

    bind_fn(G, name, id);
    return id;
}

static int lookup_var(Gen *G, const char *name) {
    for (int i = 0; i < G->nvars; i++)
        if (strcmp(G->vars[i].name, name) == 0) return G->vars[i].reg;
    return -1;
}

static void bind_var(Gen *G, const char *name, int reg) {
    int e = lookup_var(G, name);
    if (e >= 0) {
        for (int i = 0; i < G->nvars; i++)
            if (strcmp(G->vars[i].name, name) == 0) {
                G->vars[i].reg = reg;
                return;
            }
    }
    strncpy(G->vars[G->nvars].name, name, 127);
    G->vars[G->nvars].reg = reg;
    G->nvars++;
}

static int check_fixable(Gen *G, int r) {
    if (r >= 0 && r < 4096 && G->reg_ty[r] == TY_PERFECT) {
        fprintf(stderr, "Нельзя применить Совершенный к уже Совершенному регистру r%d\n", r);
        return 0;
    }
    return 1;
}

/* returns -1 for None, else register; has_val flag via out */
static int gen_node(Gen *G, Node *n, IrFunc *cur, int has_in, int in_reg, int *out_has);

static void push(IrFunc *F, Insn in) { fn_push_insn(F, in); }

static int print_mode_of(const char *name) {
    if (!name) return -1;
    if (strcmp(name, "печать") == 0 || strcmp(name, "Печать") == 0) return 0;
    if (strcmp(name, "печать_строки") == 0 || strcmp(name, "ПечатьСтроки") == 0 ||
        strcmp(name, "Печать_строки") == 0)
        return 1;
    if (strcmp(name, "печать_число") == 0 || strcmp(name, "ПечатьЧисло") == 0 ||
        strcmp(name, "Печать_число") == 0)
        return 2;
    return -1;
}

static int gen_node(Gen *G, Node *n, IrFunc *cur, int has_in, int in_reg, int *out_has) {
    *out_has = 0;
    if (!n) return 0;

    switch (n->kind) {
        case N_PROG: {
            int last = 0, has = 0;
            for (int i = 0; i < n->nkids; i++) {
                int h = 0;
                int r = gen_node(G, n->kids[i], cur, has_in, in_reg, &h);
                if (h) {
                    last = r;
                    has = 1;
                }
            }
            *out_has = has;
            return last;
        }
        case N_ROOT:
        case N_RUNTIME:
            return 0;
        case N_INT: {
            int r = new_reg(G, TY_I64);
            Insn in = {0};
            in.kind = IR_CONST_I;
            in.dst = r;
            in.ival = n->ival;
            push(cur, in);
            *out_has = 1;
            return r;
        }
        case N_FLOAT: {
            int r = new_reg(G, TY_F64);
            Insn in = {0};
            in.kind = IR_CONST_F;
            in.dst = r;
            in.fval = n->fval;
            push(cur, in);
            *out_has = 1;
            return r;
        }
        case N_STR: {
            int r = new_reg(G, TY_PTR);
            Insn in = {0};
            in.kind = IR_CONST_S;
            in.dst = r;
            strncpy(in.sval, n->sval ? n->sval : "", 255);
            push(cur, in);
            *out_has = 1;
            return r;
        }
        case N_IDENT: {
            int e = lookup_var(G, n->name ? n->name : "");
            if (e >= 0) {
                *out_has = 1;
                return e;
            }
            int r = new_reg(G, TY_I64);
            bind_var(G, n->name ? n->name : "", r);
            Insn in = {0};
            in.kind = IR_CONST_I;
            in.dst = r;
            in.ival = 0;
            push(cur, in);
            *out_has = 1;
            return r;
        }
        case N_BLOCK: {
            int last = in_reg, has = has_in;
            for (int i = 0; i < n->nkids; i++) {
                int h = 0;
                int r = gen_node(G, n->kids[i], cur, has, last, &h);
                if (h) {
                    last = r;
                    has = 1;
                }
            }
            *out_has = has;
            return last;
        }
        case N_VERB: {
            int nargs_ast = n->nkids - (n->has_next ? 1 : 0);
            int fn_id = ensure_verb(G, n->name ? n->name : "?", 0, 0);
            int args[MAX_ARGS];
            int na = 0;
            if (has_in) args[na++] = in_reg;
            for (int i = 0; i < nargs_ast; i++) {
                int h = 0;
                int r = gen_node(G, n->kids[i], cur, 0, 0, &h);
                if (h) args[na++] = r;
            }
            int result = new_reg(G, TY_IMPERFECT);
            Insn call = {0};
            call.kind = IR_CALL;
            call.func = fn_id;
            call.nargs = na;
            memcpy(call.args, args, (size_t)na * sizeof(int));
            call.has_result = 1;
            call.result = result;
            push(cur, call);
            if (n->has_next) {
                int after = new_reg(G, TY_IMPERFECT);
                Insn imp = {0};
                imp.kind = IR_IMPULSE;
                imp.a = result;
                imp.dst = after;
                push(cur, imp);
                return gen_node(G, n->kids[nargs_ast], cur, 1, after, out_has);
            }
            *out_has = 1;
            return result;
        }
        case N_FIX: {
            int fn_id = ensure_verb(G, n->name ? n->name : "?", 0, 1);
            int data_r;
            if (has_in) {
                if (!check_fixable(G, in_reg)) return 0;
                data_r = in_reg;
            } else if (n->nkids > 0) {
                int h = 0;
                int r = gen_node(G, n->kids[0], cur, 0, 0, &h);
                if (h) {
                    if (!check_fixable(G, r)) return 0;
                    data_r = r;
                } else
                    data_r = new_reg(G, TY_IMPERFECT);
            } else
                data_r = new_reg(G, TY_IMPERFECT);

            Insn call = {0};
            call.kind = IR_CALL;
            call.func = fn_id;
            call.nargs = 1;
            call.args[0] = data_r;
            call.has_result = 1;
            call.result = data_r;
            push(cur, call);
            Insn fx = {0};
            fx.kind = IR_FIX;
            fx.a = data_r;
            push(cur, fx);
            if (data_r < 4096) G->reg_ty[data_r] = TY_PERFECT;
            IrFunc *VF = fn_by_id(G, fn_id);
            if (VF) {
                VF->perfect_f = 1;
                VF->ret_ty = TY_PERFECT;
            }
            *out_has = 1;
            return data_r;
        }
        case N_TRUE:
        case N_FALSE: {
            int r = new_reg(G, TY_I64);
            Insn in = {0};
            in.kind = IR_CONST_I;
            in.dst = r;
            in.ival = (n->kind == N_TRUE) ? 1 : 0;
            push(cur, in);
            *out_has = 1;
            return r;
        }
        case N_BINOP: {
            int hl = 0, hr = 0;
            int l = gen_node(G, n->kids[0], cur, 0, 0, &hl);
            int r = gen_node(G, n->kids[1], cur, 0, 0, &hr);
            int dst = new_reg(G, TY_I64);
            Insn in = {0};
            const char *op = n->op ? n->op : "+";
            if (strcmp(op, "+") == 0) in.kind = IR_ADD;
            else if (strcmp(op, "-") == 0) in.kind = IR_SUB;
            else if (strcmp(op, "*") == 0) in.kind = IR_MUL;
            else if (strcmp(op, "/") == 0) in.kind = IR_DIV;
            else in.kind = IR_ADD;
            in.dst = dst;
            in.a = l;
            in.b = r;
            push(cur, in);
            *out_has = 1;
            return dst;
        }
        case N_CMP: {
            int hl = 0, hr = 0;
            int l = gen_node(G, n->kids[0], cur, 0, 0, &hl);
            int r = gen_node(G, n->kids[1], cur, 0, 0, &hr);
            int dst = new_reg(G, TY_I64);
            Insn in = {0};
            in.kind = IR_CMP;
            in.dst = dst;
            in.a = l;
            in.b = r;
            strncpy(in.op, n->op ? n->op : "==", 7);
            push(cur, in);
            *out_has = 1;
            return dst;
        }
        case N_ASSIGN: {
            int h = 0;
            int r = gen_node(G, n->kids[0], cur, 0, 0, &h);
            if (!h) return 0;
            int slot = lookup_var(G, n->name ? n->name : "");
            if (slot < 0) {
                slot = new_reg(G, TY_I64);
                bind_var(G, n->name ? n->name : "", slot);
            }
            Insn in = {0};
            in.kind = IR_MOVE;
            in.dst = slot;
            in.a = r;
            push(cur, in);
            *out_has = 1;
            return slot;
        }
        case N_CALL: {
            int pm = print_mode_of(n->name);
            if (pm >= 0) {
                int h = 0;
                int r = gen_node(G, n->kids[0], cur, 0, 0, &h);
                Insn in = {0};
                in.kind = IR_PRINT;
                in.a = r;
                in.print_mode = pm;
                push(cur, in);
                *out_has = h;
                return r;
            }
            int fn_id = lookup_fn(G, n->name ? n->name : "");
            if (fn_id < 0) fn_id = ensure_verb(G, n->name ? n->name : "?", 0, 0);
            int args[MAX_ARGS];
            int na = 0;
            for (int i = 0; i < n->nkids; i++) {
                int h = 0;
                int r = gen_node(G, n->kids[i], cur, 0, 0, &h);
                if (h) args[na++] = r;
            }
            int result = new_reg(G, TY_IMPERFECT);
            Insn call = {0};
            call.kind = IR_CALL;
            call.func = fn_id;
            call.nargs = na;
            memcpy(call.args, args, (size_t)na * sizeof(int));
            call.has_result = 1;
            call.result = result;
            push(cur, call);
            *out_has = 1;
            return result;
        }
        case N_AND:
        case N_OR: {
            int h = 0;
            (void)gen_node(G, n->kids[0], cur, 0, 0, &h);
            return gen_node(G, n->kids[1], cur, 0, 0, out_has);
        }
        case N_NOT:
        case N_NEG:
            return gen_node(G, n->kids[0], cur, 0, 0, out_has);
        case N_IF: {
            int h = 0;
            (void)gen_node(G, n->kids[0], cur, 0, 0, &h);
            int r = gen_node(G, n->kids[1], cur, has_in, in_reg, out_has);
            if (n->has_else) {
                int he = 0;
                (void)gen_node(G, n->kids[2], cur, has_in, in_reg, &he);
            }
            return r;
        }
        case N_WHILE: {
            int m_cond = new_label(G);
            int m_body = new_label(G);
            int m_exit = new_label(G);
            Insn g1 = {0};
            g1.kind = IR_GOTO;
            g1.label = m_cond;
            push(cur, g1);
            Insn l1 = {0};
            l1.kind = IR_LABEL;
            l1.label = m_cond;
            push(cur, l1);
            int hc = 0;
            int ru = gen_node(G, n->kids[0], cur, 0, 0, &hc);
            if (!hc) ru = new_reg(G, TY_I64);
            Insn br = {0};
            br.kind = IR_BRANCH;
            br.a = ru;
            br.label = m_body;
            br.label_f = m_exit;
            push(cur, br);
            Insn l2 = {0};
            l2.kind = IR_LABEL;
            l2.label = m_body;
            push(cur, l2);
            int hb = 0;
            (void)gen_node(G, n->kids[1], cur, has_in, in_reg, &hb);
            Insn g2 = {0};
            g2.kind = IR_GOTO;
            g2.label = m_cond;
            push(cur, g2);
            Insn l3 = {0};
            l3.kind = IR_LABEL;
            l3.label = m_exit;
            push(cur, l3);
            return 0;
        }
        case N_RETURN: {
            Insn in = {0};
            in.kind = IR_RET;
            if (n->has_return_val) {
                int h = 0;
                int r = gen_node(G, n->kids[0], cur, 0, 0, &h);
                in.has_result = h;
                in.result = r;
                *out_has = h;
                push(cur, in);
                return r;
            }
            push(cur, in);
            return 0;
        }
        case N_FN: {
            int id = new_fn_id(G);
            bind_fn(G, n->name ? n->name : "fn", id);

            /* Сохранить область видимости; аргументы VM → регистры 0..n-1 */
            VarSlot saved_vars[MAX_VARS];
            int saved_nvars = G->nvars;
            int saved_reg = G->next_reg;
            memcpy(saved_vars, G->vars, (size_t)saved_nvars * sizeof(VarSlot));
            G->nvars = 0;
            G->next_reg = 0;

            IrFunc *F = &G->funcs[G->nfuncs++];
            memset(F, 0, sizeof(*F));
            F->id = id;
            strncpy(F->name, n->name ? n->name : "fn", 127);
            F->cap = 64;
            F->body = (Insn *)calloc((size_t)F->cap, sizeof(Insn));
            F->ret_ty = TY_I64;

            if (n->nparams > 0) {
                F->nparams = n->nparams > 16 ? 16 : n->nparams;
                for (int i = 0; i < F->nparams; i++) {
                    int r = new_reg(G, TY_I64);
                    F->params[i] = r;
                    if (n->pnames && n->pnames[i])
                        bind_var(G, n->pnames[i], r);
                }
            } else {
                /* VSO: один входной импульс */
                int вход = new_reg(G, TY_IMPERFECT);
                F->nparams = 1;
                F->params[0] = вход;
            }

            int h = 0;
            int вход0 = F->nparams > 0 ? F->params[0] : 0;
            (void)gen_node(G, n->kids[0], F, F->nparams > 0, вход0, &h);
            int has_ret = 0;
            for (int i = 0; i < F->ninsns; i++)
                if (F->body[i].kind == IR_RET) has_ret = 1;
            if (!has_ret) {
                int zero = new_reg(G, TY_I64);
                Insn c0 = {0};
                c0.kind = IR_CONST_I;
                c0.dst = zero;
                c0.ival = 0;
                push(F, c0);
                Insn ret = {0};
                ret.kind = IR_RET;
                ret.has_result = 1;
                ret.result = zero;
                push(F, ret);
            }

            memcpy(G->vars, saved_vars, (size_t)saved_nvars * sizeof(VarSlot));
            G->nvars = saved_nvars;
            G->next_reg = saved_reg;
            return 0;
        }
        case N_ASYNC: {
            int fn_id = ensure_verb(G, n->name ? n->name : "?", 1, 0);
            IrFunc *VF = fn_by_id(G, fn_id);
            if (VF) {
                VF->async_f = 1;
                VF->ret_ty = TY_IMPERFECT;
            }
            int wrap_id = new_fn_id(G);
            IrFunc *W = &G->funcs[G->nfuncs++];
            memset(W, 0, sizeof(*W));
            W->id = wrap_id;
            snprintf(W->name, 127, "%s_поток", n->name ? n->name : "x");
            W->cap = 64;
            W->body = (Insn *)calloc((size_t)W->cap, sizeof(Insn));
            int вход = new_reg(G, TY_IMPERFECT);
            W->nparams = 1;
            W->params[0] = вход;
            W->ret_ty = TY_IMPERFECT;
            W->async_f = 1;
            int h = 0;
            if (n->nkids > 0) (void)gen_node(G, n->kids[0], W, 1, вход, &h);
            Insn ret = {0};
            ret.kind = IR_RET;
            ret.has_result = 1;
            ret.result = вход;
            push(W, ret);
            int result = new_reg(G, TY_IMPERFECT);
            int arg = has_in ? in_reg : result;
            Insn ch = {0};
            ch.kind = IR_START_CHOIR;
            ch.func = wrap_id;
            ch.nargs = 1;
            ch.args[0] = arg;
            push(cur, ch);
            *out_has = 1;
            return result;
        }
        case N_CHOIR: {
            int regs[MAX_ARGS];
            int nr = 0;
            for (int i = 0; i < n->nkids; i++) {
                Node *br = n->kids[i];
                if (br->kind == N_ASYNC) {
                    int wrap_id = new_fn_id(G);
                    IrFunc *W = &G->funcs[G->nfuncs++];
                    memset(W, 0, sizeof(*W));
                    W->id = wrap_id;
                    snprintf(W->name, 127, "%s_хор", br->name ? br->name : "x");
                    W->cap = 64;
                    W->body = (Insn *)calloc((size_t)W->cap, sizeof(Insn));
                    int вход = new_reg(G, TY_IMPERFECT);
                    W->nparams = 1;
                    W->params[0] = вход;
                    W->async_f = 1;
                    W->ret_ty = TY_IMPERFECT;
                    int h = 0;
                    if (br->nkids > 0) (void)gen_node(G, br->kids[0], W, 1, вход, &h);
                    Insn ret = {0};
                    ret.kind = IR_RET;
                    ret.has_result = 1;
                    ret.result = вход;
                    push(W, ret);
                    int r = new_reg(G, TY_IMPERFECT);
                    Insn ch = {0};
                    ch.kind = IR_START_CHOIR;
                    ch.func = wrap_id;
                    if (has_in) {
                        ch.nargs = 1;
                        ch.args[0] = in_reg;
                    }
                    push(cur, ch);
                    regs[nr++] = r;
                } else {
                    int h = 0;
                    int r = gen_node(G, br, cur, has_in, in_reg, &h);
                    if (h) {
                        int fn_id = new_fn_id(G);
                        IrFunc *W = &G->funcs[G->nfuncs++];
                        memset(W, 0, sizeof(*W));
                        W->id = fn_id;
                        snprintf(W->name, 127, "хор_%d", fn_id);
                        W->cap = 8;
                        W->body = (Insn *)calloc((size_t)W->cap, sizeof(Insn));
                        W->async_f = 1;
                        W->ret_ty = TY_IMPERFECT;
                        Insn ret = {0};
                        ret.kind = IR_RET;
                        ret.has_result = 1;
                        ret.result = r;
                        push(W, ret);
                        Insn ch = {0};
                        ch.kind = IR_START_CHOIR;
                        ch.func = fn_id;
                        ch.nargs = 1;
                        ch.args[0] = r;
                        push(cur, ch);
                        regs[nr++] = r;
                    }
                }
            }
            Insn join = {0};
            join.kind = IR_JOIN_CHOIR;
            join.nargs = nr;
            memcpy(join.args, regs, (size_t)nr * sizeof(int));
            push(cur, join);
            if (nr > 0) {
                *out_has = 1;
                return regs[nr - 1];
            }
            return 0;
        }
        case N_EXT: {
            int args[MAX_ARGS];
            int na = 0;
            for (int i = 0; i < n->nkids; i++) {
                int h = 0;
                int r = gen_node(G, n->kids[i], cur, 0, 0, &h);
                if (h) args[na++] = r;
            }
            int result = new_reg(G, TY_IMPERFECT);
            Insn in = {0};
            in.kind = IR_EXTERN;
            strncpy(in.lib, n->name2 ? n->name2 : "", 127);
            strncpy(in.fname, n->name ? n->name : "", 127);
            in.nargs = na;
            memcpy(in.args, args, (size_t)na * sizeof(int));
            in.has_result = 1;
            in.result = result;
            push(cur, in);
            *out_has = 1;
            return result;
        }
        case N_FOR: {
            int h = 0;
            (void)gen_node(G, n->kids[0], cur, 0, 0, &h);
            return gen_node(G, n->kids[1], cur, has_in, in_reg, out_has);
        }
        case N_BREAK:
        case N_CONTINUE:
            return 0;
        default:
            return 0;
    }
}

static int cmp_fn(const void *a, const void *b) {
    const IrFunc *fa = (const IrFunc *)a;
    const IrFunc *fb = (const IrFunc *)b;
    return fa->id - fb->id;
}

static void generate(Gen *G, Node *tree) {
    gen_init(G);
    int main_id = new_fn_id(G); /* 0 */
    IrFunc mainf;
    memset(&mainf, 0, sizeof(mainf));
    mainf.id = main_id;
    strncpy(mainf.name, "главная", 127);
    mainf.cap = 256;
    mainf.body = (Insn *)calloc((size_t)mainf.cap, sizeof(Insn));
    mainf.ret_ty = TY_IMPERFECT;

    int вход = new_reg(G, TY_IMPERFECT); /* r0 */

    if (tree->kind == N_PROG) {
        for (int i = 0; i < tree->nkids; i++) {
            int h = 0;
            int r = gen_node(G, tree->kids[i], &mainf, 1, вход, &h);
            if (h) {
                Insn mv = {0};
                mv.kind = IR_MOVE;
                mv.dst = вход;
                mv.a = r;
                push(&mainf, mv);
            }
        }
    } else {
        int h = 0;
        (void)gen_node(G, tree, &mainf, 1, вход, &h);
    }

    int exit_code = new_reg(G, TY_I64);
    Insn c0 = {0};
    c0.kind = IR_CONST_I;
    c0.dst = exit_code;
    c0.ival = 0;
    push(&mainf, c0);
    Insn ret = {0};
    ret.kind = IR_RET;
    ret.has_result = 1;
    ret.result = exit_code;
    push(&mainf, ret);

    /* Если пользователь объявил `функция главная()`, не дублируем обёртку */
    int has_user_main = 0;
    for (int i = 0; i < G->nfuncs; i++) {
        if (strcmp(G->funcs[i].name, "главная") == 0) {
            has_user_main = 1;
            break;
        }
    }
    if (!has_user_main)
        G->funcs[G->nfuncs++] = mainf;
    else {
        free(mainf.body);
    }
    qsort(G->funcs, (size_t)G->nfuncs, sizeof(IrFunc), cmp_fn);
}

/* ===== Rust Debug print ===== */
static void print_reg_list(int *args, int n) {
    printf("[");
    for (int i = 0; i < n; i++) {
        if (i) printf(", ");
        printf("Регистр(%d)", args[i]);
    }
    printf("]");
}

static void print_insn(const Insn *in) {
    switch (in->kind) {
        case IR_CONST_I:
            printf("КонстантаЦелое { назначение: Регистр(%d), значение: %lld }", in->dst,
                   (long long)in->ival);
            break;
        case IR_CONST_F:
            printf("КонстантаВещественное { назначение: Регистр(%d), значение: %g }", in->dst,
                   in->fval);
            break;
        case IR_CONST_S:
            printf("КонстантаСтрока { назначение: Регистр(%d), значение: \"%s\" }", in->dst,
                   in->sval);
            break;
        case IR_MOVE:
            printf("Переместить { назначение: Регистр(%d), источник: Регистр(%d) }", in->dst,
                   in->a);
            break;
        case IR_ADD:
            printf("Сложить { назначение: Регистр(%d), а: Регистр(%d), б: Регистр(%d) }", in->dst,
                   in->a, in->b);
            break;
        case IR_SUB:
            printf("Вычесть { назначение: Регистр(%d), а: Регистр(%d), б: Регистр(%d) }", in->dst,
                   in->a, in->b);
            break;
        case IR_MUL:
            printf("Умножить { назначение: Регистр(%d), а: Регистр(%d), б: Регистр(%d) }", in->dst,
                   in->a, in->b);
            break;
        case IR_DIV:
            printf("Разделить { назначение: Регистр(%d), а: Регистр(%d), б: Регистр(%d) }", in->dst,
                   in->a, in->b);
            break;
        case IR_CMP:
            printf("Сравнить { оператор: \"%s\", назначение: Регистр(%d), а: Регистр(%d), б: Регистр(%d) }",
                   in->op, in->dst, in->a, in->b);
            break;
        case IR_IMPULSE:
            printf("Импульс { источник: Регистр(%d), назначение: Регистр(%d) }", in->a, in->dst);
            break;
        case IR_WAIT:
            printf("ОжиданиеИмпульса { регистр: Регистр(%d) }", in->a);
            break;
        case IR_BRANCH:
            printf("ВетвлениеИмпульса { условие: Регистр(%d), истина: Метка(%d), ложь: Метка(%d) }",
                   in->a, in->label, in->label_f);
            break;
        case IR_START_CHOIR:
            printf("ЗапуститьХор { функция: ИдентификаторФункции(%d), аргументы: ", in->func);
            print_reg_list((int *)in->args, in->nargs);
            printf(" }");
            break;
        case IR_JOIN_CHOIR:
            printf("СобратьХор { регистры: ");
            print_reg_list((int *)in->args, in->nargs);
            printf(" }");
            break;
        case IR_FIX:
            printf("Зафиксировать { регистр: Регистр(%d) }", in->a);
            break;
        case IR_EXTERN:
            printf("ВнешнийВызов { библиотека: \"%s\", функция: \"%s\", аргументы: ", in->lib,
                   in->fname);
            print_reg_list((int *)in->args, in->nargs);
            if (in->has_result)
                printf(", результат: Some(Регистр(%d)) }", in->result);
            else
                printf(", результат: None }");
            break;
        case IR_CALL:
            printf("Вызвать { функция: ИдентификаторФункции(%d), аргументы: ", in->func);
            print_reg_list((int *)in->args, in->nargs);
            if (in->has_result)
                printf(", результат: Some(Регистр(%d)) }", in->result);
            else
                printf(", результат: None }");
            break;
        case IR_GOTO:
            printf("Перейти { метка: Метка(%d) }", in->label);
            break;
        case IR_RET:
            if (in->has_result)
                printf("Вернуть { значение: Some(Регистр(%d)) }", in->result);
            else
                printf("Вернуть { значение: None }");
            break;
        case IR_LABEL:
            printf("Метка { идентификатор: Метка(%d) }", in->label);
            break;
        case IR_PRINT: {
            const char *m =
                in->print_mode == 1 ? "СоСтрокой" : in->print_mode == 2 ? "Число" : "БезНовойСтроки";
            printf("Печать { режим: %s, значение: Регистр(%d) }", m, in->a);
            break;
        }
        default:
            printf("Инструкция(%d)", (int)in->kind);
            break;
    }
}

static void print_gen(Gen *G) {
    for (int i = 0; i < G->nfuncs; i++) {
        IrFunc *F = &G->funcs[i];
        printf("--- Функция: %s (async=%s, perfect=%s) ---\n", F->name,
               F->async_f ? "true" : "false", F->perfect_f ? "true" : "false");
        for (int j = 0; j < F->ninsns; j++) {
            printf("  ");
            print_insn(&F->body[j]);
            printf("\n");
        }
    }
}

#ifdef ZIP_IR_MAIN
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

    Gen G;
    generate(&G, ast);
    printf("Функций ПП: %d\n", G.nfuncs);
    print_gen(&G);

    for (int i = 0; i < L.ntoks; i++) free(L.toks[i].sval);
    free(L.toks);
    free(buf);
    return 0;
}
#endif
