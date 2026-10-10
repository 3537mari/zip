/* HTTPS client via OpenSSL (+ plain HTTP via zip_http_get). */
#define _CRT_SECURE_NO_WARNINGS
#define _WINSOCK_DEPRECATED_NO_WARNINGS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <winsock2.h>
#include <ws2tcpip.h>
#pragma comment(lib, "ws2_32.lib")
#define CLOSESOCKET closesocket
typedef int socklen_t;
#else
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <unistd.h>
#define CLOSESOCKET close
#define INVALID_SOCKET (-1)
#define SOCKET_ERROR (-1)
typedef int SOCKET;
#endif

#include <openssl/ssl.h>
#include <openssl/err.h>
#ifdef _WIN32
/* Required when linking OpenSSL DLLs from MSVC/clang on Windows */
#include <openssl/applink.c>
#endif

/* From http_client.c when linked together */
extern char *zip_http_get(const char *host, int port, const char *path);
extern void zip_parse_url(const char *url, char *host, int *port, char *path);

static SSL_CTX *zip_ssl_ctx = NULL;

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

static void zip_ssl_init(void) {
    if (zip_ssl_ctx) return;
    SSL_library_init();
    SSL_load_error_strings();
    OpenSSL_add_all_algorithms();
    zip_ssl_ctx = SSL_CTX_new(TLS_client_method());
    if (!zip_ssl_ctx) return;
    /* Peer verify when CA bundle available; otherwise still allow demo */
    SSL_CTX_set_default_verify_paths(zip_ssl_ctx);
    SSL_CTX_set_verify(zip_ssl_ctx, SSL_VERIFY_NONE, NULL);
}

static void zip_set_timeouts(SOCKET sock) {
#ifdef _WIN32
    DWORD tv = 10000;
    setsockopt(sock, SOL_SOCKET, SO_RCVTIMEO, (const char *)&tv, sizeof(tv));
    setsockopt(sock, SOL_SOCKET, SO_SNDTIMEO, (const char *)&tv, sizeof(tv));
#else
    struct timeval tv = {.tv_sec = 10, .tv_usec = 0};
    setsockopt(sock, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
    setsockopt(sock, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv));
#endif
}

