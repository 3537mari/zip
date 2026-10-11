# Bootstrap ZIP (C)
CC ?= gcc
CFLAGS ?= -O3 -Wall -Wextra -std=c11
LDFLAGS ?=

SOURCES = утилиты.c лексер.c парсер.c генератор.c главное.c
OBJECTS = $(SOURCES:.c=.o)
TARGET = zipc

.PHONY: all clean test windows clang

all: $(TARGET)

$(TARGET): $(OBJECTS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

%.o: %.c zipb.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(OBJECTS) $(TARGET) $(TARGET).exe тест_привет.zip тест_привет.zipb

test: $(TARGET)
	@echo "=== Тест: ПечатьЧисло(42) ==="
	@printf '%s\n' 'ПечатьЧисло(42)' > тест_привет.zip
	./$(TARGET) тест_привет.zip тест_привет.zipb
	@echo "=== Заголовок ==="
	@od -An -tx1 -N16 тест_привет.zipb || xxd -l 16 тест_привет.zipb || true
	@rm -f тест_привет.zip тест_привет.zipb

windows:
	clang -O3 -o $(TARGET).exe $(SOURCES)

clang:
	clang -O3 -Wall -Wextra -std=c11 -o $(TARGET) $(SOURCES)
