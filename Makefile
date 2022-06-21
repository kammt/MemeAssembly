CC_win=x86_64-w64-mingw32-gcc
CC_wasm=emcc

PLATFORM_MACRO=
ifeq ($(OS),Windows_NT)     # is Windows_NT on XP, 2000, 7, Vista, 10...
    PLATFORM_MACRO=WINDOWS
else
    detected_OS := $(shell uname -s)
    ifeq ($(detected_OS),Darwin)
    	PLATFORM_MACRO=MACOS
    else
    	PLATFORM_MACRO=LINUX
    endif
endif

# Compiler Flags
CFLAGS+=-std=gnu11 -D $(PLATFORM_MACRO) -O2
CFLAGS_DEBUG+=-O0 -Wall -Wextra -Wpedantic -Wmisleading-indentation -g

# Destination directory for make install
DESTDIR=/usr/local/bin

# Files to compile
FILES=compiler/memeasm.c compiler/compiler.c compiler/logger/log.c compiler/parser/parser.c compiler/parser/fileParser.c compiler/parser/functionParser.c compiler/analyser/analysisHelper.c compiler/analyser/parameters.c compiler/analyser/functions.c compiler/analyser/jumpMarkers.c compiler/analyser/comparisons.c compiler/analyser/randomCommands.c compiler/analyser/analyser.c compiler/translator/translator.c

.PHONY: all clean debug uninstall install windows wasm wasm/memeasm.js

# Standard compilation
all:
	$(CC) -o memeasm $(FILES) $(CFLAGS)

wasm: wasm/memeasm.js
wasm/memeasm.js:
	$(CC_wasm) -o $@ -s WASM=1 -s INVOKE_RUN=0 -s EXPORT_ES6=1 -s MODULARIZE=1 -s EXPORTED_RUNTIME_METHODS="['FS', 'callMain', 'cwrap']" -s 'EXPORT_NAME="runMemeAssemblyCompiler"' $(FILES) $(CFLAGS)

# Compilation with debugging-flags
debug:
	$(CC) -o memeasm $(FILES) $(CFLAGS) $(CFLAGS_DEBUG)

# Remove the compiled executable from this directory
clean:
	$(RM) memeasm

# Removes "memeasm" from DESTDIR
uninstall:
	$(RM) $(DESTDIR)/memeasm

# Compiles an executable and stores it in DESTDIR
install:
	mkdir -p $(DESTDIR)
	$(CC) -o $(DESTDIR)/memeasm $(FILES) $(CFLAGS)

# For building a windows executable under Linux
windows:
	$(CC_win) -o memeasm.exe $(FILES) $(CFLAGS)