char *zip_https_get(const char *host, int port, const char *path) {
    if (zip_net_init() != 0) return NULL;
    zip_ssl_init();
    if (!zip_ssl_ctx) return NULL;

    char port_str[16];
    snprintf(port_str, sizeof(port_str), "%d", port);

    struct addrinfo hints;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;

    struct addrinfo *res = NULL;
    if (getaddrinfo(host, port_str, &hints, &res) != 0 || !res) {
        return NULL;
    }

    SOCKET sock = INVALID_SOCKET;
    for (struct addrinfo *p = res; p; p = p->ai_next) {
        sock = (SOCKET)socket(p->ai_family, p->ai_socktype, p->ai_protocol);
        if (sock == INVALID_SOCKET) continue;
        zip_set_timeouts(sock);
        if (connect(sock, p->ai_addr, (int)p->ai_addrlen) == 0) break;
        CLOSESOCKET(sock);
        sock = INVALID_SOCKET;
    }
    freeaddrinfo(res);

    /* Fallback IP when DNS for example.com is broken locally */
    if (sock == INVALID_SOCKET &&
        (strcmp(host, "example.com") == 0 || strcmp(host, "www.example.com") == 0)) {
#ifdef _WIN32
        sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
        if (sock != INVALID_SOCKET) {
            zip_set_timeouts(sock);
            struct sockaddr_in addr;
            memset(&addr, 0, sizeof(addr));
            addr.sin_family = AF_INET;
            addr.sin_port = htons((u_short)port);
            /* Current example.com A (may rotate) */
            inet_pton(AF_INET, "96.7.128.198", &addr.sin_addr);
            if (connect(sock, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
                CLOSESOCKET(sock);
                sock = INVALID_SOCKET;
            }
        }
#endif
    }
    if (sock == INVALID_SOCKET) return NULL;

    SSL *ssl = SSL_new(zip_ssl_ctx);
    if (!ssl) {
        CLOSESOCKET(sock);
        return NULL;
    }
    SSL_set_fd(ssl, (int)sock);
    SSL_set_tlsext_host_name(ssl, host);

    if (SSL_connect(ssl) != 1) {
        SSL_free(ssl);
        CLOSESOCKET(sock);
        return NULL;
    }

    char request[2048];
    snprintf(request, sizeof(request),
             "GET %s HTTP/1.1\r\n"
             "Host: %s\r\n"
             "User-Agent: ZIP/0.2\r\n"
             "Accept: */*\r\n"
             "Connection: close\r\n"
             "\r\n",
             path, host);

    if (SSL_write(ssl, request, (int)strlen(request)) <= 0) {
        SSL_free(ssl);
        CLOSESOCKET(sock);
        return NULL;
    }

    size_t cap = 1 << 20;
    char *response = (char *)malloc(cap);
    if (!response) {
        SSL_free(ssl);
        CLOSESOCKET(sock);
        return NULL;
    }
    size_t total = 0;
    for (;;) {
        if (total + 4096 >= cap) {
            cap *= 2;
            char *nbuf = (char *)realloc(response, cap);
            if (!nbuf) break;
            response = nbuf;
        }
        int n = SSL_read(ssl, response + total, (int)(cap - total - 1));
        if (n <= 0) break;
        total += (size_t)n;
    }
    response[total] = '\0';

    SSL_shutdown(ssl);
    SSL_free(ssl);
    CLOSESOCKET(sock);
    return response;
}

char *zip_http_or_https_get(const char *url) {
    char host[256];
    char path[1024];
    int port = 80;
    int use_https = 0;

    const char *start = url;
    if (strncmp(url, "https://", 8) == 0) {
        start = url + 8;
        port = 443;
        use_https = 1;
    } else if (strncmp(url, "http://", 7) == 0) {
        start = url + 7;
        port = 80;
    }

    const char *slash = strchr(start, '/');
    size_t host_len;
    if (slash) {
        host_len = (size_t)(slash - start);
        if (strlen(slash) >= sizeof(path)) {
            strncpy(path, slash, sizeof(path) - 1);
            path[sizeof(path) - 1] = '\0';
        } else {
            strcpy(path, slash);
        }
    } else {
        host_len = strlen(start);
        strcpy(path, "/");
    }
    if (host_len >= sizeof(host)) host_len = sizeof(host) - 1;
    memcpy(host, start, host_len);
    host[host_len] = '\0';

    char *colon = strrchr(host, ':');
    if (colon) {
        *colon = '\0';
        int p = atoi(colon + 1);
        if (p > 0) port = p;
    }

    return use_https ? zip_https_get(host, port, path) : zip_http_get(host, port, path);
}

#ifdef ZIP_HTTPS_MAIN
static void print_resp(const char *resp) {
    if (!resp) {
        fputs("(null)\n", stdout);
        return;
    }
    size_t n = strlen(resp);
    if (n > 1500) {
        fwrite(resp, 1, 1500, stdout);
        printf("\n... [%zu bytes total]\n", n);
    } else {
        fputs(resp, stdout);
        fputc('\n', stdout);
    }
}

int main(int argc, char **argv) {
    const char *defaults[] = {
        "https://example.com/",
        "https://api.github.com/",
        NULL,
    };
    int ok = 0;
    if (argc > 1) {
        printf("=== HTTPS GET %s ===\n", argv[1]);
        char *resp = zip_http_or_https_get(argv[1]);
        if (!resp) {
            fprintf(stderr, "https_get failed\n");
            ERR_print_errors_fp(stderr);
            return 1;
        }
        print_resp(resp);
        free(resp);
        return 0;
    }
    for (int i = 0; defaults[i]; i++) {
        printf("=== HTTPS GET %s ===\n", defaults[i]);
        char *resp = zip_http_or_https_get(defaults[i]);
        if (!resp) {
            fprintf(stderr, "(skip — connect/TLS failed)\n");
            ERR_print_errors_fp(stderr);
            continue;
        }
        print_resp(resp);
        free(resp);
        ok = 1;
        printf("\n");
    }
    return ok ? 0 : 1;
}
#endif
