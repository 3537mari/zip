#pragma once
#include <stdlib.h>

/* Portable fallback: malloc is enough for GEMM correctness; alignment is best-effort. */
static inline void *zip_aligned_alloc(size_t align, size_t size) {
    (void)align;
    return malloc(size);
}
static inline void zip_aligned_free(void *p) { free(p); }
