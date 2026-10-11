/* Runtime ZIP-архивов (чтение stored; запись минимального архива). */
#define _CRT_SECURE_NO_WARNINGS
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char имя[256];
    uint32_t размер_несжатый;
    uint32_t размер_сжатый;
    uint32_t смещение;
    uint32_t crc32;
    uint16_t метод;
} ZipЗапись;

typedef struct {
    ZipЗапись *записи;
    int количество;
    int ёмкость;
    FILE *файл;
} ZipАрхив;

void zip_архив_закрыть(ZipАрхив *а);

static uint32_t zip_crc32(const uint8_t *data, size_t len) {
    uint32_t crc = 0xFFFFFFFFu;
    size_t i;
    int j;
    for (i = 0; i < len; i++) {
        crc ^= data[i];
        for (j = 0; j < 8; j++) {
            crc = (crc >> 1) ^ (0xEDB88320u & (uint32_t)-(int)(crc & 1u));
        }
    }
    return ~crc;
}

static void write_u16(FILE *f, uint16_t v) { fwrite(&v, 2, 1, f); }
static void write_u32(FILE *f, uint32_t v) { fwrite(&v, 4, 1, f); }

int zip_архив_создать_простой(const char *путь, const char *имя_внутри,
                              const char *данные) {
    FILE *f;
    uint32_t sz, crc, off_cd;
    uint16_t nlen;
    if (!путь || !имя_внутри || !данные) return -1;
    f = fopen(путь, "wb");
    if (!f) return -1;
    sz = (uint32_t)strlen(данные);
    crc = zip_crc32((const uint8_t *)данные, sz);
    nlen = (uint16_t)strlen(имя_внутри);

    write_u32(f, 0x04034b50u);
    write_u16(f, 20);
    write_u16(f, 0);
    write_u16(f, 0);
    write_u16(f, 0);
    write_u16(f, 0);
    write_u32(f, crc);
    write_u32(f, sz);
    write_u32(f, sz);
    write_u16(f, nlen);
    write_u16(f, 0);
    fwrite(имя_внутри, 1, nlen, f);
    fwrite(данные, 1, sz, f);

    off_cd = (uint32_t)ftell(f);
    write_u32(f, 0x02014b50u);
    write_u16(f, 20);
    write_u16(f, 20);
    write_u16(f, 0);
    write_u16(f, 0);
    write_u16(f, 0);
    write_u16(f, 0);
    write_u32(f, crc);
    write_u32(f, sz);
    write_u32(f, sz);
    write_u16(f, nlen);
    write_u16(f, 0);
    write_u16(f, 0);
    write_u16(f, 0);
    write_u16(f, 0);
    write_u32(f, 0);
    write_u32(f, 0);
    fwrite(имя_внутри, 1, nlen, f);
    {
        uint32_t cd_size = (uint32_t)ftell(f) - off_cd;
        write_u32(f, 0x06054b50u);
        write_u16(f, 0);
        write_u16(f, 0);
        write_u16(f, 1);
        write_u16(f, 1);
        write_u32(f, cd_size);
        write_u32(f, off_cd);
        write_u16(f, 0);
    }
    fclose(f);
    return 0;
}

ZipАрхив *zip_архив_открыть(const char *путь) {
    FILE *ф;
    ZipАрхив *а;
    long размер, позиция, min_pos, start;
    if (!путь) return NULL;
    ф = fopen(путь, "rb");
    if (!ф) return NULL;
    а = (ZipАрхив *)calloc(1, sizeof(ZipАрхив));
    if (!а) {
        fclose(ф);
        return NULL;
    }
    а->файл = ф;
    а->ёмкость = 16;
    а->записи = (ZipЗапись *)calloc((size_t)а->ёмкость, sizeof(ZipЗапись));
    if (!а->записи) {
        fclose(ф);
        free(а);
        return NULL;
    }

    fseek(ф, 0, SEEK_END);
    размер = ftell(ф);
    if (размер < 22) {
        zip_архив_закрыть(а);
        return NULL;
    }
    start = размер - 22;
    min_pos = размер - 22 - 65535;
    if (min_pos < 0) min_pos = 0;
    for (позиция = start; позиция >= min_pos; позиция--) {
        uint32_t подпись = 0;
        fseek(ф, позиция, SEEK_SET);
        if (fread(&подпись, 4, 1, ф) != 1) continue;
        if (подпись == 0x06054b50u) {
            uint16_t количество_записей = 0;
            uint32_t смещение_каталога = 0;
            int i;
            fseek(ф, позиция + 10, SEEK_SET);
            fread(&количество_записей, 2, 1, ф);
            fseek(ф, позиция + 16, SEEK_SET);
            fread(&смещение_каталога, 4, 1, ф);
            fseek(ф, смещение_каталога, SEEK_SET);
            for (i = 0; i < (int)количество_записей; i++) {
                uint32_t подпись_записи = 0;
                ZipЗапись запись;
                uint16_t метод = 0, длина_имени = 0, длина_доп = 0,
                         длина_комментария = 0;
                memset(&запись, 0, sizeof(запись));
                if (fread(&подпись_записи, 4, 1, ф) != 1) break;
                if (подпись_записи != 0x02014b50u) break;
                fseek(ф, 6, SEEK_CUR); /* version made + needed + flags */
                fread(&метод, 2, 1, ф);
                запись.метод = метод;
                fseek(ф, 4, SEEK_CUR); /* time + date */
                fread(&запись.crc32, 4, 1, ф);
                fread(&запись.размер_сжатый, 4, 1, ф);
                fread(&запись.размер_несжатый, 4, 1, ф);
                fread(&длина_имени, 2, 1, ф);
                fread(&длина_доп, 2, 1, ф);
                fread(&длина_комментария, 2, 1, ф);
                fseek(ф, 8, SEEK_CUR);
                fread(&запись.смещение, 4, 1, ф);
                if (длина_имени >= sizeof(запись.имя))
                    длина_имени = (uint16_t)(sizeof(запись.имя) - 1);
                fread(запись.имя, 1, длина_имени, ф);
                запись.имя[длина_имени] = '\0';
                fseek(ф, длина_доп + длина_комментария, SEEK_CUR);
                if (а->количество >= а->ёмкость) {
                    int n = а->ёмкость * 2;
                    ZipЗапись *nr = (ZipЗапись *)realloc(
                        а->записи, (size_t)n * sizeof(ZipЗапись));
                    if (!nr) break;
                    а->записи = nr;
                    а->ёмкость = n;
                }
                а->записи[а->количество++] = запись;
            }
            return а;
        }
    }
    zip_архив_закрыть(а);
    return NULL;
}

