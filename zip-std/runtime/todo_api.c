/* ZIP TODO REST API — production demo (JSON file DB + Bearer tokens).
 *
 * GET    /health
 * POST   /api/вход          {"имя","пароль"}
 * GET    /api/задачи        Authorization: Bearer <token>
 * POST   /api/задачи        {"заголовок","описание"}
 * GET    /api/задачи/{id}
 * PUT    /api/задачи/{id}   {"заголовок"?, "описание"?, "выполнена"?}
 * DELETE /api/задачи/{id}
 */
#define _CRT_SECURE_NO_WARNINGS
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#ifdef _WIN32
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <winsock2.h>
#include <ws2tcpip.h>
#include <direct.h>
#pragma comment(lib, "ws2_32.lib")
#define CLOSESOCKET closesocket
#define MKDIR(p) _mkdir(p)
typedef int socklen_t;
#else
#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <unistd.h>
#define CLOSESOCKET close
#define INVALID_SOCKET (-1)
#define SOCKET_ERROR (-1)
#define MKDIR(p) mkdir(p, 0755)
typedef int SOCKET;
#endif

#define MAX_TASKS 256
#define MAX_TOKENS 64
#define MAX_TITLE 256
#define MAX_DESC 1024
#define MAX_TOKEN 80

typedef struct {
    int id;
    char title[MAX_TITLE];
    char desc[MAX_DESC];
    int done;
    long long created;
    long long updated;
} Task;

typedef struct {
    Task tasks[MAX_TASKS];
    int n_tasks;
    int next_id;
    char tokens[MAX_TOKENS][MAX_TOKEN];
    int n_tokens;
    char db_path[512];
} Store;

static Store G;

static int net_init(void) {
#ifdef _WIN32
    static int ready = 0;
    if (!ready) {
        WSADATA wsa;
        if (WSAStartup(MAKEWORD(2, 2), &wsa) != 0) return -1;
        ready = 1;
    }
#endif
    return 0;
}

static long long now_ms(void) {
    return (long long)time(NULL) * 1000;
}

static void log_line(const char *msg) {
    fprintf(stderr, "[todo] %s\n", msg);
    fflush(stderr);
}

static void http_reply(SOCKET c, int code, const char *body) {
    char hdr[512];
    const char *status = code == 200   ? "OK"
                         : code == 201 ? "Created"
                         : code == 401 ? "Unauthorized"
                         : code == 404 ? "Not Found"
                         : code == 400 ? "Bad Request"
                                       : "Error";
    size_t blen = body ? strlen(body) : 0;
    int hn = snprintf(hdr, sizeof(hdr),
                      "HTTP/1.1 %d %s\r\nContent-Type: application/json; charset=utf-8\r\n"
                      "Content-Length: %zu\r\nAccess-Control-Allow-Origin: *\r\n"
                      "Access-Control-Allow-Headers: Authorization, Content-Type\r\n"
                      "Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n"
                      "Connection: close\r\n\r\n",
                      code, status, blen);
    send(c, hdr, hn, 0);
    if (body && blen) send(c, body, (int)blen, 0);
}

static void json_escape(const char *s, char *out, size_t cap) {
    size_t p = 0;
    for (; *s && p + 2 < cap; s++) {
        if (*s == '"' || *s == '\\') {
            out[p++] = '\\';
            out[p++] = *s;
        } else if ((unsigned char)*s < 32) {
            continue;
        } else {
            out[p++] = *s;
        }
    }
    out[p] = 0;
}

static int extract_str(const char *json, const char *key, char *out, size_t cap) {
    char pat[128];
    snprintf(pat, sizeof(pat), "\"%s\"", key);
    const char *p = strstr(json, pat);
    if (!p) return -1;
    p = strchr(p + strlen(pat), ':');
    if (!p) return -1;
    p++;
    while (*p && isspace((unsigned char)*p)) p++;
    if (*p != '"') return -1;
    p++;
    size_t i = 0;
    while (*p && *p != '"' && i + 1 < cap) {
        if (*p == '\\' && p[1]) p++;
        out[i++] = *p++;
    }
    out[i] = 0;
    return 0;
}

