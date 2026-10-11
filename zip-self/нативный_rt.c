/* ZIP-SELF: LLVM IR codegen — mirror of zipc/src/llvm_ir.rs */
#define _CRT_SECURE_NO_WARNINGS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <ctype.h>
#include <stdarg.h>

/* Embed IR gen + parser + lexer */
#include "промежуток_rt.c"

typedef struct {
    char *data;
    int len;
    int cap;
} Buf;

static void buf_init(Buf *b, int cap) {
    b->cap = cap > 0 ? cap : 4096;
    b->len = 0;
    b->data = (char *)malloc((size_t)b->cap);
    b->data[0] = 0;
}

static void buf_reserve(Buf *b, int need) {
    if (b->len + need + 1 <= b->cap) return;
    int nc = b->cap;
    while (nc < b->len + need + 1) nc *= 2;
    b->data = (char *)realloc(b->data, (size_t)nc);
    b->cap = nc;
}

static void buf_s(Buf *b, const char *s) {
    int n = (int)strlen(s);
    buf_reserve(b, n);
    memcpy(b->data + b->len, s, (size_t)n);
    b->len += n;
    b->data[b->len] = 0;
}

static void buf_fmt(Buf *b, const char *fmt, ...) {
    char tmp[1024];
    va_list ap;
    va_start(ap, fmt);
    int n = vsnprintf(tmp, sizeof(tmp), fmt, ap);
    va_end(ap);
    if (n < 0) return;
    if (n >= (int)sizeof(tmp)) {
        /* truncated — grow via heap */
        char *big = (char *)malloc((size_t)n + 1);
        va_start(ap, fmt);
        vsnprintf(big, (size_t)n + 1, fmt, ap);
        va_end(ap);
        buf_s(b, big);
        free(big);
    } else {
        buf_s(b, tmp);
    }
}

static void ascii_name(const char *in, char *out, int outn) {
    int j = 0;
    for (int i = 0; in[i] && j + 1 < outn; i++) {
        unsigned char c = (unsigned char)in[i];
        if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_')
            out[j++] = (char)c;
        else
            out[j++] = '_';
    }
    out[j] = 0;
}

static void mangled(const char *name, char *out, int outn) {
    if (strcmp(name, "главная") == 0) {
        strncpy(out, "zip_glavnaya", (size_t)outn - 1);
        out[outn - 1] = 0;
        return;
    }
    int j = 0;
    out[j++] = 'z';
    out[j++] = 'i';
    out[j++] = 'p';
    out[j++] = '_';
    const unsigned char *p = (const unsigned char *)name;
    while (*p && j + 8 < outn) {
        unsigned char c = *p;
        if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9')) {
            out[j++] = (char)c;
            p++;
        } else {
            /* UTF-8 codepoint */
            uint32_t cp;
            size_t adv = 1;
            if (c < 0x80) {
                cp = c;
                adv = 1;
            } else if ((c & 0xE0) == 0xC0) {
                cp = ((c & 0x1F) << 6) | (p[1] & 0x3F);
                adv = 2;
            } else if ((c & 0xF0) == 0xE0) {
                cp = ((c & 0x0F) << 12) | ((p[1] & 0x3F) << 6) | (p[2] & 0x3F);
                adv = 3;
            } else {
                cp = c;
                adv = 1;
            }
            j += snprintf(out + j, (size_t)(outn - j), "u%04x", cp);
            p += adv;
        }
    }
    out[j] = 0;
}

static void escape_llvm(const char *s, char *out, int outn) {
    int j = 0;
    for (const unsigned char *p = (const unsigned char *)s; *p && j + 5 < outn; p++) {
        unsigned char b = *p;
        if (b == '\\') {
            out[j++] = '\\';
            out[j++] = '\\';
        } else if (b == '"') {
            j += snprintf(out + j, (size_t)(outn - j), "\\22");
        } else if (b == '\n') {
            j += snprintf(out + j, (size_t)(outn - j), "\\0A");
        } else if (b >= 0x20 && b <= 0x7e) {
            out[j++] = (char)b;
        } else {
            j += snprintf(out + j, (size_t)(outn - j), "\\%02X", b);
        }
    }
    out[j] = 0;
}

