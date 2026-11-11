# Rust rewrite: intentional differences to the C compiler

The Rust frontend (`src/main.rs`, `src/frontend/`) reproduces the C compiler (`compiler/`)
byte for byte: the `-S` output, diagnostics and exit codes match for every compile mode,
optimisation level and target. The exceptions are listed here. `tests/frontend_parity.sh`
checks parity against the C reference builds. It whitelists exactly these differences:
the per-fix rewrite rules are in `tests/frontend_parity_whitelist.pl`, and `check_fixed`
covers the cases where the C compiler crashes or hangs.

Line numbers refer to the C sources at commit 8f186d8.

## Fixed bugs

These are unintended behaviours: crashes, hangs, undefined behaviour, assembly that can never
assemble or link, or behaviour that contradicts the program's own help text or comments. The
Rust port implements the intended behaviour instead.

| # | C source | Trigger | Old behaviour (C) | New behaviour (Rust) |
|---|----------|---------|-------------------|----------------------|
| 1 | `compiler/compiler.c:333` | `wait, that's illegal` | The pattern is `xor r13 r13` (missing comma). gas fails with ``Error: junk `r13' after expression``, so every program using the command fails to assemble. | Emits `xor r13, r13`, both in the `-S` text and on the direct Emitter path. |
| 2 | `compiler/commands.h:68` (enum), used at `translator/translator.c:207` and `:472` | `-O-2` or `-O-s` | `o_s` follows `o_3 = -3` and therefore gets the value -2, the same as `o_2`. Both options insert `push rax`/`pop rax` after every command **and** append `.align 536870912`. | `-O-2` only inserts the push/pop filler and `-O-s` only appends the alignment, as the help page describes. (`OPT_S` is now -4.) |
| 3 | `compiler/translator/translator.c:173` | Bully mode, a two-parameter command whose **second** parameter is a pointer and whose first is a decimal or character, e.g. `parry 5 you filthy casual rax do you know de wey` | `paramTypes[index + 1 % 2]` evaluates to `paramTypes[index + 1]`. For index 1 this reads past the array (it lands on `isPointer`, which counts as a register), so no operand size is added. The result is `sub [rax], 0x5`, which gas rejects as ambiguous. | Uses `(index + 1) % 2`, the other parameter. The pointer gets the bully operand size, e.g. `sub BYTE PTR [rax], 0x5`. |
| 4 | `compiler/translator/translator.c:289` | Any macOS output (`killParent`, used by martyrdom) | `mov rax, 60` is the Linux `exit` number. On macOS it is not exit (the other two syscalls in the block use macOS numbers). | `mov rax, 0x2000001` (macOS `exit`) on macOS. Linux is unchanged. |
| 5 | `compiler/translator/translator.c:301-303` | Bully mode, executable output, no `main` function, target **Windows** | The injected `main` contains the martyrdom code, which references `killParent` and `.LsigStruct` (never emitted on Windows) and uses macOS syscall numbers. Linking always fails. | On Windows the injected `main` has no martyrdom code. Martyrdom is unsupported on Windows, as in the regular `main` path at `translator.c:319-330`. |
| 6 | `compiler/translator/translator.c:301-302` | Same as #5, target **macOS** | The injected entry point is named `main`, but macOS needs `_main` (which `mainFunctionExists`, `functions.c:52-57`, checks for). Linking fails. | The injected entry point is `_main` on macOS. |
| 7 | `compiler/analyser/randomCommands.c:88` | "perfectly balanced as all things should be" with several input files, one of them without code (bully mode accepts empty files), e.g. `-fcompile-mode bully -S -o x.S empty.memeasm pb.memeasm` | `rand() % 0` followed by `parsedCommands[...]` of a NULL array crashes the compiler (SIGFPE on x86-64, SIGSEGV on the aarch64 reference). | A randomly chosen file without lines is skipped (one `rand()` value is consumed and the loop draws again). Otherwise the random sequence is unchanged. |
| 8 | `compiler/analyser/randomCommands.c:83` | Bully mode, "perfectly balanced" in a program where more lines were already dropped by bully fixes than will remain, e.g. `tests/frontend_cases/fixes/perfectly_balanced_hang.memeasm` (4× orphaned `fuck go back` + 1× perfectly balanced) | The loop wants more lines than are still translatable and never ends: the compiler hangs. | The loop stops once every translatable line is deleted, i.e. it deletes `min(linesToBeDeleted, translatable lines)`. Results are unchanged whenever the C code terminated. |
| 9 | `compiler/analyser/parameters.c:545` | Bully mode, non-`mov` 64-bit arithmetic with a decimal that is not sign-extendable from 32 bits, e.g. `rbx units are ready, with 77777777777 more well on the way` (see `tests/frontend_cases/bully/pointer_second_and_truncation.memeasm`) | The decimal string is cut at index `computedIndex % 32`. If the number is shorter, this writes past the allocation (out-of-bounds write) and keeps the number. If the cut leaves more than 32 bits, the number stays too large. Either way gas rejects `add rbx, 0x121BEAB471`. | Truncates at the same index (when it is inside the string), then drops further trailing digits until the number fits into a sign-extended 32-bit immediate (`add rbx, 0x2E5BF271`). Numbers that the C cut already made valid are unchanged. |
| 10 | `compiler/memeasm.c:38-40` | `-h` | The usage lines still advertise `[-i \| -d]`, although `-i` was removed on purpose (upstream commit e99f0b3 "memeasm.c: Removed -i"; the info log level has no users), and `-i` is rejected as an unknown option. | The usage lines say `[-d]`. `-i` stays rejected, as the author intended. |

## Undefined behaviour given a defined result (no visible change)

In these cases the C program has undefined behaviour, but the reference builds happen to
produce a sensible result. The Rust port defines that result explicitly.

- `compiler/analyser/parameters.c:523`: `__builtin_clzll(~number)` is undefined for
  `number == -1`. The port computes the intended 1 bit, which is what the aarch64 reference
  gives, and does not depend on `clz(0)`.
- `compiler/analyser/analysisHelper.c:45`: the `-d` message "Comparing against parameter %s"
  prints `parameters[0]` of parameterless jump markers such as `upgrade`. That slot is
  uninitialised stack memory (garbage, or a stale string). The port prints `(null)`. The parity
  script masks this one debug line.
- `compiler/parser/fileParser.c:41`: `removeLineBreaksAndTabs` underflows its index when a line
  is all whitespace before a NUL byte. The port returns the empty line.
- `compiler/translator/translator.c:173`: the out-of-bounds read of `paramTypes[2]`. It is
  removed by fix #3.
- STABS function info was written after *every* command (`translator.c:346-348`). This no
  longer matters, because STABS was replaced (see below).

## Possible bugs, kept

These may be unintended, but they are plausibly part of the joke or harmless, so the original
behaviour is kept.

- `-O-2`/`-O-3` filler and `-O69420` behave exactly as before, including `movups [rsp + 8]`
  writing above the stack pointer.
- Bully mode inserts the martyrdom code into the injected `main` even with `-fno-martyrdom`
  (`translator.c:303`). This is kept on Linux and macOS.
- The confused stonks label is always emitted, even when no `confused stonks` is used: the
  analysis always runs `srand(time)` + `rand() % loc` per file (`randomCommands.c:36-45`).
- Bully mode's injected fake functions (for orphaned commands) get no `_` prefix on macOS.
  They are local helper names, so this is harmless.
- Command line edge cases of `getopt_long_only` that are not listed below behave as with
  glibc on all hosts.

## Intentional changes (not bugs)

### Debug information: DWARF instead of STABS

- `-g` now produces DWARF line information instead of STABS. In the `-S` text, the
  `.stabs`/`.stabn` directives, the `.Lcmd_N`/`.Lret_*` labels and the `.LEOF` end marker
  (with its preceding blank line) are gone. Each input file gets
  `.file N "/absolute/path"` where the `N_SO` stab used to be, and every translated command
  is preceded by `\t.loc N line`.
- On the direct Emitter path (`-O` / executables) the frontend calls `set_debug_meta`,
  `debug_file`, `debug_line` and `debug_function_begin`/`end` instead.
- The `-g` notes for Windows, macOS and bully mode are unchanged. The help line now reads
  "Currently, only the DWARF format is supported (Linux-only)".
- The parity script strips both debug formats before diffing.

### New option `-ftarget`

`-ftarget=<linux|macos|windows>` (also `-ftarget <os>`, `--ftarget`, `--target`) selects the
target operating system at run time. It replaces the C `#ifdef LINUX/MACOS/WINDOWS` builds, and
the default is the host OS. The help page gains one last line:
` -ftarget=<linux|macos|windows> - Change the target operating system (default: <host>)`.
An invalid value prints `Error: invalid target (must be one of "linux", "macos", "windows")`
and exits with code 1.

