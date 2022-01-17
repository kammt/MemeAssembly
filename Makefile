CC_win=x86_64-w64-mingw32-gcc

PLATFORM_MACRO=
ifeq ($(OS),Windows_NT)     # is Windows_NT on XP, 2000, 7, Vista, 10...
    PLATFORM_MACRO=WINDOWS
else
    detected_OS := $(shell uname)  # same as "uname -s"
    #assuming that a non-Windows OS is Linux
    PLATFORM_MACRO=LINUX
endif

# Compiler Flags
CFLAGS+=-std=gnu11 -D $(PLATFORM_MACRO)
CFLAGS_DEBUG+=-Wall -Wextra -Wpedantic -Wmisleading-indentation -g
# Destination directory for make install
DESTDIR=/usr/bin
# Files to compile
FILES=compiler/memeasm.c compiler/compiler.c compiler/logger/log.c compiler/parser/parser.c compiler/analyzer/parameters.c compiler/analyzer/functions.c compiler/analyzer/jumpLabels.c compiler/analyzer/comparisons.c compiler/analyzer/randomCommands.c compiler/translator/translator.c

.PHONY: all clean debug uninstall install

# Standard compilation
all:
	$(CC) -o memeasm $(FILES) $(CFLAGS)

# Compilation with debugging-flags
debug:
	$(CC) -o memeasm $(FILES) $(CFLAGS) $(CFLAGS_DEBUG)

# Removed the compiled executable from this directory
clean: 
	$(RM) memeasm

# Removes "memeasm" from DESTDIR
uninstall: 
	$(RM) $(DESTDIR)/memeasm

# Compiles an executable and stores it in DESTDIR
install:
	mkdir $(DESTDIR) -p
	$(CC) -o $(DESTDIR)/memeasm $(FILES) $(CFLAGS)

# For building a windows executable under Linux
windows:
	$(CC_win) -o memeasm.exe $(FILES) $(CFLAGS)