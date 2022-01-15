CC=gcc
CC_win=x86_64-w64-mingw32-gcc

CFLAGS+=-std=gnu11
CFLAGS_DEBUG+=-Wall -Wextra -Wpedantic -Wshadow -Wdouble-promotion -Wformat=2 -Wformat-truncation -Wundef -fno-common -Wconversion -Wmisleading-indentation -g3
DESTDIR=/usr/bin
FILES=compiler/memeasm.c compiler/compiler.c compiler/logger/log.c compiler/parser/parser.c compiler/analyzer/parameters.c compiler/analyzer/functions.c compiler/analyzer/jumpLabels.c compiler/analyzer/comparisons.c compiler/analyzer/randomCommands.c compiler/translator/translator.c

.PHONY: all clean debug uninstall install

all:
	$(CC) -o memeasm $(FILES) $(CFLAGS)

debug:
	$(CC) -o memeasm $(FILES) $(CFLAGS) $(CFLAGS_DEBUG)

clean: 
	$(RM) memeasm

uninstall: 
	$(RM) $(DESTDIR)/memeasm

install:
	mkdir $(DESTDIR) -p
	$(CC) -o $(DESTDIR)/memeasm $(FILES) $(CFLAGS)

windows:
	$(CC_win) -o memeasm.exe $(FILES)