static int max_reg_of_fn(IrFunc *F) {
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
            case IR_PRINT:
                regs[nr++] = in->a;
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
            case IR_BRANCH:
                regs[nr++] = in->a;
                break;
            case IR_RET:
                if (in->has_result) regs[nr++] = in->result;
                break;
            case IR_CALL:
                if (in->has_result) regs[nr++] = in->result;
                break;
            case IR_FIX:
            case IR_WAIT:
                regs[nr++] = in->a;
                break;
            default:
                break;
        }
        for (int k = 0; k < nr; k++)
            if (regs[k] > maxr) maxr = regs[k];
    }
    return maxr;
}

static const char *fn_llvm_name(Gen *G, int id, char *buf, int buflen) {
    IrFunc *F = fn_by_id(G, id);
    if (F && strcmp(F->name, "главная") == 0) {
        strncpy(buf, "main", (size_t)buflen - 1);
        buf[buflen - 1] = 0;
        return buf;
    }
    if (F) {
        mangled(F->name, buf, buflen);
        return buf;
    }
    snprintf(buf, (size_t)buflen, "zip_fn_%d", id);
    return buf;
}

static void collect_strings(Gen *G, char ***out, int *nout) {
    *out = NULL;
    *nout = 0;
    for (int fi = 0; fi < G->nfuncs; fi++) {
        IrFunc *F = &G->funcs[fi];
        for (int i = 0; i < F->ninsns; i++) {
            if (F->body[i].kind != IR_CONST_S) continue;
            const char *s = F->body[i].sval;
            int found = 0;
            for (int k = 0; k < *nout; k++)
                if (strcmp((*out)[k], s) == 0) {
                    found = 1;
                    break;
                }
            if (!found) {
                *out = (char **)realloc(*out, (size_t)(*nout + 1) * sizeof(char *));
                (*out)[*nout] = _strdup(s);
                (*nout)++;
            }
        }
    }
}

static int str_index(char **strs, int n, const char *s) {
    for (int i = 0; i < n; i++)
        if (strcmp(strs[i], s) == 0) return i;
    return 0;
}

