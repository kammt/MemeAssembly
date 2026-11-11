# Thin wrapper around cargo, so that `make`, `make install` etc. keep working.
# Building needs a Rust toolchain (cargo) and a C compiler for the vendored fadec encoder (see build.rs).

CARGO ?= cargo
# Extra flags for every cargo invocation, e.g. CARGO_FLAGS=--locked
CARGO_FLAGS ?=

# cargo's output directory (honours CARGO_TARGET_DIR from the environment)
TARGET_DIR := $(or $(CARGO_TARGET_DIR),target)

EXE :=
ifeq ($(OS),Windows_NT)     # is Windows_NT on XP, 2000, 7, Vista, 10...
    # MSVC cannot build the vendored fadec C code, so use the GNU (MinGW-w64) toolchain
    CARGO := $(CARGO) +stable-x86_64-pc-windows-gnu
    EXE := .exe
endif

# For building a windows executable under Linux
CC_win = x86_64-w64-mingw32-gcc
WINDOWS_TARGET = x86_64-pc-windows-gnu

# Destination directory for make install
bindir=/usr/local/bin

INSTALL=install
INSTALL_PROGRAM=$(INSTALL)

RELEASE_BIN := $(TARGET_DIR)/release/memeasm$(EXE)
DEBUG_BIN := $(TARGET_DIR)/debug/memeasm$(EXE)
SOURCES := Cargo.toml Cargo.lock build.rs $(shell find src vendor -type f 2>/dev/null)

.PHONY: all clean debug uninstall install windows

# Standard compilation, leaves ./memeasm in this directory
all:
	$(CARGO) build --release $(CARGO_FLAGS)
	cp -f $(RELEASE_BIN) memeasm$(EXE)

# Compilation with debug info and without optimisations
debug:
	$(CARGO) build $(CARGO_FLAGS)
	cp -f $(DEBUG_BIN) memeasm$(EXE)

# Only rebuilt when out of date, so that `make && sudo make install` does not need cargo as root
$(RELEASE_BIN): $(SOURCES)
	$(CARGO) build --release $(CARGO_FLAGS)
	@touch $@

# Remove the compiled executables and cargo's build directory
clean:
	$(RM) memeasm memeasm.exe
	$(CARGO) clean

# Removes "memeasm" from DESTDIR
uninstall:
	$(RM) $(DESTDIR)$(bindir)/memeasm

# Compiles an executable (if necessary) and stores it in DESTDIR
install: $(RELEASE_BIN)
	mkdir -p $(DESTDIR)$(bindir)
	$(INSTALL_PROGRAM) $(RELEASE_BIN) $(DESTDIR)$(bindir)/memeasm

# For building a windows executable under Linux (needs `rustup target add x86_64-pc-windows-gnu` and mingw-w64)
windows:
	CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=$(CC_win) CC_x86_64_pc_windows_gnu=$(CC_win) \
		$(CARGO) build --release --target $(WINDOWS_TARGET) $(CARGO_FLAGS)
	cp -f $(TARGET_DIR)/$(WINDOWS_TARGET)/release/memeasm.exe memeasm.exe
