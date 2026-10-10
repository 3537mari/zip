/* ZIP STD runtime: GEMM with blocked optimization (clang -O3 / vectorize). */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>
#include "alloc.h"

#ifndef ZIP_N
#define ZIP_N 512
#endif

static void gemm_naive(const double *A, const double *B, double *C, int M, int K, int N) {
    for (int i = 0; i < M; i++) {
        for (int j = 0; j < N; j++) {
            double sum = 0.0;
            for (int l = 0; l < K; l++) {
                sum += A[i * K + l] * B[l * N + j];
            }
            C[i * N + j] = sum;
        }
    }
}

static void gemm_blocked(const double *A, const double *B, double *C, int M, int K, int N, int BS) {
    /* ikj order: keep C row hot, good for auto-vectorize on j */
    (void)BS;
    memset(C, 0, (size_t)M * (size_t)N * sizeof(double));
    for (int i = 0; i < M; i++) {
        for (int l = 0; l < K; l++) {
            double a = A[i * K + l];
            const double *brow = &B[l * N];
            double *crow = &C[i * N];
            for (int j = 0; j < N; j++) {
                crow[j] += a * brow[j];
            }
        }
    }
}

#ifdef ZIP_BENCH_MAIN
int main(void) {
    const int n = ZIP_N;
    size_t elems = (size_t)n * (size_t)n;
    double *A = (double *)zip_aligned_alloc(64, elems * sizeof(double));
    double *B = (double *)zip_aligned_alloc(64, elems * sizeof(double));
    double *C = (double *)zip_aligned_alloc(64, elems * sizeof(double));
    if (!A || !B || !C) {
        fprintf(stderr, "alloc failed\n");
        return 1;
    }
    for (size_t i = 0; i < elems; i++) {
        A[i] = 1.0;
        B[i] = 2.0;
    }

    clock_t t0 = clock();
    gemm_blocked(A, B, C, n, n, n, 32);
    clock_t t1 = clock();
    double sec = (double)(t1 - t0) / CLOCKS_PER_SEC;

    /* Expected C[i,j] = 2*n for A=1, B=2 */
    double expect = 2.0 * (double)n;
    int ok = (C[0] == expect && C[elems - 1] == expect);

    printf("512x512 umnozhenie zaversheno\n");
    printf("ZIP_GEMM_sec: ");
    /* print milliseconds as integer for ZIP-like print_число compat */
    long ms = (long)(sec * 1000.0);
    printf("%ld\n", ms);
    printf("checksum_ok: %d\n", ok);
    printf("C00: %.1f\n", C[0]);

    zip_aligned_free(A);
    zip_aligned_free(B);
    zip_aligned_free(C);
    (void)gemm_naive;
    return ok ? 0 : 2;
}
#endif
