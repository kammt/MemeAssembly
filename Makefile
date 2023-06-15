CC_win=x86_64-w64-mingw32-gcc

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
CFLAGS+=-std=gnu17 -D $(PLATFORM_MACRO) -O2
CFLAGS_DEBUG+=-O0 -Wall -Wextra -Wpedantic -Wmisleading-indentation -g

# Destination directory for make install
bindir=/usr/local/bin

INSTALL=install
INSTALL_PROGRAM=$(INSTALL)

# Files to compile
FILES=compiler/memeasm.c compiler/compiler.c compiler/logger/log.c compiler/parser/parser.c compiler/parser/fileParser.c compiler/parser/functionParser.c compiler/analyser/analysisHelper.c compiler/analyser/parameters.c compiler/analyser/functions.c compiler/analyser/jumpMarkers.c compiler/analyser/comparisons.c compiler/analyser/randomCommands.c compiler/analyser/analyser.c compiler/translator/translator.c

.PHONY: all clean debug uninstall install windows wasm wasm/memeasm.js

# Standard compilation
all:
	$(CC) -o memeasm $(FILES) $(CFLAGS)

wasm: wasm/memeasm.js
wasm/memeasm.js:
	docker run --rm -v $(shell pwd):/src -u $(shell id -u):$(shell id -g) \
		emscripten/emsdk emcc -o $@ -s WASM=1 -s INVOKE_RUN=0 -s EXPORT_ES6=1 -s MODULARIZE=1 -s EXPORTED_RUNTIME_METHODS="['FS', 'callMain', 'cwrap']" -s 'EXPORT_NAME="runMemeAssemblyCompiler"' $(FILES) $(CFLAGS)

# Compilation with debugging-flags
debug:
	$(CC) -o memeasm $(FILES) $(CFLAGS) $(CFLAGS_DEBUG)

# Remove the compiled executable from this directory
clean:
	$(RM) memeasm wasm/memeasm.{js,wasm}

# Removes "memeasm" from DESTDIR
uninstall: 
	$(RM) $(DESTDIR)$(bindir)/memeasm

# Compiles an executable and stores it in DESTDIR
install: all
	mkdir -p $(DESTDIR)$(bindir)
	$(INSTALL_PROGRAM) memeasm $(DESTDIR)$(bindir)/memeasm

# For building a windows executable under Linux
windows:
	$(CC_win) -o memeasm.exe $(FILES) $(CFLAGS)