### Command line parsing: clap instead of getopt_long_only

All documented and previously accepted spellings keep their meaning and are processed in
command-line order. This covers:

- `-o`, `-S`, `-O`, `-O-1/-2/-3/-s`, `-O69420`, `-d`, `-g`, `-h`, `-v`.
- `-fcompile-mode <mode>` / `=<mode>`, `-fno-martyrdom`, and multiple input files.
- Single-dash long options and unique abbreviations (`-fc`, `-out`).
- Short clusters (`-dSo out`) and attached values (`-oout`).
- `--`, and options after the input files.

`tests/frontend_parity.sh` checks all of these. What changed:

1. **Argument errors come from clap.** This covers unknown options (`-x`, `-i`), ambiguous
   abbreviations (`-f`) and an option missing its value at the end of the command line (`-S -o`, `x -fcompile-mode`).
   - Old output: the glibc getopt message (`<argv0>: unrecognized option '-x'`,
     `option '-f' is ambiguous; possibilities: ...`), then `Error: Unknown option provided`
     and `Usage: <argv0> -o outputFile inputFile`.
   - New output: clap's message on stderr (`error: unexpected argument '-x' found`,
     `error: a value is required for '--output <FILE>' but none was supplied`, ...) plus its
     tip/usage text. The usage line names the program `memeasm`, not argv[0].
   - The exit code is still 1.