int zip_архив_количество(ZipАрхив *а) { return а ? а->количество : 0; }

const char *zip_архив_имя(ZipАрхив *а, int индекс) {
    if (!а || индекс < 0 || индекс >= а->количество) return NULL;
    return а->записи[индекс].имя;
}

uint32_t zip_архив_размер(ZipАрхив *а, int индекс) {
    if (!а || индекс < 0 || индекс >= а->количество) return 0;
    return а->записи[индекс].размер_несжатый;
}

char *zip_архив_извлечь(ZipАрхив *а, const char *имя, uint32_t *размер) {
    int i;
    if (!а || !имя) return NULL;
    for (i = 0; i < а->количество; i++) {
        if (strcmp(а->записи[i].имя, имя) == 0) {
            ZipЗапись *з = &а->записи[i];
            uint32_t подпись = 0;
            uint16_t длина_имени = 0, длина_доп = 0;
            char *данные;
            fseek(а->файл, з->смещение, SEEK_SET);
            if (fread(&подпись, 4, 1, а->файл) != 1 || подпись != 0x04034b50u)
                return NULL;
            fseek(а->файл, 22, SEEK_CUR);
            fread(&длина_имени, 2, 1, а->файл);
            fread(&длина_доп, 2, 1, а->файл);
            fseek(а->файл, длина_имени + длина_доп, SEEK_CUR);
            if (з->метод != 0) return NULL;
            данные = (char *)malloc((size_t)з->размер_несжатый + 1);
            if (!данные) return NULL;
            if (fread(данные, 1, з->размер_несжатый, а->файл) !=
                з->размер_несжатый) {
                free(данные);
                return NULL;
            }
            данные[з->размер_несжатый] = '\0';
            if (размер) *размер = з->размер_несжатый;
            return данные;
        }
    }
    return NULL;
}

void zip_архив_закрыть(ZipАрхив *а) {
    if (!а) return;
    if (а->файл) fclose(а->файл);
    free(а->записи);
    free(а);
}

#ifdef ZIP_ARCHIVE_MAIN
int main(void) {
    const char *path = "zip_demo_test.zip";
    ZipАрхив *а;
    int i, n;
    if (zip_архив_создать_простой(path, "привет.txt", "Привет, ZIP!") != 0) {
        printf("Не могу создать архив\n");
        return 1;
    }
    а = zip_архив_открыть(path);
    if (!а) {
        printf("Не могу открыть архив\n");
        return 1;
    }
    n = zip_архив_количество(а);
    printf("Файлов в архиве:\n%d\n", n);
    for (i = 0; i < n; i++) {
        printf("%s\n", zip_архив_имя(а, i));
        printf("%u\n", (unsigned)zip_архив_размер(а, i));
    }
    {
        uint32_t sz = 0;
        char *data = zip_архив_извлечь(а, "привет.txt", &sz);
        printf("Содержимое:\n%s\n", data ? data : "?");
        free(data);
    }
    zip_архив_закрыть(а);
    remove(path);
    return 0;
}
#endif

#ifdef ZIP_ARCHIVE_UNITTEST
static int fails;
#define CHECK(c)                                                               \
    do {                                                                       \
        if (!(c)) {                                                            \
            fprintf(stderr, "FAIL %s:%d %s\n", __FILE__, __LINE__, #c);        \
            fails++;                                                           \
        }                                                                      \
    } while (0)

int main(void) {
    const char *path = "zip_unit_test.zip";
    ZipАрхив *а;
    uint32_t sz = 0;
    char *data;
    CHECK(zip_архив_создать_простой(path, "a.txt", "hello") == 0);
    а = zip_архив_открыть(path);
    CHECK(а != NULL);
    CHECK(zip_архив_количество(а) == 1);
    CHECK(strcmp(zip_архив_имя(а, 0), "a.txt") == 0);
    CHECK(zip_архив_размер(а, 0) == 5);
    data = zip_архив_извлечь(а, "a.txt", &sz);
    CHECK(data && sz == 5 && strcmp(data, "hello") == 0);
    free(data);
    CHECK(zip_архив_извлечь(а, "нет.txt", &sz) == NULL);
    zip_архив_закрыть(а);
    remove(path);
    if (fails) {
        fprintf(stderr, "%d failed\n", fails);
        return 1;
    }
    printf("archive unit tests ok\n");
    return 0;
}
#endif