static int extract_bool(const char *json, const char *key, int *out) {
    char pat[128];
    snprintf(pat, sizeof(pat), "\"%s\"", key);
    const char *p = strstr(json, pat);
    if (!p) return -1;
    p = strchr(p + strlen(pat), ':');
    if (!p) return -1;
    p++;
    while (*p && isspace((unsigned char)*p)) p++;
    if (strncmp(p, "true", 4) == 0) {
        *out = 1;
        return 0;
    }
    if (strncmp(p, "false", 5) == 0) {
        *out = 0;
        return 0;
    }
    return -1;
}

static void task_to_json(const Task *t, char *out, size_t cap) {
    char te[MAX_TITLE * 2], de[MAX_DESC * 2];
    json_escape(t->title, te, sizeof(te));
    json_escape(t->desc, de, sizeof(de));
    snprintf(out, cap,
             "{\"id\":%d,\"заголовок\":\"%s\",\"описание\":\"%s\",\"выполнена\":%s,"
             "\"создана\":%lld,\"обновлена\":%lld}",
             t->id, te, de, t->done ? "true" : "false", t->created, t->updated);
}

static int save_db(void) {
    FILE *f = fopen(G.db_path, "wb");
    if (!f) {
        log_line("ошибка записи БД");
        return -1;
    }
    fprintf(f, "{\n  \"следующий_id\": %d,\n  \"задачи\": [\n", G.next_id);
    for (int i = 0; i < G.n_tasks; i++) {
        char buf[2048];
        task_to_json(&G.tasks[i], buf, sizeof(buf));
        fprintf(f, "    %s%s\n", buf, i + 1 < G.n_tasks ? "," : "");
    }
    fprintf(f, "  ]\n}\n");
    fclose(f);
    return 0;
}

static void load_db(void) {
    G.n_tasks = 0;
    G.next_id = 1;
    G.n_tokens = 0;
    FILE *f = fopen(G.db_path, "rb");
    if (!f) {
        save_db();
        return;
    }
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (sz <= 0 || sz > 4 * 1024 * 1024) {
        fclose(f);
        return;
    }
    char *buf = (char *)malloc((size_t)sz + 1);
    if (!buf) {
        fclose(f);
        return;
    }
    size_t n = fread(buf, 1, (size_t)sz, f);
    fclose(f);
    buf[n] = 0;

    const char *p = strstr(buf, "\"следующий_id\"");
    if (p) {
        p = strchr(p, ':');
        if (p) G.next_id = atoi(p + 1);
        if (G.next_id < 1) G.next_id = 1;
    }

    p = strstr(buf, "\"задачи\"");
    if (!p) {
        free(buf);
        return;
    }
    p = strchr(p, '[');
    if (!p) {
        free(buf);
        return;
    }
    p++;
    while (*p && G.n_tasks < MAX_TASKS) {
        while (*p && *p != '{') p++;
        if (!*p) break;
        const char *end = strchr(p, '}');
        if (!end) break;
        char obj[4096];
        size_t len = (size_t)(end - p + 1);
        if (len >= sizeof(obj)) len = sizeof(obj) - 1;
        memcpy(obj, p, len);
        obj[len] = 0;

        Task *t = &G.tasks[G.n_tasks];
        memset(t, 0, sizeof(*t));
        const char *id = strstr(obj, "\"id\"");
        if (id) {
            id = strchr(id, ':');
            if (id) t->id = atoi(id + 1);
        }
        extract_str(obj, "заголовок", t->title, sizeof(t->title));
        extract_str(obj, "описание", t->desc, sizeof(t->desc));
        extract_bool(obj, "выполнена", &t->done);
        const char *c = strstr(obj, "\"создана\"");
        if (c) {
            c = strchr(c, ':');
            if (c) t->created = atoll(c + 1);
        }
        const char *u = strstr(obj, "\"обновлена\"");
        if (u) {
            u = strchr(u, ':');
            if (u) t->updated = atoll(u + 1);
        }
        if (t->id > 0) {
            G.n_tasks++;
            if (t->id >= G.next_id) G.next_id = t->id + 1;
        }
        p = end + 1;
    }
    free(buf);
}