static void emit_function(Buf *b, Gen *G, IrFunc *F, char **strs, int nstrs) {
    char namebuf[256];
    const char *fname;
    if (strcmp(F->name, "главная") == 0)
        fname = "main";
    else {
        mangled(F->name, namebuf, sizeof(namebuf));
        fname = namebuf;
    }

    buf_fmt(b, "define i64 @%s() {\n", fname);
    buf_s(b, "entry:\n");

    int maxr = max_reg_of_fn(F);
    for (int i = 0; i <= maxr; i++) buf_fmt(b, "  %%r%d = alloca i64, align 8\n", i);

    uint32_t tmp = 0;
#define NEXT_TMP() (tmp++)

    for (int i = 0; i < F->ninsns; i++) {
        Insn *in = &F->body[i];
        switch (in->kind) {
            case IR_CONST_I:
                buf_fmt(b, "  store i64 %lld, ptr %%r%d, align 8\n", (long long)in->ival, in->dst);
                break;
            case IR_CONST_S: {
                int idx = str_index(strs, nstrs, in->sval);
                int len = (int)strlen(in->sval) + 1;
                uint32_t t = NEXT_TMP();
                buf_fmt(b,
                        "  %%t%u = getelementptr inbounds [%d x i8], ptr @.str%d, i64 0, i64 0\n", t,
                        len, idx);
                buf_fmt(b, "  %%t%u_p = ptrtoint ptr %%t%u to i64\n  store i64 %%t%u_p, ptr %%r%d, align 8\n",
                        t, t, t, in->dst);
                break;
            }
            case IR_CONST_F: {
                union {
                    double d;
                    int64_t i;
                } u;
                u.d = in->fval;
                buf_fmt(b, "  store i64 %lld, ptr %%r%d, align 8\n", (long long)u.i, in->dst);
                break;
            }
            case IR_MOVE:
            case IR_IMPULSE: {
                uint32_t t = NEXT_TMP();
                buf_fmt(b, "  %%t%u = load i64, ptr %%r%d, align 8\n  store i64 %%t%u, ptr %%r%d, align 8\n",
                        t, in->a, t, in->dst);
                break;
            }
            case IR_ADD:
            case IR_SUB:
            case IR_MUL:
            case IR_DIV: {
                const char *op = in->kind == IR_ADD   ? "add"
                                 : in->kind == IR_SUB ? "sub"
                                 : in->kind == IR_MUL ? "mul"
                                                      : "sdiv";
                uint32_t t1 = NEXT_TMP();
                uint32_t t2 = NEXT_TMP();
                uint32_t t3 = NEXT_TMP();
                buf_fmt(b,
                        "  %%t%u = load i64, ptr %%r%d, align 8\n  %%t%u = load i64, ptr %%r%d, align 8\n  "
                        "%%t%u = %s i64 %%t%u, %%t%u\n  store i64 %%t%u, ptr %%r%d, align 8\n",
                        t1, in->a, t2, in->b, t3, op, t1, t2, t3, in->dst);
                break;
            }
            case IR_CMP: {
                const char *pred = "ne";
                if (strcmp(in->op, "==") == 0) pred = "eq";
                else if (strcmp(in->op, "!=") == 0) pred = "ne";
                else if (strcmp(in->op, "<") == 0) pred = "slt";
                else if (strcmp(in->op, ">") == 0) pred = "sgt";
                else if (strcmp(in->op, "<=") == 0) pred = "sle";
                else if (strcmp(in->op, ">=") == 0) pred = "sge";
                uint32_t t1 = NEXT_TMP();
                uint32_t t2 = NEXT_TMP();
                uint32_t t3 = NEXT_TMP();
                uint32_t t4 = NEXT_TMP();
                buf_fmt(b,
                        "  %%t%u = load i64, ptr %%r%d, align 8\n  %%t%u = load i64, ptr %%r%d, align 8\n  "
                        "%%t%u = icmp %s i64 %%t%u, %%t%u\n  %%t%u = zext i1 %%t%u to i64\n  store i64 "
                        "%%t%u, ptr %%r%d, align 8\n",
                        t1, in->a, t2, in->b, t3, pred, t1, t2, t4, t3, t4, in->dst);
                break;
            }
            case IR_PRINT: {
                uint32_t t = NEXT_TMP();
                buf_fmt(b, "  %%t%u = load i64, ptr %%r%d, align 8\n", t, in->a);
                if (in->print_mode == 1) {
                    uint32_t tp = NEXT_TMP();
                    buf_fmt(b, "  %%t%u = inttoptr i64 %%t%u to ptr\n  call i32 @puts(ptr %%t%u)\n", tp,
                            t, tp);
                } else if (in->print_mode == 0) {
                    uint32_t tp = NEXT_TMP();
                    buf_fmt(b,
                            "  %%t%u = inttoptr i64 %%t%u to ptr\n  call i32 (ptr, ...) @printf(ptr "
                            "@.fmt_str, ptr %%t%u)\n",
                            tp, t, tp);
                } else {
                    buf_fmt(b, "  call i32 (ptr, ...) @printf(ptr @.fmt_i64n, i64 %%t%u)\n", t);
                }
                break;
            }
            case IR_LABEL:
                buf_fmt(b, "L%d:\n", in->label);
                break;
            case IR_GOTO:
                buf_fmt(b, "  br label %%L%d\n", in->label);
                break;
            case IR_BRANCH: {
                uint32_t t = NEXT_TMP();
                uint32_t tc = NEXT_TMP();
                buf_fmt(b,
                        "  %%t%u = load i64, ptr %%r%d, align 8\n  %%t%u = icmp ne i64 %%t%u, 0\n  br i1 "
                        "%%t%u, label %%L%d, label %%L%d\n",
                        t, in->a, tc, t, tc, in->label, in->label_f);
                break;
            }
            case IR_RET:
                if (in->has_result) {
                    uint32_t t = NEXT_TMP();
                    buf_fmt(b, "  %%t%u = load i64, ptr %%r%d, align 8\n  ret i64 %%t%u\n", t,
                            in->result, t);
                } else {
                    buf_s(b, "  ret i64 0\n");
                }
                break;
            case IR_CALL: {
                char nb[256];
                fn_llvm_name(G, in->func, nb, sizeof(nb));
                uint32_t t = NEXT_TMP();
                buf_fmt(b, "  %%t%u = call i64 @%s()\n", t, nb);
                if (in->has_result)
                    buf_fmt(b, "  store i64 %%t%u, ptr %%r%d, align 8\n", t, in->result);
                break;
            }
            case IR_ALLOC:
                buf_fmt(b, "  store i64 0, ptr %%r%d, align 8\n", in->dst);
                break;
            case IR_FIX:
            case IR_WAIT:
            case IR_START_CHOIR:
            case IR_JOIN_CHOIR:
            case IR_EXTERN:
                buf_s(b, "  ; stub\n");
                break;
            default:
                buf_s(b, "  ; stub\n");
                break;
        }
    }

    int has_ret = 0;
    for (int i = 0; i < F->ninsns; i++)
        if (F->body[i].kind == IR_RET) has_ret = 1;
    if (!has_ret) buf_s(b, "  ret i64 0\n");
    buf_s(b, "}\n");
#undef NEXT_TMP
}

