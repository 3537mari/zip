/* ZIP STD: cryptography via OpenSSL */
#define _CRT_SECURE_NO_WARNINGS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/sha.h>
#include <openssl/evp.h>
#include <openssl/rand.h>
#ifdef _WIN32
#include <openssl/applink.c>
#endif

unsigned char *zip_sha256(const unsigned char *data, size_t len) {
    unsigned char *hash = (unsigned char *)malloc(SHA256_DIGEST_LENGTH);
    if (!hash) return NULL;
    SHA256(data, len, hash);
    return hash;
}

char *zip_sha256_hex(const unsigned char *data, size_t len) {
    unsigned char hash[SHA256_DIGEST_LENGTH];
    SHA256(data, len, hash);
    char *hex = (char *)malloc(SHA256_DIGEST_LENGTH * 2 + 1);
    if (!hex) return NULL;
    for (int i = 0; i < SHA256_DIGEST_LENGTH; i++) {
        sprintf(hex + i * 2, "%02x", hash[i]);
    }
    hex[SHA256_DIGEST_LENGTH * 2] = '\0';
    return hex;
}

typedef struct {
    unsigned char *ciphertext;
    size_t ciphertext_len;
    unsigned char tag[16];
    unsigned char iv[12];
} ZipAesGcmResult;

ZipAesGcmResult *zip_aes_gcm_encrypt(const unsigned char *data, size_t len,
                                     const unsigned char *key) {
    ZipAesGcmResult *out = (ZipAesGcmResult *)calloc(1, sizeof(ZipAesGcmResult));
    if (!out) return NULL;
    if (RAND_bytes(out->iv, 12) != 1) {
        free(out);
        return NULL;
    }

    EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
    if (!ctx) {
        free(out);
        return NULL;
    }
    EVP_EncryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, 12, NULL);
    EVP_EncryptInit_ex(ctx, NULL, NULL, key, out->iv);

    out->ciphertext = (unsigned char *)malloc(len + 16);
    int outl = 0;
    EVP_EncryptUpdate(ctx, out->ciphertext, &outl, data, (int)len);
    out->ciphertext_len = (size_t)outl;
    int fin = 0;
    EVP_EncryptFinal_ex(ctx, out->ciphertext + outl, &fin);
    out->ciphertext_len += (size_t)fin;
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_GET_TAG, 16, out->tag);
    EVP_CIPHER_CTX_free(ctx);
    return out;
}

unsigned char *zip_aes_gcm_decrypt(const unsigned char *ciphertext, size_t len,
                                   const unsigned char *key, const unsigned char *iv,
                                   const unsigned char *tag) {
    EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
    if (!ctx) return NULL;
    EVP_DecryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL);
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, 12, NULL);
    EVP_DecryptInit_ex(ctx, NULL, NULL, key, iv);

    unsigned char *plain = (unsigned char *)malloc(len + 16);
    int outl = 0;
    EVP_DecryptUpdate(ctx, plain, &outl, ciphertext, (int)len);
    int total = outl;
    EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_TAG, 16, (void *)tag);
    int ret = EVP_DecryptFinal_ex(ctx, plain + outl, &outl);
    EVP_CIPHER_CTX_free(ctx);
    if (ret <= 0) {
        free(plain);
        return NULL;
    }
    total += outl;
    plain[total] = '\0';
    return plain;
}

unsigned char *zip_random_bytes(size_t n) {
    unsigned char *b = (unsigned char *)malloc(n);
    if (!b) return NULL;
    if (RAND_bytes(b, (int)n) != 1) {
        free(b);
        return NULL;
    }
    return b;
}

#ifdef ZIP_CRYPTO_MAIN
int main(void) {
    const char *msg = "привет мир";
    size_t len = strlen(msg);
    char *hex = zip_sha256_hex((const unsigned char *)msg, len);
    printf("SHA256: %s\n", hex ? hex : "(null)");
    free(hex);

    unsigned char *key = zip_random_bytes(32);
    const char *secret = "секретное сообщение";
    size_t slen = strlen(secret);
    ZipAesGcmResult *enc = zip_aes_gcm_encrypt((const unsigned char *)secret, slen, key);
    if (!enc) {
        fprintf(stderr, "encrypt failed\n");
        return 1;
    }
    printf("AES-GCM encrypted: %zu bytes\n", enc->ciphertext_len);
    unsigned char *dec =
        zip_aes_gcm_decrypt(enc->ciphertext, enc->ciphertext_len, key, enc->iv, enc->tag);
    if (!dec) {
        fprintf(stderr, "decrypt failed\n");
        return 1;
    }
    printf("AES-GCM decrypted: %s\n", dec);
    free(dec);
    free(enc->ciphertext);
    free(enc);
    free(key);
    return 0;
}
#endif
