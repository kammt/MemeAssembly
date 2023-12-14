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
CFLAGS+=-std=c++20 -D $(PLATFORM_MACRO) -O2 
CFLAGS_DEBUG+=-O0 -Wall -Wextra -Wpedantic -Wmisleading-indentation -g

# Destination directory for make install
bindir=/usr/local/bin

INSTALL=install
INSTALL_PROGRAM=$(INSTALL)

# Files to compile
FILES=compiler/memeasm.c compiler/compiler.cpp compiler/logger/log.cpp compiler/parser/parser.c compiler/parser/commandParser.c compiler/analyser/analyser.cpp compiler/analyser/functions.c compiler/analyser/parameters.c compiler/translator/translator.c

.PHONY: all clean debug uninstall install windows wasm memeasm.js

# Standard compilation
all:
	g++ -o memeasm $(FILES) $(CFLAGS)

wasm: memeasm.js
memeasm.js:
	docker run --rm -v $(shell pwd):/src -u $(shell id -u):$(shell id -g) \
		emscripten/emsdk emcc -o $@ -s WASM=1 -s INVOKE_RUN=0 -s EXPORT_ES6=1 -s MODULARIZE=1 -s EXPORTED_RUNTIME_METHODS="['FS', 'callMain', 'cwrap']" -s 'EXPORT_NAME="runMemeAssemblyCompiler"' $(FILES) $(CFLAGS)

# Compilation with debugging-flags
debug:
	g++ -o memeasm $(FILES) $(CFLAGS) $(CFLAGS_DEBUG)

# Remove the compiled executable from this directory
clean:
	$(RM) memeasm memeasm.{js,wasm}

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