static void emit_llvm_ir(Gen *G, const char *module_name, Buf *b) {
    char mod[256];
    ascii_name(module_name, mod, sizeof(mod));
    buf_fmt(b, "; ModuleID = '%s'\n", mod);
    buf_s(b, "source_filename = \"zip\"\n");
    buf_s(b,
          "target datalayout = "
          "\"e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n");
    buf_s(b, "target triple = \"x86_64-pc-windows-msvc\"\n\n");
    buf_s(b, "declare i32 @puts(ptr)\n");
    buf_s(b, "declare i32 @printf(ptr, ...)\n\n");
    buf_s(b, "@.fmt_str = private unnamed_addr constant [3 x i8] c\"%s\\00\"\n");
    buf_s(b, "@.fmt_i64n = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\"\n\n");

    char **strs = NULL;
    int nstrs = 0;
    collect_strings(G, &strs, &nstrs);
    for (int i = 0; i < nstrs; i++) {
        char esc[1024];
        escape_llvm(strs[i], esc, sizeof(esc));
        int len = (int)strlen(strs[i]) + 1;
        buf_fmt(b, "@.str%d = private unnamed_addr constant [%d x i8] c\"%s\\00\"\n", i, len, esc);
    }
    if (nstrs) buf_s(b, "\n");

    for (int i = 0; i < G->nfuncs; i++) {
        emit_function(b, G, &G->funcs[i], strs, nstrs);
        buf_s(b, "\n");
    }

    for (int i = 0; i < nstrs; i++) free(strs[i]);
    free(strs);
}

static int write_file(const char *path, const void *data, int len) {
    FILE *f = fopen(path, "wb");
    if (!f) return -1;
    fwrite(data, 1, (size_t)len, f);
    fclose(f);
    return 0;
}

static int run_clang_ll(const char *ll, const char *exe) {
    char cmd[2048];
#ifdef _WIN32
    snprintf(cmd, sizeof(cmd),
             "clang \"%s\" -O3 -march=native -Wl,/Brepro -o \"%s\"", ll, exe);
#else
    snprintf(cmd, sizeof(cmd), "clang \"%s\" -O3 -march=native -o \"%s\"", ll, exe);
#endif
    return system(cmd);
}

static int file_exists(const char *p) {
    FILE *f = fopen(p, "rb");
    if (!f) return 0;
    fclose(f);
    return 1;
}

static int join_path(char *out, int outn, const char *a, const char *b) {
    return snprintf(out, (size_t)outn, "%s/%s", a, b) < outn;
}

