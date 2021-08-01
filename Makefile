CC=gcc
CFLAGS+=-lm -std=gnu11 -O3
CFLAGS_DEBUG+=-Wall -Wextra -Wpedantic -Wshadow -Wdouble-promotion -Wformat=2 -Wformat-truncation -Wundef -fno-common -Wconversion -Wmisleading-indentation -g3

.PHONY: all clean debug uninstall install

all: compiler/memeasm.c compiler/compiler.c compiler/analyser.c compiler/translator.c compiler/logger/log.c compiler/preprocessor.c
	$(CC) -o memeasm $^ $(CFLAGS)

debug: compiler/memeasm.c compiler/compiler.c compiler/analyser.c compiler/translator.c compiler/logger/log.c compiler/preprocessor.c
	$(CC) -o memeasm $^ $(CFLAGS) $(CLFAGS_DEBUG)

clean: 
	$(RM) memeasm

uninstall: 
	$(RM) /usr/bin/memeasm

install: compiler/memeasm.c compiler/compiler.c compiler/analyser.c compiler/translator.c compiler/logger/log.c compiler/preprocessor.c
	$(CC) -o /usr/bin/memeasm $^ $(CFLAGS)