#include "zipb.h"

void *zip_alloc(size_t n) {
    void *p = calloc(1, n ? n : 1);
    if (!p) { fprintf(stderr, "zip: OOM %zu\n", n); exit(1); }
    return p;
}
void *zip_realloc(void *p, size_t n) {
    void *q = realloc(p, n ? n : 1);
    if (!q) { fprintf(stderr, "zip: realloc OOM %zu\n", n); exit(1); }
    return q;
}
char *zip_strdup(const char *s) {
    if (!s) return NULL;
    size_t n = strlen(s);
    char *d = (char *)zip_alloc(n + 1);
    memcpy(d, s, n + 1);
    return d;
}
int zip_streq(const char *a, const char *b) {
    if (!a || !b) return a == b;
    return strcmp(a, b) == 0;
}

Buf *buf_new(void) {
    Buf *b = (Buf *)zip_alloc(sizeof(Buf));
    b->cap = 8192;
    b->data = (uint8_t *)zip_alloc(b->cap);
    return b;
}
void buf_free(Buf *b) {
    if (!b) return;
    free(b->data);
    free(b);
}
static void grow(Buf *b, size_t need) {
    while (b->len + need > b->cap) {
        b->cap *= 2;
        b->data = (uint8_t *)zip_realloc(b->data, b->cap);
    }
}
void buf_byte(Buf *b, uint8_t v) { grow(b, 1); b->data[b->len++] = v; }
void buf_u16(Buf *b, uint16_t v) {
    grow(b, 2);
    b->data[b->len++] = (uint8_t)(v & 0xFF);
    b->data[b->len++] = (uint8_t)((v >> 8) & 0xFF);
}
void buf_u32(Buf *b, uint32_t v) {
    grow(b, 4);
    b->data[b->len++] = (uint8_t)(v & 0xFF);
    b->data[b->len++] = (uint8_t)((v >> 8) & 0xFF);
    b->data[b->len++] = (uint8_t)((v >> 16) & 0xFF);
    b->data[b->len++] = (uint8_t)((v >> 24) & 0xFF);
}
void buf_i64(Buf *b, int64_t v) {
    grow(b, 8);
    for (int i = 0; i < 8; i++) b->data[b->len++] = (uint8_t)((v >> (8 * i)) & 0xFF);
}
void buf_f64(Buf *b, double v) {
    union { double d; uint64_t u; } x; x.d = v;
    buf_i64(b, (int64_t)x.u);
}
void buf_bytes(Buf *b, const void *p, size_t n) {
    grow(b, n);
    memcpy(b->data + b->len, p, n);
    b->len += n;
}
void buf_patch_u32(Buf *b, size_t at, uint32_t v) {
    if (at + 4 > b->len) return;
    b->data[at] = (uint8_t)(v & 0xFF);
    b->data[at+1] = (uint8_t)((v >> 8) & 0xFF);
    b->data[at+2] = (uint8_t)((v >> 16) & 0xFF);
    b->data[at+3] = (uint8_t)((v >> 24) & 0xFF);
}

Node *node_new(NodeType t) {
    Node *n = (Node *)zip_alloc(sizeof(Node));
    n->type = t;
    n->cap = 4;
    n->kids = (Node **)zip_alloc(n->cap * sizeof(Node *));
    return n;
}
void node_add(Node *parent, Node *kid) {
    if (!parent || !kid) return;
    if (parent->n >= parent->cap) {
        parent->cap *= 2;
        parent->kids = (Node **)zip_realloc(parent->kids, parent->cap * sizeof(Node *));
    }
    parent->kids[parent->n++] = kid;
}
void node_free(Node *n) {
    if (!n) return;
    for (int i = 0; i < n->n; i++) node_free(n->kids[i]);
    free(n->kids);
    free(n->name);
    free(n->type_name);
    free(n->str);
    free(n);
}