/* Locate zip-self/<file> from cwd / ZIP_ROOT / input path. */
static int find_zip_self_file(const char *fname, const char *hint_path, char *out, int outn) {
    const char *env = getenv("ZIP_ROOT");
    if (env && env[0]) {
        snprintf(out, (size_t)outn, "%s/zip-self/%s", env, fname);
        if (file_exists(out)) return 1;
    }
    snprintf(out, (size_t)outn, "zip-self/%s", fname);
    if (file_exists(out)) return 1;
    snprintf(out, (size_t)outn, "%s", fname);
    if (file_exists(out)) return 1;
    if (hint_path) {
        char dir[512];
        strncpy(dir, hint_path, sizeof(dir) - 1);
        dir[sizeof(dir) - 1] = 0;
        char *slash = strrchr(dir, '/');
        char *bslash = strrchr(dir, '\\');
        char *cut = slash > bslash ? slash : bslash;
        if (cut) {
            *cut = 0;
            snprintf(out, (size_t)outn, "%s/%s", dir, fname);
            if (file_exists(out)) return 1;
            snprintf(out, (size_t)outn, "%s/../%s", dir, fname);
            if (file_exists(out)) return 1;
        }
    }
    /* walk up */
    const char *prefixes[] = {".", "..", "../..", "../../..", NULL};
    for (int i = 0; prefixes[i]; i++) {
        snprintf(out, (size_t)outn, "%s/zip-self/%s", prefixes[i], fname);
        if (file_exists(out)) return 1;
    }
    return 0;
}

static const char *find_runtime_name(Node *n) {
    if (!n) return NULL;
    if (n->kind == N_RUNTIME && n->name) return n->name;
    if (n->kind == N_PROG || n->kind == N_BLOCK) {
        for (int i = 0; i < n->nkids; i++) {
            const char *r = find_runtime_name(n->kids[i]);
            if (r) return r;
        }
    }
    return NULL;
}

static int map_runtime(const char *name, const char **c_file, const char **macro) {
    if (!name) return 0;
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
    if (strcmp(name, "parser_zip") == 0 || strcmp(name, "parser_rt") == 0 ||
        strcmp(name, "parser_ast") == 0) {
        *c_file = "парсер_rt.c";
        *macro = "ZIP_PARSER_MAIN";
        return 1;
    }
    if (strcmp(name, "ir_zip") == 0 || strcmp(name, "ir_ast") == 0 ||
        strcmp(name, "промежуток") == 0 || strcmp(name, "промежуток_rt") == 0) {
        *c_file = "промежуток_rt.c";
        *macro = "ZIP_IR_MAIN";
        return 1;
    }
    if (strcmp(name, "байткод_эмиттер_rt") == 0 || strcmp(name, "bytecode_emitter_rt") == 0) {
        *c_file = "bytecode_emitter_rt.c";
        *macro = "ZIP_BAJTKOD_MAIN";
        return 1;
    }
    if (strcmp(name, "компилятор_rt") == 0 || strcmp(name, "compiler_rt") == 0) {
        *c_file = "compiler_rt.c";
        *macro = "ZIP_COMPILER_MAIN";
        return 1;
    }
    /* ASCII aliases */
    if (strcmp(name, "native_rt") == 0) {
        *c_file = "native_rt.c";
        *macro = "ZIP_NATIVE_MAIN";
        return 1;
    }
    return 0;
}

static int compile_runtime_c(const char *c_path, const char *macro, const char *exe_path) {
    char cmd[4096];
#ifdef _WIN32
    snprintf(cmd, sizeof(cmd),
             "clang \"%s\" -O3 -march=native -D%s -Wl,/Brepro -o \"%s\"", c_path, macro,
             exe_path);
#else
    snprintf(cmd, sizeof(cmd), "clang \"%s\" -O3 -march=native -D%s -o \"%s\"", c_path, macro,
             exe_path);
#endif
    printf("  clang runtime: %s -D%s → %s\n", c_path, macro, exe_path);
    return system(cmd);
}