static void make_token(const char *user, char *out, size_t cap) {
    unsigned h = 2166136261u;
    long long t = now_ms();
    char raw[256];
    snprintf(raw, sizeof(raw), "%s:%lld:%u", user, t, (unsigned)rand());
    for (const char *p = raw; *p; p++) {
        h ^= (unsigned char)*p;
        h *= 16777619u;
    }
    snprintf(out, cap, "%08x%08x%08x", h, (unsigned)(t & 0xffffffffu),
             (unsigned)((t >> 17) ^ h));
}

static int token_ok(const char *tok) {
    if (!tok || !tok[0]) return 0;
    for (int i = 0; i < G.n_tokens; i++) {
        if (strcmp(G.tokens[i], tok) == 0) return 1;
    }
    return 0;
}

static int find_task(int id) {
    for (int i = 0; i < G.n_tasks; i++)
        if (G.tasks[i].id == id) return i;
    return -1;
}

static void list_json(char *out, size_t cap) {
    size_t pos = 0;
    pos += (size_t)snprintf(out + pos, cap - pos, "[");
    for (int i = 0; i < G.n_tasks && pos + 8 < cap; i++) {
        if (i) pos += (size_t)snprintf(out + pos, cap - pos, ",");
        char one[2048];
        task_to_json(&G.tasks[i], one, sizeof(one));
        pos += (size_t)snprintf(out + pos, cap - pos, "%s", one);
    }
    snprintf(out + pos, cap - pos, "]");
}

static const char *header_value(const char *req, const char *name) {
    char line[128];
    snprintf(line, sizeof(line), "\r\n%s:", name);
    const char *p = strstr(req, line);
    if (!p) {
        /* case-insensitive Authorization */
        p = strstr(req, "\r\nauthorization:");
        if (!p) p = strstr(req, "\r\nAuthorization:");
        if (!p) return NULL;
        p = strchr(p + 2, ':');
    } else {
        p = strchr(p + 2, ':');
    }
    if (!p) return NULL;
    p++;
    while (*p == ' ') p++;
    return p;
}

static void get_bearer(const char *req, char *out, size_t cap) {
    out[0] = 0;
    const char *h = header_value(req, "Authorization");
    if (!h) return;
    if (strncmp(h, "Bearer ", 7) == 0 || strncmp(h, "bearer ", 7) == 0) h += 7;
    size_t i = 0;
    while (h[i] && h[i] != '\r' && h[i] != '\n' && i + 1 < cap) {
        out[i] = h[i];
        i++;
    }
    out[i] = 0;
}

static int hexv(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return -1;
}

static void url_decode(char *s) {
    char *r = s, *w = s;
    while (*r) {
        if (*r == '%' && hexv(r[1]) >= 0 && hexv(r[2]) >= 0) {
            *w++ = (char)((hexv(r[1]) << 4) | hexv(r[2]));
            r += 3;
        } else if (*r == '+') {
            *w++ = ' ';
            r++;
        } else {
            *w++ = *r++;
        }
    }
    *w = 0;
}

static int path_is(const char *url, const char *a, const char *b) {
    return strcmp(url, a) == 0 || (b && strcmp(url, b) == 0);
}

static int path_prefix(const char *url, const char *a, const char *b, const char **rest) {
    size_t la = strlen(a);
    if (strncmp(url, a, la) == 0) {
        *rest = url + la;
        return 1;
    }
    if (b) {
        size_t lb = strlen(b);
        if (strncmp(url, b, lb) == 0) {
            *rest = url + lb;
            return 1;
        }
    }
    return 0;
}

