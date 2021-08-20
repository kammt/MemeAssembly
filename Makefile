CC=gcc
CFLAGS+=-std=gnu11
CFLAGS_DEBUG+=-Wall -Wextra -Wpedantic -Wshadow -Wdouble-promotion -Wformat=2 -Wformat-truncation -Wundef -fno-common -Wconversion -Wmisleading-indentation -g3
DESTDIR=/usr/bin

.PHONY: all clean debug uninstall install

all: compiler/memeasm.c compiler/compiler.c compiler/logger/log.c compiler/parser/parser.c compiler/analyzer/parameters.c compiler/analyzer/functions.c compiler/analyzer/jumpLabels.c compiler/analyzer/comparisons.c compiler/analyzer/randomCommands.c compiler/translator/translator.c
	$(CC) -o memeasm $^ $(CFLAGS)

debug: compiler/memeasm.c compiler/compiler.c compiler/logger/log.c compiler/parser/parser.c compiler/analyzer/parameters.c compiler/analyzer/functions.c compiler/analyzer/jumpLabels.c compiler/analyzer/comparisons.c compiler/analyzer/randomCommands.c compiler/translator/translator.c
	$(CC) -o memeasm $^ $(CFLAGS) $(CFLAGS_DEBUG)

clean: 
	$(RM) memeasm

uninstall: 
	$(RM) $(DESTDIR)/memeasm

install: compiler/memeasm.c compiler/compiler.c compiler/logger/log.c compiler/parser/parser.c compiler/analyzer/parameters.c compiler/analyzer/functions.c compiler/analyzer/jumpLabels.c compiler/analyzer/comparisons.c compiler/analyzer/randomCommands.c compiler/translator/translator.c
	mkdir $(DESTDIR) -p
	$(CC) -o $(DESTDIR)/memeasm $^ $(CFLAGS)