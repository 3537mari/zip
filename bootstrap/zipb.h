#ifndef ZIPB_H
#define ZIPB_H

#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <stdbool.h>

#define ZIPB_MAGIC "ZIPB"
#define ZIPB_VERSION 3
#define ZIPC_BOOTSTRAP_VERSION "3.1.0"
#define ZIP_MAX_LOCALS 256
#define ZIP_MAX_FUNCS 256
#define ZIP_MAX_STRINGS 4096

typedef enum {
    OP_NOP=0x00, OP_POP=0x01, OP_DUP=0x02, OP_SWAP=0x03,
    OP_PUSH_I64=0x10, OP_PUSH_F64=0x11, OP_PUSH_STR=0x12,
    OP_PUSH_NULL=0x13, OP_PUSH_TRUE=0x14, OP_PUSH_FALSE=0x15,
    OP_ADD=0x20, OP_SUB=0x21, OP_MUL=0x22, OP_DIV=0x23, OP_MOD=0x24,
    OP_EQ=0x30, OP_NE=0x31, OP_LT=0x32, OP_LE=0x33, OP_GT=0x34, OP_GE=0x35,
    OP_LOAD=0x40, OP_STORE=0x41,
    OP_JUMP=0x50, OP_JUMP_IF=0x51, OP_JUMP_IF_NOT=0x52,
    OP_CALL=0x60, OP_RET=0x61, OP_CALL_NATIVE=0x62,
    OP_PULSE=0x70, OP_PERFECT_LOCK=0x71,
    OP_CHOIR_SPAWN=0x72, OP_CHOIR_JOIN=0x73, OP_ASYNC_CALL=0x74,
    OP_NEW_SET=0x80, OP_NEW_ITER=0x81, OP_NEW_NOTHING=0x82,
    OP_NEW_SOMETHING=0x83, OP_NEW_SUCCESS=0x84, OP_NEW_ERROR=0x85,
    OP_CHECK_TAG=0x86,
    OP_PRINT=0x90, OP_PRINTLN=0x91, OP_PRINT_I64=0x92,
    /* v6: структуры */
    OP_NEW_STRUCT=0x93, OP_LOAD_FIELD=0x94, OP_STORE_FIELD=0x95,
    OP_GET_FIELD_PTR=0x96,
    /* v6: массивы */
    OP_NEW_ARRAY=0x97, OP_LOAD_INDEX=0x98, OP_STORE_INDEX=0x99,
    OP_ARRAY_LEN=0x9A,
    /* v6: указатели */
    OP_ADDR_OF=0x9B, OP_DEREF=0x9C, OP_ALLOC=0x9D, OP_FREE=0x9E,
    OP_PTR_ADD=0x9F, OP_PTR_SUB=0xA0, OP_PTR_DIFF=0xA1,
    OP_LOAD_PTR=0xA2, OP_STORE_PTR=0xA3
} Opcode;

typedef enum {
    TK_EOF=0,
    TK_IMPULSE, TK_CHOIR, TK_NET, TK_PERFECT, TK_STREAM,
    TK_EQ, TK_COLON, TK_LPAREN, TK_RPAREN, TK_LBRACE, TK_RBRACE,
    TK_LBRACK, TK_RBRACK,
    TK_COMMA, TK_ARROW, TK_RANGE, TK_PLUS, TK_MINUS, TK_STAR, TK_SLASH, TK_PERCENT,
    TK_LT, TK_LE, TK_GT, TK_GE, TK_EQEQ, TK_NE,
    TK_INT, TK_FLOAT, TK_STRING, TK_IDENT, TK_ROOT, TK_TEMPLATE,
    TK_IF, TK_ELSE, TK_WHILE, TK_FOR, TK_IN, TK_RETURN, TK_BREAK, TK_CONTINUE,
    TK_FUNCTION, TK_TYPE, TK_VERB, TK_AGENT, TK_FLOW,
    TK_TRUE, TK_FALSE, TK_AND, TK_OR, TK_NOT,
    TK_NOTHING, TK_SOMETHING, TK_SUCCESS, TK_ERROR,
    TK_AT, TK_HASH,
    TK_STRUCT,
    TK_DOT,
    TK_SEMI,
    TK_AMP
} TokenType;

typedef struct {
    TokenType type;
    int64_t i64;
    double f64;
    char *str;
    int line, col;
} Token;

typedef enum {
    NODE_PROGRAM=1, NODE_FUNCTION, NODE_PARAM, NODE_CALL,
    NODE_CONST_I64, NODE_CONST_F64, NODE_CONST_STR, NODE_CONST_BOOL,
    NODE_IDENT, NODE_BINOP, NODE_UNARY, NODE_ASSIGN,
    NODE_IF, NODE_WHILE, NODE_FOR, NODE_RETURN, NODE_BREAK, NODE_CONTINUE,
    NODE_BLOCK, NODE_ANNOT, NODE_DIRECTIVE,
    NODE_PRINT_STR, NODE_PRINT_I64, NODE_PERFECT, NODE_PULSE, NODE_CHOIR,
    NODE_STRUCT, NODE_FIELD,
    NODE_STRUCT_LIT, NODE_FIELD_ACCESS, NODE_FIELD_STORE,
    NODE_ARRAY_TYPE, NODE_ARRAY_LIT, NODE_INDEX_ACCESS, NODE_INDEX_STORE,
    NODE_ADDR_OF, NODE_DEREF, NODE_ALLOC, NODE_FREE, NODE_STORE_PTR
} NodeType;

typedef struct Node {
    NodeType type;
    char *name;
    char *type_name;
    int64_t i64;
    double f64;
    char *str;
    int op;
    int arity;
    struct Node **kids;
    int n, cap;
} Node;

typedef struct {
    uint8_t *data;
    size_t len, cap;
} Buf;

void *zip_alloc(size_t n);
void *zip_realloc(void *p, size_t n);
char *zip_strdup(const char *s);
int zip_streq(const char *a, const char *b);

Buf *buf_new(void);
void buf_free(Buf *b);
void buf_byte(Buf *b, uint8_t v);
void buf_u16(Buf *b, uint16_t v);
void buf_u32(Buf *b, uint32_t v);
void buf_i64(Buf *b, int64_t v);
void buf_f64(Buf *b, double v);
void buf_bytes(Buf *b, const void *p, size_t n);
void buf_patch_u32(Buf *b, size_t at, uint32_t v);

Node *node_new(NodeType t);
void node_add(Node *parent, Node *kid);
void node_free(Node *n);

Token *lex(const char *src, size_t *out_n);
void tokens_free(Token *t, size_t n);
Node *parse(Token *toks, size_t n);
Buf *codegen(Node *ast);

const char *token_name(TokenType t);
TokenType keyword_lookup(const char *w);

int validate_ast(Node *ast);
void print_ast(Node *n, int indent);
void dump_zipb(const uint8_t *data, size_t len, FILE *out);
int dump_file(const char *path);

#endif