2. **Argument errors are found before any option is acted on.** C handled options one by one,
   so `memeasm -h -x` printed the help page and exited 0. Now clap rejects `-x` first and the
   exit code is 1. memeasm's own value checks (invalid `-O` level, compile mode, target) still
   run in command-line order, so `-v -O-7` prints the version (exit 0) and `-O-7 -v` prints
   the `-O` error (exit 1), as in C.
3. **New accepted aliases:** `--compile-mode`, `--no-martyrdom`, `--target`/`-target`,
   `--object` (= bare `-O`), `--opt-level=<level>` (= `-O<level>`) and `--version`/`-version`.
   `--ftarget`/`-ftarget` is new anyway, and it also counts as a candidate for abbreviations,
   so `-ft` now means `-ftarget`.
4. `POSIXLY_CORRECT` (stop at the first non-option) is not honoured.
5. The help page (`-h`) has the same content and exit code 0. It differs only in the `-g` line
   (DWARF), the `[-d]` usage fix (#10) and the added `-ftarget` line.

### Other known divergences

- The `# Generated by ... on <date>` timestamp is printed in UTC (std has no time zone
  database). Timestamps are excluded from parity anyway.
- `-O`/executable output no longer pipes text through gcc. The frontend drives
  `backend::emitter::Emitter` directly, instruction for instruction like the `-S` text.
  - Assembler errors are reported as gas would report them (`{standard input}: Assembler
    messages:` / `{standard input}: Error: ...`), but without line numbers and only the first
    one, followed by the usual "Assembling/linking failed..." message (exit 1).
  - `.align 536870912` (`-O-s`) cannot be emitted on this path yet, because the Emitter has no
    `align` method (an interface request to the backend).
- Function names that are also register names (e.g. a function called `rax`) are treated as
  symbols on the direct path, whereas gas would read them as registers.
