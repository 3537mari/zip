#include "zipb.h"

typedef struct { const char *w; TokenType t; } Kw;

static const Kw KW[] = {
    {"если", TK_IF}, {"иначе", TK_ELSE}, {"пока", TK_WHILE}, {"для", TK_FOR},
    {"в", TK_IN}, {"возврат", TK_RETURN}, {"прервать", TK_BREAK},
    {"продолжить", TK_CONTINUE}, {"функция", TK_FUNCTION}, {"тип", TK_TYPE},
    {"глагол", TK_VERB}, {"агент", TK_AGENT}, {"поток", TK_FLOW},
    {"истина", TK_TRUE}, {"ложь", TK_FALSE}, {"и", TK_AND}, {"или", TK_OR},
    {"не", TK_NOT}, {"ничто", TK_NOTHING}, {"Ничто", TK_NOTHING},
    {"нечто", TK_SOMETHING}, {"Нечто", TK_SOMETHING},
    {"успех", TK_SUCCESS}, {"Успех", TK_SUCCESS},
    {"ошибка", TK_ERROR}, {"Ошибка", TK_ERROR},
    {"структура", TK_STRUCT},
    {NULL, TK_EOF}
};

TokenType keyword_lookup(const char *w) {
    for (int i = 0; KW[i].w; i++)
        if (strcmp(KW[i].w, w) == 0) return KW[i].t;
    return TK_IDENT;
}

const char *token_name(TokenType t) {
    switch (t) {
    case TK_EOF: return "EOF";
    case TK_IMPULSE: return ">>";
    case TK_CHOIR: return "||";
    case TK_NET: return ">>>";
    case TK_PERFECT: return "!";
    case TK_STREAM: return "~";
    case TK_EQ: return "=";
    case TK_EQEQ: return "==";
    case TK_NE: return "!=";
    case TK_LT: return "<";
    case TK_LE: return "<=";
    case TK_GT: return ">";
    case TK_GE: return ">=";
    case TK_PLUS: return "+";
    case TK_MINUS: return "-";
    case TK_STAR: return "*";
    case TK_SLASH: return "/";
    case TK_PERCENT: return "%";
    case TK_RANGE: return "..";
    case TK_ARROW: return "->";
    case TK_FUNCTION: return "функция";
    case TK_IF: return "если";
    case TK_ELSE: return "иначе";
    case TK_WHILE: return "пока";
    case TK_FOR: return "для";
    case TK_RETURN: return "возврат";
    case TK_INT: return "int";
    case TK_FLOAT: return "float";
    case TK_STRING: return "string";
    case TK_IDENT: return "ident";
    case TK_STRUCT: return "структура";
    case TK_DOT: return ".";
    default: return "?";
    }
}
