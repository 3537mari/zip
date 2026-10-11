# Bootstrap ZIP (C) — полный путь
CC ?= gcc
CFLAGS ?= -O3 -Wall -Wextra -std=c11 -D_CRT_SECURE_NO_WARNINGS
LDFLAGS ?=

SOURCES = утилиты.c таблицы.c лексер.c парсер.c генератор.c дамп.c проверка.c главное.c
OBJECTS = $(SOURCES:.c=.o)
TARGET = zipc

.PHONY: all clean test windows clang loc

all: $(TARGET)

$(TARGET): $(OBJECTS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

%.o: %.c zipb.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(OBJECTS) $(TARGET) $(TARGET).exe t_*.zip t_*.zipb

test: $(TARGET)
	@printf '%s\n' 'ПечатьЧисло(2+3)' > t_arith.zip
	./$(TARGET) t_arith.zip t_arith.zipb
	@od -An -tx1 -N16 t_arith.zipb 2>/dev/null || xxd -l 16 t_arith.zipb || true

windows clang:
	clang $(CFLAGS) -o $(TARGET).exe $(SOURCES)

loc:
	@wc -l zipb.h $(SOURCES) 2>/dev/null || (Get-Content zipb.h,утилиты.c,таблицы.c,лексер.c,парсер.c,генератор.c,главное.c | Measure-Object -Line)
