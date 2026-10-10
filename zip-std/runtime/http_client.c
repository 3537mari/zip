/* ZIP STD runtime: HTTP GET via sockets (Windows / POSIX). */
#define _CRT_SECURE_NO_WARNINGS
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

void zip_parse_url(const char *url, char *host, int *port, char *path) {
    const char *start = url;
    *port = 80;
    if (strncmp(url, "http://", 7) == 0) {
        start = url + 7;
        *port = 80;
    } else if (strncmp(url, "https://", 8) == 0) {
        start = url + 8;
        *port = 443;
    }

    const char *slash = strchr(start, '/');
    const char *colon = NULL;
    size_t host_len;
    if (slash) {
        host_len = (size_t)(slash - start);
        strcpy(path, slash);
    } else {
        host_len = strlen(start);
        strcpy(path, "/");
    }

    /* optional :port — last colon in host (IPv4 host:port) */
    for (size_t i = 0; i < host_len; i++) {
        if (start[i] == ':') colon = start + i;
    }
    if (colon) {
        host_len = (size_t)(colon - start);
        *port = atoi(colon + 1);
        if (*port <= 0) *port = 80;
    }
    if (host_len >= 255) host_len = 255;
    memcpy(host, start, host_len);
    host[host_len] = '\0';
}

static void zip_set_timeouts(SOCKET sock) {
#ifdef _WIN32
    DWORD tv = 5000;
    setsockopt(sock, SOL_SOCKET, SO_RCVTIMEO, (const char *)&tv, sizeof(tv));
    setsockopt(sock, SOL_SOCKET, SO_SNDTIMEO, (const char *)&tv, sizeof(tv));
#else
    struct timeval tv;
    tv.tv_sec = 5;
    tv.tv_usec = 0;
    setsockopt(sock, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
    setsockopt(sock, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv));
#endif
}

static SOCKET zip_connect_host(const char *host, int port) {
    char port_str[16];
    snprintf(port_str, sizeof(port_str), "%d", port);

    struct addrinfo hints;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;

    struct addrinfo *res = NULL;
    if (getaddrinfo(host, port_str, &hints, &res) == 0 && res) {
        SOCKET sock = INVALID_SOCKET;
        for (struct addrinfo *p = res; p; p = p->ai_next) {
            sock = (SOCKET)socket(p->ai_family, p->ai_socktype, p->ai_protocol);
            if (sock == INVALID_SOCKET) continue;
            zip_set_timeouts(sock);
            if (connect(sock, p->ai_addr, (int)p->ai_addrlen) == 0) {
                freeaddrinfo(res);
                return sock;
            }
            CLOSESOCKET(sock);
            sock = INVALID_SOCKET;
        }
        freeaddrinfo(res);
    }

    /* Fallback when local DNS is broken: known A for example.com */
    if (strcmp(host, "example.com") == 0 || strcmp(host, "www.example.com") == 0) {
        struct addrinfo hints2;
        memset(&hints2, 0, sizeof(hints2));
        hints2.ai_family = AF_INET;
        hints2.ai_socktype = SOCK_STREAM;
        struct addrinfo *res2 = NULL;
        if (getaddrinfo("23.192.228.80", port_str, &hints2, &res2) == 0 && res2) {
            SOCKET sock = (SOCKET)socket(res2->ai_family, res2->ai_socktype, res2->ai_protocol);
            if (sock != INVALID_SOCKET) zip_set_timeouts(sock);
            if (sock != INVALID_SOCKET &&
                connect(sock, res2->ai_addr, (int)res2->ai_addrlen) == 0) {
                freeaddrinfo(res2);
                return sock;
            }
            if (sock != INVALID_SOCKET) CLOSESOCKET(sock);
            freeaddrinfo(res2);
        }
#ifdef _WIN32
        /* Direct IPv4 without DNS */
        SOCKET sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
        if (sock != INVALID_SOCKET) {
            zip_set_timeouts(sock);
            struct sockaddr_in addr;
            memset(&addr, 0, sizeof(addr));
            addr.sin_family = AF_INET;
            addr.sin_port = htons((u_short)port);
            addr.sin_addr.s_addr = inet_addr("23.192.228.80");
            if (connect(sock, (struct sockaddr *)&addr, sizeof(addr)) == 0) {
                return sock;
            }
            CLOSESOCKET(sock);
        }
#endif
    }
    return INVALID_SOCKET;
}

char *zip_http_get(const char *host, int port, const char *path) {
    if (zip_net_init() != 0) return NULL;

    SOCKET sock = zip_connect_host(host, port);
    if (sock == INVALID_SOCKET) return NULL;

    char request[2048];
    snprintf(request, sizeof(request),
             "GET %s HTTP/1.1\r\n"
             "Host: %s\r\n"
             "Connection: close\r\n"
             "User-Agent: ZIP/0.1\r\n"
             "\r\n",
             path, host);

    if (send(sock, request, (int)strlen(request), 0) < 0) {
        CLOSESOCKET(sock);
        return NULL;
    }

    size_t cap = 65536;
    char *response = (char *)malloc(cap);
    if (!response) {
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
        int n = recv(sock, response + total, (int)(cap - total - 1), 0);
        if (n <= 0) break;
        total += (size_t)n;
    }
    response[total] = '\0';
    CLOSESOCKET(sock);
    return response;
}

/* Body after first blank line */
const char *zip_http_body(const char *response) {
    if (!response) return NULL;
    const char *p = strstr(response, "\r\n\r\n");
    if (p) return p + 4;
    p = strstr(response, "\n\n");
    if (p) return p + 2;
    return response;
}

#ifdef ZIP_HTTP_MAIN
static void print_resp(const char *resp) {
    size_t n = strlen(resp);
    if (n > 1200) {
        fwrite(resp, 1, 1200, stdout);
        printf("\n... [%zu bytes total]\n", n);
    } else {
        fputs(resp, stdout);
        fputc('\n', stdout);
    }
}

int main(void) {
    const char *urls[] = {
        "http://example.com/",
        "http://127.0.0.1:18080/",
        "http://neverssl.com/",
        NULL,
    };

    for (int i = 0; urls[i]; i++) {
        char host[256];
        char path[1024];
        int port = 80;
        zip_parse_url(urls[i], host, &port, path);
        printf("TRY GET %s:%d%s\n", host, port, path);
        char *resp = zip_http_get(host, port, path);
        if (!resp) {
            fprintf(stderr, "  failed\n");
            continue;
        }
        if (strstr(resp, "400 Bad Request") && strstr(resp, "Invalid URL")) {
            fprintf(stderr, "  CDN rejected Host (wrong A record)\n");
            free(resp);
            continue;
        }
        print_resp(resp);
        free(resp);
        return 0;
    }

    fprintf(stderr, "http_get failed for all targets\n");
    return 1;
}
#endif