static void handle(SOCKET c, const char *req, int n) {
    char method[16] = {0};
    char url[1024] = {0};
    sscanf(req, "%15s %1023s", method, url);
    char *q = strchr(url, '?');
    if (q) *q = 0;
    url_decode(url);

    char *body = strstr(req, "\r\n\r\n");
    if (body) body += 4;
    else body = "";

    char logbuf[256];
    snprintf(logbuf, sizeof(logbuf), "%s %s", method, url);
    log_line(logbuf);

    if (strcmp(method, "OPTIONS") == 0) {
        http_reply(c, 200, "{}");
        return;
    }

    if (strcmp(method, "GET") == 0 && strcmp(url, "/health") == 0) {
        http_reply(c, 200, "{\"статус\":\"ok\",\"сервис\":\"todo_сервис\"}");
        return;
    }

    if (strcmp(method, "POST") == 0 && path_is(url, "/api/вход", "/api/login")) {
        char name[128] = {0}, pass[128] = {0};
        extract_str(body, "имя", name, sizeof(name));
        extract_str(body, "пароль", pass, sizeof(pass));
        if (strcmp(name, "admin") == 0 && strcmp(pass, "admin") == 0) {
            if (G.n_tokens >= MAX_TOKENS) {
                http_reply(c, 500, "{\"ошибка\":\"слишком много сессий\"}");
                return;
            }
            make_token(name, G.tokens[G.n_tokens], MAX_TOKEN);
            char resp[256];
            snprintf(resp, sizeof(resp), "{\"токен\":\"%s\"}", G.tokens[G.n_tokens]);
            G.n_tokens++;
            log_line("вход успешен");
            http_reply(c, 200, resp);
            return;
        }
        http_reply(c, 401, "{\"ошибка\":\"Неверные данные\"}");
        return;
    }

    char token[MAX_TOKEN];
    get_bearer(req, token, sizeof(token));
    int authed = token_ok(token);

    const char *id_rest = NULL;
    int is_tasks = path_is(url, "/api/задачи", "/api/tasks");
    int is_task_id = path_prefix(url, "/api/задачи/", "/api/tasks/", &id_rest);

    if (is_tasks || is_task_id) {
        if (!authed) {
            http_reply(c, 401, "{\"ошибка\":\"Не авторизован\"}");
            return;
        }
    }

    if (strcmp(method, "GET") == 0 && is_tasks) {
        char *buf = (char *)malloc(512 * 1024);
        if (!buf) {
            http_reply(c, 500, "{\"ошибка\":\"память\"}");
            return;
        }
        list_json(buf, 512 * 1024);
        http_reply(c, 200, buf);
        free(buf);
        return;
    }

    if (strcmp(method, "POST") == 0 && is_tasks) {
        if (G.n_tasks >= MAX_TASKS) {
            http_reply(c, 500, "{\"ошибка\":\"лимит задач\"}");
            return;
        }
        Task *t = &G.tasks[G.n_tasks];
        memset(t, 0, sizeof(*t));
        t->id = G.next_id++;
        if (extract_str(body, "заголовок", t->title, sizeof(t->title)) != 0 || !t->title[0]) {
            http_reply(c, 400, "{\"ошибка\":\"нужен заголовок\"}");
            return;
        }
        extract_str(body, "описание", t->desc, sizeof(t->desc));
        t->done = 0;
        t->created = t->updated = now_ms();
        G.n_tasks++;
        save_db();
        char one[2048];
        task_to_json(t, one, sizeof(one));
        http_reply(c, 201, one);
        return;
    }

    if (is_task_id) {
        int id = atoi(id_rest);
        int idx = find_task(id);
        if (strcmp(method, "GET") == 0) {
            if (idx < 0) {
                http_reply(c, 404, "{\"ошибка\":\"Не найдено\"}");
                return;
            }
            char one[2048];
            task_to_json(&G.tasks[idx], one, sizeof(one));
            http_reply(c, 200, one);
            return;
        }
        if (strcmp(method, "PUT") == 0) {
            if (idx < 0) {
                http_reply(c, 404, "{\"ошибка\":\"Не найдено\"}");
                return;
            }
            Task *t = &G.tasks[idx];
            char tmp[MAX_TITLE];
            if (extract_str(body, "заголовок", tmp, sizeof(tmp)) == 0 && tmp[0])
                snprintf(t->title, sizeof(t->title), "%s", tmp);
            if (extract_str(body, "описание", tmp, sizeof(tmp)) == 0)
                snprintf(t->desc, sizeof(t->desc), "%s", tmp);
            int d;
            if (extract_bool(body, "выполнена", &d) == 0) t->done = d;
            t->updated = now_ms();
            save_db();
            char one[2048];
            task_to_json(t, one, sizeof(one));
            http_reply(c, 200, one);
            return;
        }
        if (strcmp(method, "DELETE") == 0) {
            if (idx < 0) {
                http_reply(c, 404, "{\"ошибка\":\"Не найдено\"}");
                return;
            }
            for (int i = idx; i + 1 < G.n_tasks; i++) G.tasks[i] = G.tasks[i + 1];
            G.n_tasks--;
            save_db();
            http_reply(c, 200, "{\"статус\":\"удалено\"}");
            return;
        }
    }

    (void)n;
    http_reply(c, 404, "{\"ошибка\":\"Не найдено\"}");
}