#ifdef ZIP_NATIVE_MAIN
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
        fprintf(stderr, "Использование: %s <вход.zip> [выход.exe] [--emit-ll]\n", argv[0]);
        return 1;
    }
    const char *in_path = argv[1];
    const char *out_path = argc >= 3 ? argv[2] : "a.out";
    int emit_only = 0;
    for (int i = 2; i < argc; i++)
        if (strcmp(argv[i], "--emit-ll") == 0 || strcmp(argv[i], "--вывод-llvm") == 0) emit_only = 1;

    /* if out looks like flag, default name */
    if (out_path[0] == '-') out_path = "a.out";

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

    Parser P = {0};
    P.toks = L.toks;
    P.ntoks = L.ntoks;
    P.pos = 0;
    Node *ast = parse_program(&P);

    char exe_path[512];
    strncpy(exe_path, out_path, sizeof(exe_path) - 1);
    exe_path[sizeof(exe_path) - 1] = 0;
#ifdef _WIN32
    {
        size_t el = strlen(exe_path);
        if (el < 4 || (strcmp(exe_path + el - 4, ".exe") != 0 && strcmp(exe_path + el - 4, ".EXE") != 0))
            strncat(exe_path, ".exe", sizeof(exe_path) - el - 1);
    }
#endif

    /* #runtime("...") → clang C-mirror (same as Rust zipc bootstrap) */
    {
        const char *rt = find_runtime_name(ast);
        const char *cfile = NULL;
        const char *macro = NULL;
        if (rt && map_runtime(rt, &cfile, &macro)) {
            char cpath[1024];
            if (!find_zip_self_file(cfile, in_path, cpath, sizeof(cpath))) {
                /* try ASCII alias for native */
                if (strcmp(macro, "ZIP_NATIVE_MAIN") == 0 &&
                    find_zip_self_file("native_rt.c", in_path, cpath, sizeof(cpath))) {
                    /* ok */
                } else {
                    fprintf(stderr, "runtime «%s»: не найден %s\n", rt, cfile);
                    return 1;
                }
            }
            if (emit_only) {
                fprintf(stderr, "runtime «%s» — нет .ll (C-зеркало)\n", rt);
                return 1;
            }
            int rc = compile_runtime_c(cpath, macro, exe_path);
            if (rc != 0) {
                fprintf(stderr, "Ошибка clang runtime (код %d)\n", rc);
                return 1;
            }
            printf("✓ Runtime «%s» → %s\n", rt, exe_path);
            for (int i = 0; i < L.ntoks; i++) free(L.toks[i].sval);
            free(L.toks);
            free(src);
            return 0;
        }
    }

    Gen G;
    generate(&G, ast);

    /* module name from input stem */
    char mod[256];
    const char *base = in_path;
    for (const char *p = in_path; *p; p++)
        if (*p == '/' || *p == '\\') base = p + 1;
    strncpy(mod, base, sizeof(mod) - 1);
    mod[sizeof(mod) - 1] = 0;
    char *dot = strrchr(mod, '.');
    if (dot) *dot = 0;

    Buf b;
    buf_init(&b, 65536);
    emit_llvm_ir(&G, mod, &b);

    char ll_path[512];
    snprintf(ll_path, sizeof(ll_path), "%s.ll", out_path);
    /* strip .exe before .ll if present */
    {
        char tmp[512];
        strncpy(tmp, out_path, sizeof(tmp) - 1);
        tmp[sizeof(tmp) - 1] = 0;
        size_t tl = strlen(tmp);
        if (tl > 4 && (strcmp(tmp + tl - 4, ".exe") == 0 || strcmp(tmp + tl - 4, ".EXE") == 0))
            tmp[tl - 4] = 0;
        snprintf(ll_path, sizeof(ll_path), "%s.ll", tmp);
    }

    if (write_file(ll_path, b.data, b.len) != 0) {
        fprintf(stderr, "Не могу записать %s\n", ll_path);
        return 1;
    }
    printf("LLVM IR: %s (%d байт)\n", ll_path, b.len);

    if (emit_only) {
        fwrite(b.data, 1, (size_t)b.len, stdout);
        return 0;
    }

    int rc = run_clang_ll(ll_path, exe_path);
    if (rc != 0) {
        fprintf(stderr, "Ошибка clang (код %d)\n", rc);
        return 1;
    }
    printf("✓ Собрано: %s\n", exe_path);

    for (int i = 0; i < L.ntoks; i++) free(L.toks[i].sval);
    free(L.toks);
    free(src);
    free(b.data);
    return 0;
}
#endif
