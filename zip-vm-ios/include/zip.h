#ifndef ZIP_H
#define ZIP_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Compile ZIP source to ZIPB (hex string).
 * @param source UTF-8 ZIP program.
 * @return hex string (caller must free with zip_free), or NULL on error.
 */
char *zip_compile(const char *source);

/**
 * Run ZIPB (hex string), return VM output.
 */
char *zip_run(const char *bytecode_hex);

/**
 * Full pipeline: source → output.
 */
char *zip_compile_and_run(const char *source);

/**
 * Free a string returned by zip_compile / zip_run / zip_compile_and_run.
 */
void zip_free(char *s);

/**
 * ZIP version (static string, do not free).
 */
const char *zip_version(void);

/**
 * 1 if last operation succeeded, 0 otherwise.
 */
int zip_last_success(void);

#ifdef __cplusplus
}
#endif

#endif /* ZIP_H */
