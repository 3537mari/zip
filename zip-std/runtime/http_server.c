/* HTTP-сервер реестра пакетов ZIP (Идея экосистемы).
 *
 * GET  /индекс
 * GET  /пакет/{имя}/{версия}
 * GET  /поиск?q=...
 * POST /пакет/{имя}/{версия}  — сохранить тело в пакеты/
 *
 * Корень реестра: env ZIP_REGISTRY_ROOT / ZIPM_REGISTRY, иначе ./реестр
 */
#define _CRT_SECURE_NO_WARNINGS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

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
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <sys/stat.h>
#define CLOSESOCKET close
#define INVALID_SOCKET (-1)
#define SOCKET_ERROR (-1)
#define MKDIR(p) mkdir(p, 0755)
typedef int SOCKET;
#endif

static int zip_net_init(void) {
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

static void registry_root(char *out, size_t n) {
    const char *env = getenv("ZIP_REGISTRY_ROOT");
    if (!env || !env[0]) env = getenv("ZIPM_REGISTRY");
    if (!env || !env[0]) env = getenv("ЗИПМ_РЕЕСТР");
    if (env && env[0] && !(env[0] == 'h' && env[1] == 't')) {
        snprintf(out, n, "%s", env);
        return;
    }
    snprintf(out, n, "реестр");
}

static int read_file(const char *path, char **data, size_t *len) {
    FILE *f = fopen(path, "rb");
    if (!f) return -1;
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (sz < 0) {
        fclose(f);
        return -1;
    }
    char *buf = (char *)malloc((size_t)sz + 1);
    if (!buf) {
        fclose(f);
        return -1;
    }
    size_t n = fread(buf, 1, (size_t)sz, f);
    fclose(f);
    buf[n] = '\0';
    *data = buf;
    *len = n;
    return 0;
}

static void http_reply(SOCKET c, int code, const char *ctype, const char *body, size_t blen) {
    char hdr[512];
    const char *status = code == 200 ? "OK" : code == 404 ? "Not Found" : code == 201 ? "Created" : "Error";
    int hn = snprintf(hdr, sizeof(hdr),
                      "HTTP/1.1 %d %s\r\nContent-Type: %s\r\nContent-Length: %zu\r\n"
                      "Access-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
                      code, status, ctype, blen);
    send(c, hdr, hn, 0);
    if (body && blen) send(c, body, (int)blen, 0);
}

static void json_escape_append(char *dst, size_t *pos, size_t cap, const char *s) {
    for (; *s && *pos + 2 < cap; s++) {
        if (*s == '"' || *s == '\\') {
            dst[(*pos)++] = '\\';
            dst[(*pos)++] = *s;
        } else if ((unsigned char)*s < 32) {
            continue;
        } else {
            dst[(*pos)++] = *s;
        }
    }
}

/* Упрощённый разбор index.toml → JSON-массив пакетов */
static void serve_index(SOCKET c, const char *root) {
    char path[1024];
    snprintf(path, sizeof(path), "%s/index.toml", root);
    char *data = NULL;
    size_t len = 0;
    if (read_file(path, &data, &len) != 0) {
        const char *empty = "{\"packages\":[],\"ok\":false}";
        http_reply(c, 200, "application/json; charset=utf-8", empty, strlen(empty));
        return;
    }
    size_t cap = len * 2 + 128;
    char *out = (char *)malloc(cap);
    if (!out) {
        free(data);
        http_reply(c, 500, "text/plain", "OOM", 3);
        return;
    }
    size_t pos = 0;
    pos += (size_t)snprintf(out + pos, 64, "{\"format\":\"toml\",\"text\":\"");
    json_escape_append(out, &pos, cap, data);
    pos += (size_t)snprintf(out + pos, 16, "\"}");
    http_reply(c, 200, "application/json; charset=utf-8", out, pos);
    free(out);
    free(data);
}

static void serve_search(SOCKET c, const char *root, const char *query) {
    /* Переиспользуем индекс и фильтруем на клиенте — здесь простая строковая фильтрация */
    char path[1024];
    snprintf(path, sizeof(path), "%s/index.toml", root);
    char *data = NULL;
    size_t len = 0;
    if (read_file(path, &data, &len) != 0) {
        const char *empty = "{\"результаты\":[]}";
        http_reply(c, 200, "application/json; charset=utf-8", empty, strlen(empty));
        return;
    }
    char *out = (char *)malloc(len + 128);
    size_t pos = 0;
    pos += (size_t)snprintf(out + pos, 64, "{\"результаты\":[");
    int first = 1;
    char *p = data;
    while ((p = strstr(p, "[пакеты.")) != NULL) {
        p += 9;
        char name[256];
        size_t i = 0;
        while (*p && *p != ']' && i + 1 < sizeof(name)) name[i++] = *p++;
        name[i] = 0;
        int match = 1;
        if (query && query[0]) {
            match = 0;
            for (char *a = name, *b = (char *)query; *a && *b;) {
                /* substring case-sensitive */
                if (strstr(name, query)) {
                    match = 1;
                    break;
                }
                break;
            }
            if (strstr(name, query)) match = 1;
        }
        if (match) {
            if (!first) out[pos++] = ',';
            first = 0;
            pos += (size_t)snprintf(out + pos, 64, "{\"имя\":\"");
            json_escape_append(out, &pos, len + 100, name);
            pos += (size_t)snprintf(out + pos, 8, "\"}");
        }
    }
    pos += (size_t)snprintf(out + pos, 8, "]}");
    http_reply(c, 200, "application/json; charset=utf-8", out, pos);
    free(out);
    free(data);
}

static void serve_package_get(SOCKET c, const char *root, const char *name, const char *ver) {
    char path[1024];
    snprintf(path, sizeof(path), "%s/пакеты/%s/%s/зипм.манифест", root, name, ver);
    char *data = NULL;
    size_t len = 0;
    if (read_file(path, &data, &len) != 0) {
        snprintf(path, sizeof(path), "%s/пакеты/%s/%s/checksum.sha256", root, name, ver);
        if (read_file(path, &data, &len) != 0) {
            const char *msg = "{\"ошибка\":\"пакет не найден\"}";
            http_reply(c, 404, "application/json; charset=utf-8", msg, strlen(msg));
            return;
        }
    }
    /* Отдаём манифест (+ метаданные). Полный tar.gz — следующий этап. */
    http_reply(c, 200, "text/plain; charset=utf-8", data, len);
    free(data);
}

static void serve_package_post(SOCKET c, const char *root, const char *name, const char *ver,
                               const char *body, size_t blen) {
    char dir[1024], path[1100];
    snprintf(dir, sizeof(dir), "%s/пакеты/%s/%s", root, name, ver);
    /* mkdir -p упрощённо */
    char tmp[1024];
    snprintf(tmp, sizeof(tmp), "%s/пакеты", root);
    MKDIR(tmp);
    snprintf(tmp, sizeof(tmp), "%s/пакеты/%s", root, name);
    MKDIR(tmp);
    MKDIR(dir);
    snprintf(path, sizeof(path), "%s/payload.bin", dir);
    FILE *f = fopen(path, "wb");
    if (!f) {
        http_reply(c, 500, "text/plain", "cannot write", 12);
        return;
    }
    if (blen) fwrite(body, 1, blen, f);
    fclose(f);
    const char *ok = "{\"статус\":\"опубликовано\"}";
    http_reply(c, 201, "application/json; charset=utf-8", ok, strlen(ok));
}

static void handle_client(SOCKET c, const char *root) {
    char buf[65536];
    int n = recv(c, buf, (int)sizeof(buf) - 1, 0);
    if (n <= 0) {
        CLOSESOCKET(c);
        return;
    }
    buf[n] = '\0';

    char method[16] = {0};
    char url[1024] = {0};
    sscanf(buf, "%15s %1023s", method, url);

    char *body = strstr(buf, "\r\n\r\n");
    size_t blen = 0;
    if (body) {
        body += 4;
        blen = (size_t)(n - (body - buf));
    }

    if (strcmp(method, "GET") == 0 && (strcmp(url, "/индекс") == 0 || strcmp(url, "/index") == 0)) {
        serve_index(c, root);
    } else if (strcmp(method, "GET") == 0 && strncmp(url, "/поиск", 7) == 0) {
        const char *q = strstr(url, "q=");
        serve_search(c, root, q ? q + 2 : "");
    } else if (strcmp(method, "GET") == 0 && strncmp(url, "/search", 7) == 0) {
        const char *q = strstr(url, "q=");
        serve_search(c, root, q ? q + 2 : "");
    } else if ((strcmp(method, "GET") == 0 || strcmp(method, "POST") == 0) &&
               (strncmp(url, "/пакет/", 8) == 0 || strncmp(url, "/package/", 9) == 0)) {
        const char *p = strstr(url, "/пакет/") ? url + 8 : url + 9;
        char name[256] = {0}, ver[64] = {0};
        sscanf(p, "%255[^/]/%63s", name, ver);
        /* strip query */
        char *qq = strchr(ver, '?');
        if (qq) *qq = 0;
        if (strcmp(method, "GET") == 0)
            serve_package_get(c, root, name, ver);
        else
            serve_package_post(c, root, name, ver, body ? body : "", blen);
    } else if (strcmp(method, "GET") == 0 && (strcmp(url, "/") == 0 || strcmp(url, "/health") == 0)) {
        const char *msg = "{\"сервис\":\"реестр-zip\",\"статус\":\"ok\"}";
        http_reply(c, 200, "application/json; charset=utf-8", msg, strlen(msg));
    } else {
        const char *msg = "{\"ошибка\":\"не найдено\"}";
        http_reply(c, 404, "application/json; charset=utf-8", msg, strlen(msg));
    }
    CLOSESOCKET(c);
}

int zip_слушать_порт(int порт) {
    if (zip_net_init() != 0) return -1;
    char root[512];
    registry_root(root, sizeof(root));

    SOCKET sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (sock == INVALID_SOCKET) return -1;
    int yes = 1;
    setsockopt(sock, SOL_SOCKET, SO_REUSEADDR, (const char *)&yes, sizeof(yes));

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_ANY);
    addr.sin_port = htons((unsigned short)порт);

    if (bind(sock, (struct sockaddr *)&addr, sizeof(addr)) == SOCKET_ERROR) {
        CLOSESOCKET(sock);
        return -2;
    }
    if (listen(sock, 16) == SOCKET_ERROR) {
        CLOSESOCKET(sock);
        return -3;
    }

    printf("Реестр ZIP слушает http://localhost:%d\n", порт);
    printf("Корень: %s\n", root);
    printf("API: GET /индекс | GET /пакет/{имя}/{версия} | GET /поиск?q= | POST /пакет/...\n");

    for (;;) {
        SOCKET client = accept(sock, NULL, NULL);
        if (client == INVALID_SOCKET) continue;
        handle_client(client, root);
    }
}

#ifdef ZIP_HTTP_SERVER_MAIN
int main(int argc, char **argv) {
    int port = 8080;
    if (argc > 1) port = atoi(argv[1]);
    if (port <= 0) port = 8080;
    return zip_слушать_порт(port) == 0 ? 0 : 1;
}
#endif