static int listen_port(int port) {
    if (net_init() != 0) return -1;
    SOCKET sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (sock == INVALID_SOCKET) return -1;
    int yes = 1;
    setsockopt(sock, SOL_SOCKET, SO_REUSEADDR, (const char *)&yes, sizeof(yes));
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_ANY);
    addr.sin_port = htons((unsigned short)port);
    if (bind(sock, (struct sockaddr *)&addr, sizeof(addr)) == SOCKET_ERROR) {
        CLOSESOCKET(sock);
        return -2;
    }
    if (listen(sock, 32) == SOCKET_ERROR) {
        CLOSESOCKET(sock);
        return -3;
    }
    printf("=== TODO-СЕРВИС ZIP ===\n");
    printf("Сервис запущен на http://localhost:%d\n", port);
    printf("API:\n");
    printf("  POST   /api/вход | /api/login\n");
    printf("  GET    /api/задачи | /api/tasks\n");
    printf("  POST   /api/задачи | /api/tasks\n");
    printf("  GET    /api/задачи/{id} | /api/tasks/{id}\n");
    printf("  PUT    /api/задачи/{id} | /api/tasks/{id}\n");
    printf("  DELETE /api/задачи/{id} | /api/tasks/{id}\n");
    printf("  GET    /health\n");
    printf("БД: %s\n", G.db_path);
    fflush(stdout);

    for (;;) {
        SOCKET client = accept(sock, NULL, NULL);
        if (client == INVALID_SOCKET) continue;
        char buf[65536];
        int rn = recv(client, buf, (int)sizeof(buf) - 1, 0);
        if (rn > 0) {
            buf[rn] = 0;
            handle(client, buf, rn);
        }
        CLOSESOCKET(client);
    }
}

static void resolve_db_path(void) {
    const char *env = getenv("ZIP_TODO_DB");
    if (env && env[0]) {
        snprintf(G.db_path, sizeof(G.db_path), "%s", env);
        return;
    }
    /* рядом с cwd: данные/задачи.json */
    MKDIR("данные");
    snprintf(G.db_path, sizeof(G.db_path), "данные/задачи.json");
}

#ifdef ZIP_TODO_MAIN
int main(int argc, char **argv) {
    srand((unsigned)time(NULL));
    resolve_db_path();
    load_db();
    int port = 8080;
    if (argc > 1) port = atoi(argv[1]);
    if (port <= 0) port = 8080;
    return listen_port(port) == 0 ? 0 : 1;
}
#endif

#ifdef ZIP_TODO_TEST
/* Автотест хранилища без сети */
int main(void) {
    int fails = 0;
    srand(1);
    snprintf(G.db_path, sizeof(G.db_path), "данные/тест_задачи.json");
    MKDIR("данные");
    remove(G.db_path);
    load_db();
    if (G.n_tasks != 0) {
        fprintf(stderr, "fail empty\n");
        fails++;
    }
    Task *t = &G.tasks[G.n_tasks++];
    memset(t, 0, sizeof(*t));
    t->id = G.next_id++;
    snprintf(t->title, sizeof(t->title), "тест");
    snprintf(t->desc, sizeof(t->desc), "описание");
    t->created = t->updated = 1;
    save_db();
    int n = G.n_tasks;
    load_db();
    if (G.n_tasks != n || strcmp(G.tasks[0].title, "тест") != 0) {
        fprintf(stderr, "fail reload\n");
        fails++;
    }
    make_token("admin", G.tokens[0], MAX_TOKEN);
    G.n_tokens = 1;
    if (!token_ok(G.tokens[0]) || token_ok("bad")) {
        fprintf(stderr, "fail token\n");
        fails++;
    }
    remove(G.db_path);
    if (fails) {
        fprintf(stderr, "%d fails\n", fails);
        return 1;
    }
    printf("todo storage tests ok\n");
    return 0;
}
#endif
