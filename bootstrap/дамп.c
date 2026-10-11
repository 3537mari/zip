#include "zipb.h"

/* Disassembler / dump helpers for ZIPB v3 (bootstrap tooling). */

static const char *op_name(uint8_t op) {
    switch (op) {
    case OP_NOP: return "NOP";
    case OP_POP: return "POP";
    case OP_DUP: return "DUP";
    case OP_SWAP: return "SWAP";
    case OP_PUSH_I64: return "PUSH_I64";
    case OP_PUSH_F64: return "PUSH_F64";
    case OP_PUSH_STR: return "PUSH_STR";
    case OP_PUSH_NULL: return "PUSH_NULL";
    case OP_PUSH_TRUE: return "PUSH_TRUE";
    case OP_PUSH_FALSE: return "PUSH_FALSE";
    case OP_ADD: return "ADD";
    case OP_SUB: return "SUB";
    case OP_MUL: return "MUL";
    case OP_DIV: return "DIV";
    case OP_MOD: return "MOD";
    case OP_EQ: return "EQ";
    case OP_NE: return "NE";
    case OP_LT: return "LT";
    case OP_LE: return "LE";
    case OP_GT: return "GT";
    case OP_GE: return "GE";
    case OP_LOAD: return "LOAD";
    case OP_STORE: return "STORE";
    case OP_JUMP: return "JUMP";
    case OP_JUMP_IF: return "JUMP_IF";
    case OP_JUMP_IF_NOT: return "JUMP_IF_NOT";
    case OP_CALL: return "CALL";
    case OP_RET: return "RET";
    case OP_CALL_NATIVE: return "CALL_NATIVE";
    case OP_PULSE: return "PULSE";
    case OP_PERFECT_LOCK: return "PERFECT_LOCK";
    case OP_CHOIR_SPAWN: return "CHOIR_SPAWN";
    case OP_CHOIR_JOIN: return "CHOIR_JOIN";
    case OP_ASYNC_CALL: return "ASYNC_CALL";
    case OP_NEW_SET: return "NEW_SET";
    case OP_NEW_ITER: return "NEW_ITER";
    case OP_NEW_NOTHING: return "NEW_NOTHING";
    case OP_NEW_SOMETHING: return "NEW_SOMETHING";
    case OP_NEW_SUCCESS: return "NEW_SUCCESS";
    case OP_NEW_ERROR: return "NEW_ERROR";
    case OP_CHECK_TAG: return "CHECK_TAG";
    case OP_PRINT: return "PRINT";
    case OP_PRINTLN: return "PRINTLN";
    case OP_PRINT_I64: return "PRINT_I64";
    default: return "???";
    }
}

static uint32_t rd_u32(const uint8_t *p) {
    return (uint32_t)p[0] | ((uint32_t)p[1] << 8) | ((uint32_t)p[2] << 16) | ((uint32_t)p[3] << 24);
}
static uint16_t rd_u16(const uint8_t *p) {
    return (uint16_t)p[0] | ((uint16_t)p[1] << 8);
}
static int64_t rd_i64(const uint8_t *p) {
    uint64_t u = 0;
    for (int i = 0; i < 8; i++) u |= ((uint64_t)p[i]) << (8 * i);
    return (int64_t)u;
}

void dump_zipb(const uint8_t *data, size_t len, FILE *out) {
    if (!out) out = stdout;
    if (len < 16 || memcmp(data, "ZIPB", 4) != 0) {
        fprintf(out, "not a ZIPB file\n");
        return;
    }
    uint16_t ver = rd_u16(data + 4);
    uint16_t flags = rd_u16(data + 6);
    uint32_t nfn = rd_u32(data + 8);
    uint32_t stroff = rd_u32(data + 12);
    fprintf(out, "ZIPB v%u flags=%u funcs=%u str_off=%u size=%zu\n",
            ver, flags, nfn, stroff, len);

    if (stroff >= len) return;
    size_t p = stroff;
    uint32_t nstr = rd_u32(data + p); p += 4;
    fprintf(out, "strings: %u\n", nstr);
    for (uint32_t i = 0; i < nstr && p + 4 <= len; i++) {
        uint32_t L = rd_u32(data + p); p += 4;
        if (p + L > len) break;
        fprintf(out, "  [%u] \"", i);
        fwrite(data + p, 1, L, out);
        fprintf(out, "\"\n");
        p += L;
    }

    for (uint32_t f = 0; f < nfn && p + 16 <= len; f++) {
        uint32_t name_i = rd_u32(data + p); p += 4;
        uint16_t arity = rd_u16(data + p); p += 2;
        uint16_t locals = rd_u16(data + p); p += 2;
        uint32_t fflags = rd_u32(data + p); p += 4;
        uint32_t clen = rd_u32(data + p); p += 4;
        fprintf(out, "func[%u] name=%u arity=%u locals=%u flags=%u code=%u\n",
                f, name_i, arity, locals, fflags, clen);
        if (p + clen > len) break;
        size_t end = p + clen;
        uint32_t ip = 0;
        while (p < end) {
            uint8_t op = data[p++];
            fprintf(out, "  %04u  %s", ip, op_name(op));
            switch (op) {
            case OP_PUSH_I64:
                if (p + 8 <= end) { fprintf(out, " %lld", (long long)rd_i64(data + p)); p += 8; }
                break;
            case OP_PUSH_F64:
                if (p + 8 <= end) { p += 8; fprintf(out, " <f64>"); }
                break;
            case OP_PUSH_STR:
            case OP_LOAD:
            case OP_STORE:
            case OP_JUMP:
            case OP_JUMP_IF:
            case OP_JUMP_IF_NOT:
            case OP_CALL:
            case OP_CHOIR_SPAWN:
            case OP_ASYNC_CALL:
                if (p + 4 <= end) { fprintf(out, " %u", rd_u32(data + p)); p += 4; }
                break;
            case OP_CALL_NATIVE:
                if (p + 8 <= end) {
                    fprintf(out, " lib=%u fun=%u", rd_u32(data + p), rd_u32(data + p + 4));
                    p += 8;
                }
                break;
            case OP_PULSE:
                if (p + 4 <= end) {
                    fprintf(out, " %u->%u", rd_u16(data + p), rd_u16(data + p + 2));
                    p += 4;
                }
                break;
            case OP_PERFECT_LOCK:
                if (p + 2 <= end) { fprintf(out, " %u", rd_u16(data + p)); p += 2; }
                break;
            default:
                break;
            }
            fprintf(out, "\n");
            ip++;
        }
    }
}

/* Optional CLI hook: zipc --dump file.zipb */
int dump_file(const char *path) {
    FILE *f = fopen(path, "rb");
    if (!f) { fprintf(stderr, "cannot open %s\n", path); return 1; }
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    uint8_t *buf = (uint8_t *)zip_alloc((size_t)sz);
    fread(buf, 1, (size_t)sz, f);
    fclose(f);
    dump_zipb(buf, (size_t)sz, stdout);
    free(buf);
    return 0;
}
