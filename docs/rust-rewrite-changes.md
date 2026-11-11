# Rust rewrite: intentional differences to the C compiler

The Rust frontend (`src/main.rs`, `src/frontend/`) reproduces the C compiler (`compiler/`):
the `-S` output, the diagnostics on each of stdout and stderr, and the exit codes match for
every compile mode, optimisation level and target. The exceptions are listed here. The
command line itself is new (clap, see "Command line"), and random choices are not
reproducible (see "Randomness").
`tests/frontend_parity.sh` checks parity against the C reference builds, passing the clap
spelling of each option to the Rust compiler. It whitelists exactly these differences: the
per-fix rewrite rules are in `tests/frontend_parity_whitelist.pl`, `check_fixed` covers the
cases where the C compiler crashes or hangs, `check_new_error` the inputs that are now
rejected (#14, #18) and `check_assembles` the bully fix-ups that now assemble (#18). Programs
with random choices are compared structurally (see "Randomness").

Line numbers refer to the C sources in `compiler/` at commit 8f186d8 (still on `main`).

## Fixed bugs

These are unintended behaviours: crashes, hangs, undefined behaviour, assembly that can never
assemble or link, or behaviour that contradicts the program's own help text or comments. The
Rust port implements the intended behaviour instead.

| # | C source | Trigger | Old behaviour (C) | New behaviour (Rust) |
|---|----------|---------|-------------------|----------------------|
| 1 | `compiler/compiler.c:333` | `wait, that's illegal` | The pattern is `xor r13 r13` (missing comma). gas fails with ``Error: junk `r13' after expression``, so every program using the command fails to assemble. | Emits `xor r13, r13`, both in the `-S` text and on the direct Emitter path. |
| 2 | `compiler/commands.h:68` (enum), used at `translator/translator.c:207` and `:472` | `-O-2` or `-O-s` | `o_s` follows `o_3 = -3` and therefore gets the value -2, the same as `o_2`. Both options insert `push rax`/`pop rax` after every command **and** append `.align 536870912`. | `--opt-level=-2` only inserts the push/pop filler and `--opt-level=-s` only appends the alignment, as the help page describes. (`OPT_S` is now -4.) |
| 3 | `compiler/translator/translator.c:173` | Bully mode, a two-parameter command whose **second** parameter is a pointer and whose first is a decimal or character, e.g. `parry 5 you filthy casual rax do you know de wey` | `paramTypes[index + 1 % 2]` evaluates to `paramTypes[index + 1]`. For index 1 this reads past the array (it lands on `isPointer`, which counts as a register), so no operand size is added. The result is `sub [rax], 0x5`, which gas rejects as ambiguous. | Uses `(index + 1) % 2`, the other parameter. The pointer gets the bully operand size, e.g. `sub BYTE PTR [rax], 0x5`. |
| 4 | `compiler/translator/translator.c:289` | Any macOS output (`killParent`, used by martyrdom) | `mov rax, 60` is the Linux `exit` number. On macOS it is not exit (the other two syscalls in the block use macOS numbers). | `mov rax, 0x2000001` (macOS `exit`) on macOS. Linux is unchanged. |
| 5 | `compiler/translator/translator.c:301-303` | Bully mode, executable output, no `main` function, target **Windows** | The injected `main` contains the martyrdom code, which references `killParent` and `.LsigStruct` (never emitted on Windows) and uses macOS syscall numbers. Linking always fails. | On Windows the injected `main` has no martyrdom code. Martyrdom is unsupported on Windows, as in the regular `main` path at `translator.c:319-330`. |
| 6 | `compiler/translator/translator.c:301-302` | Same as #5, target **macOS** | The injected entry point is named `main`, but macOS needs `_main` (which `mainFunctionExists`, `functions.c:52-57`, checks for). Linking fails. | The injected entry point is `_main` on macOS. |
| 7 | `compiler/analyser/randomCommands.c:88` | "perfectly balanced as all things should be" with several input files, one of them without code (bully mode accepts empty files), e.g. `-fcompile-mode bully -S -o x.S empty.memeasm pb.memeasm` | `rand() % 0` followed by `parsedCommands[...]` of a NULL array crashes the compiler (SIGFPE on x86-64, SIGSEGV on the aarch64 reference). | A randomly chosen file without lines is skipped and the loop draws again. |
| 8 | `compiler/analyser/randomCommands.c:83` | Bully mode, "perfectly balanced" in a program where more lines were already dropped by bully fixes than will remain, e.g. `tests/frontend_cases/fixes/perfectly_balanced_hang.memeasm` (4× orphaned `fuck go back` + 1× perfectly balanced) | The loop wants more lines than are still translatable and never ends: the compiler hangs. | The loop stops once every translatable line is deleted, i.e. it deletes `min(linesToBeDeleted, translatable lines)`. Results are unchanged whenever the C code terminated. |
| 9 | `compiler/analyser/parameters.c:545` | Bully mode, non-`mov` 64-bit arithmetic with a decimal that is not sign-extendable from 32 bits, e.g. `rbx units are ready, with 77777777777 more well on the way` (see `tests/frontend_cases/bully/pointer_second_and_truncation.memeasm`) | The decimal string is cut at index `computedIndex % 32`. If the number is shorter, this writes past the allocation (out-of-bounds write, which can also corrupt the heap so that the compiler aborts with `free(): invalid size`) and keeps the number. If the cut leaves more than 32 bits, the number stays too large. Either way gas rejects `add rbx, 0x121BEAB471`. | Truncates at the same index (when it is inside the string), then drops further trailing digits until the number fits into a sign-extended 32-bit immediate (`add rbx, 0x2E5BF271`). Numbers that the C cut already made valid are unchanged. |
| 10 | `compiler/memeasm.c:38-40` | `-h` | The usage lines still advertise `[-i \| -d]`, although `-i` was removed on purpose (upstream commit e99f0b3 "memeasm.c: Removed -i"; the info log level has no users), and `-i` is rejected as an unknown option. | The usage lines say `[-d]`. `-i` stays rejected, as the author intended. (The help page has further changes for the new command line, see "Command line".) |
| 11 | `compiler/analyser/parameters.c:119-122` (escape table), `:207-216` (`translateCharacter`), `:352-367` (ASCII codes) | Character parameters `\v`, `\` and ASCII codes with leading zeros, e.g. `what can I say except \v`, `what can I say except \`, `what can I say except 010` | The parameter text goes to gas unchanged: `'\v'` (gas has no `\v` escape: 0x76 'v'), `'\'` (gas reads an escaped quote: 0x27) and `010` (octal to gas: 8, although the analyser validated it as decimal 10). | The real value is written as a decimal number: 11, 92, 10 (`0065` → 65). Both paths. |
| 12 | `compiler/compiler.c:172-186` | `look at what x needs to mimic a fraction of y` | `cqo` runs before the dividend is loaded, so it sign-extends the *old* rax: the quotient is wrong (or the division traps) whenever their signs differ. With the dividend in `rdx`, the final `pop rdx` restores the old rdx, so the division has no effect at all (`rdx = 12345` divided by 10 stays 12345). | The dividend is pushed, loaded into rax, then sign-extended; the quotient is stored back into its stack slot and the registers are restored. Truncating signed division for every register combination (dividend or divisor in rax/rdx, a pointer dividend); the quotient replaces the dividend also when it is rdx, and rdx is otherwise preserved, as before. Both paths. |
| 13 | `compiler/compiler.c:188-207` | `x UNLIMITED POWER y` with y = 1 or y < 0 | The exponent is decremented twice before the first test, so y = 1 (and every negative y) loops about 2^64 times. | Square-and-multiply (at most 64 iterations). For y < 0 the integer result is 1 if x = 1, 1 or -1 (by parity of y) if x = -1, and 0 otherwise. Both paths. |
| 14 | `compiler/analyser/parameters.c:300` | A decimal outside the 64-bit range, e.g. `rax is brilliant, but I like 99999999999999999999` | `strtoll` saturates silently to INT64_MAX/INT64_MIN. | noob/obfuscated: compile error `invalid parameter provided: "…" does not fit into 64 Bits`. Bully: replaced by `computedIndex % 256`, like the other bully replacements of numbers (`parameters.c:533`). |
| 15 | `compiler/compiler.c:342-346` | `it's over 9000` with an 8-bit operand (`al`, `r8b`, or a bully pointer that got the size BYTE) | `cmp al, 9000`: gas truncates 9000 to 0x28 (40) with a warning that `gcc -w` hides, so values above 40 pass. | An 8-bit value is never over 9000: the command is a plain `hlt`. Both paths. No other truncation remains: register/immediate sizes are checked by `parameters.c:516-550`, and bully immediates next to a randomly sized pointer are truncated explicitly (#18). |
| 16 | `compiler/translator/translator.c:300-304` | Bully mode, executable, a program with no code at all (e.g. an empty file) | The injected `main` has nothing to fall into except `writechar`, which prints a character and returns an uninitialised rax as exit code. | After the martyrdom code (which is kept, see "Possible bugs, kept") the injected `main` returns 0 (`xor rax, rax` / `ret`). Executables only, so not visible in `-S`. |
| 17 | `compiler/translator/translator.c:355-412` | Windows, any print or read (`what can I say except`, `let me in. LET ME IIIIIIIIN`) | `sub rsp, 32` reserves only the shadow space, so the 5th API argument (`mov QWORD PTR [rsp + 32], 0`) overwrites the saved r9. r10 and r11, which the Windows API may clobber, are not saved. | `push rcx, rax, rdx, r8, r9, r10, r11` / `sub rsp, 48` (shadow space plus the 5th argument slot, 16-byte alignment unchanged) and the reverse on exit: prints do not modify any register, as on Linux. Both paths. |
| 18 | `compiler/analyser/parameters.c:556-585` (checks 3/4, skipped in bully mode), `compiler/translator/translator.c:166-178` | Parameter combinations the analyser accepts but gas rejects (`tests/frontend_cases/fixes/{high_byte_with_*,pointer_*,push_imm*}.memeasm`): a pointer whose base is not a 64/32-bit register (`al do you know de wey`); a pointer where the instruction cannot take memory (memory-to-memory `sneak 100`, the divisor of a division, the exponent of a power, `rdrand`, a label name); a pointer in a one-operand command (`not`, `upvote`, `downvote`, `upgrades`, `they had us…`, `it's over 9000`: ambiguous operand size); `ah`/`bh`/`ch`/`dh` together with a register that needs a REX prefix (`spl`, `bpl`, `sil`, `dil`, `r8b`-`r15b`, or a pointer based on `r8`-`r15`); `stonks` with a decimal that is not sign-extendable from 32 bits | gas fails (`ambiguous operand size`, `invalid use of register`, `too many memory references`, `can't encode register 'ah' in an instruction requiring REX prefix`, `operand type mismatch`). | noob/obfuscated: a compile error at the line, `invalid parameter combination: a pointer must be a 64 or 32 bit register` / `…: this command cannot use a pointer here` / `…: operand size unknown` (the message of check 4) / `…: 'ah' cannot be combined with 'spl'` / `…: pushed decimal numbers must be sign-extendable from 32 Bits`. Bully mode never produces an assembler error: an unusable pointer is dropped (`isPointer = 0`, as the C code does for decimals and characters), a one-operand pointer gets the random bully operand size of #3 (an immediate stored through it is truncated to that size), the high register is replaced by a random non-high 8-bit register, and pushed decimals are shortened like #9. An enumeration of every command with 54 parameter kinds in every position (registers of all sizes, high/REX bytes, decimals around every size limit, characters, pointers), in noob and bully mode (three seeds) for Linux and Windows, checks that every accepted line assembles without warnings and that `-O` produces the same `.text` bytes. |
| 19 | `compiler/compiler.c:69` | Calling a function whose name gas reads as something else: `$`, `.`, a register name (`rax`, `r8d`, `xmm0`, …), an operator (`and`, `not`, …) or a size/segment keyword (`byte`, `near`, `fs`, `rip`, …), e.g. `$: whomst has summoned the almighty one` | `call $` calls the current address (endless recursion, SIGSEGV), `call rax` is an indirect call, `call byte` a call to address 1. | `call "$"`: gas's quoted symbol syntax. The function label and `.global` keep the plain name (gas reads those correctly), and every name that worked before is unchanged. The direct path calls the same quoted symbol. |
| 20 | `compiler/translator/translator.c:472` | `-O-s` for macOS | `.align 536870912`: Apple's `as` (and llvm-mc) read `.align n` as 2^n on Mach-O and reject n >= 32 ("invalid alignment value"), so `-O-s` never assembled for macOS. | `.p2align 29` (the intended 512 MiB) for macOS, in `-S` and on the direct path. Linux and Windows keep `.align 536870912`. |

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

- The `-2`/`-3` filler and `69420` optimisation levels (C: `-O-2`, `-O-3`, `-O69420`) behave
  exactly as before, including `movups [rsp + 8]`
  writing above the stack pointer.
- Bully mode inserts the martyrdom code into the injected `main` even with `--no-martyrdom`
  (C: `-fno-martyrdom`, `translator.c:303`). This is kept on Linux and macOS.
- Multi-line comment keywords (`commands.h`) are defined but never implemented. Kept.
- `x UNLIMITED POWER y` with `x` = `rsp` overwrites the stack pointer while the stack is in
  use (C: `push rsp` / `imul rsp, [rsp]`; now `push rsp` / `mov rsp, 1`). Using rsp as a
  number is not meaningful, so this stays unsupported.
- The confused stonks label is always emitted at a random line of every file, even when no
  `confused stonks` is used (`randomCommands.c:36-45` always chose the line).
- Bully mode's injected fake functions (for orphaned commands) get no `_` prefix on macOS.
  They are local helper names, so this is harmless.

## Intentional changes (not bugs)

### Debug information: DWARF instead of STABS

- `-g` now produces DWARF line information instead of STABS. In the `-S` text, the
  `.stabs`/`.stabn` directives, the `.Lcmd_N`/`.Lret_*` labels and the `.LEOF` end marker
  (with its preceding blank line) are gone. Each input file gets
  `.file N "/absolute/path"` where the `N_SO` stab used to be, and every translated command,
  including function definitions, is preceded by `\t.loc N line`. The definition's row covers
  the martyrdom prologue of `main`, so `break main` resolves to main's line instead of the
  previous function's `ret`.
- The helper code after the program (`writechar`, `readchar`) is preceded by `\t.loc N 0`
  (`debug_line(file, 0)` on the direct path), so it is not attributed to the last source
  line. gas drops `.loc` rows with line 0 (binutils `dwarf2_gen_line_info`), so only objects
  from the direct path contain that row; gdb ignores line-0 rows when looking up lines, lldb
  and addr2line honour them. `killParent` comes before the first row and has no line
  information in either case.
- With several input files, C's `-g` output did not assemble at all (each file defined the
  same `.Lcmd_N` labels). DWARF has no such labels.
- On the direct Emitter path (`-O` / executables) the frontend calls `set_debug_meta`,
  `debug_file`, `debug_line` and `debug_function_begin`/`end` instead.
- The `-g` notes for Windows, macOS and bully mode are unchanged. The help line now reads
  "Currently, only the DWARF format is supported (Linux-only)".
- The parity script strips both debug formats before diffing.

### Command line: plain clap instead of getopt_long_only

The command line is parsed by clap with its ordinary semantics; the getopt_long_only
emulation is gone. Old → new spelling:

| C (getopt_long_only) | Rust (clap) |
|----------------------|-------------|
| `-o <file>` | `-o <file>`, `--output <file>` |
| `-S` | `-S`, `--assembly` |
| `-O` | `-O`, `--object` |
| `-O-1`, `-O-2`, `-O-3`, `-O-s`, `-O69420` | `--opt-level=-1`, `=-2`, `=-3`, `=-s`, `=69420` (also `--opt-level -1`) |
| `-fcompile-mode <mode>` / `-fcompile-mode=<mode>` | `--compile-mode <mode>` / `--compile-mode=<mode>` (`noob`, `bully`, `obfuscated`; default `noob`) |
| `-fno-martyrdom` | `--no-martyrdom` |
| (none: one C build per OS) | `--target <linux\|macos\|windows>` (default: the host OS) |
| `-d` | `-d`, `--debug` |
| `-g` | `-g`, `--debug-info` |
| `-h`, `--help` | `-h`, `--help` |
| `-v` | `-v`, `--version` |

Short options can still be clustered and take attached values (`-dSo out`, `-oout`), `--`
ends the options, options may follow the input files, and several input files are accepted.
`--target` replaces the C `#ifdef LINUX/MACOS/WINDOWS` builds; it selects the target
operating system at run time.

What changed beyond the spelling:

1. **Removed spellings.** `-O-1` … `-O69420`, `-fcompile-mode`, `-fno-martyrdom`, single-dash
   long options (`-help`, `-output`) and abbreviations of long options (`-fc`, `--comp`) are
   gone. `-O` is only the object-file flag: `-O-1` is now the error
   `unexpected argument '--' found`.
2. **All argument errors come from clap** and are found before anything is acted on: unknown
   options, missing values (`-S -o`), invalid values (`--compile-mode foo`,
   `--opt-level=-7`, `--target amiga`, listing the possible values) and conflicts. They are
   printed on stderr in clap's format (`error: unexpected argument '-x' found`, a tip and a
   usage line naming the program `memeasm`), exit code 1. C printed the getopt message plus
   `Error: Unknown option provided` / `Invalid optimisation level specified: …` /
   `Error: invalid compile mode (…)` / `Error: invalid target (…)`, and handled options one
   by one, so `-h -x` used to show the
   help page and `-v -O-7` the version; now both are errors.
3. **`-S` and `-O` conflict** (`error: the argument '--assembly' cannot be used with
   '--object'`, exit 1). In C the last one won.
4. **Repeated options:** the last occurrence wins for every option (`--opt-level=-1
   --opt-level=69420` is `69420`, as in C). A repeated `-g` on Windows/macOS now prints its
   note once (C printed it per occurrence).
5. `-h`/`--help` and `-v`/`--version` exit with code 0 and take precedence over missing input
   or output files. Option values that are not valid UTF-8 are rejected by clap (C took them
   as bytes); input and output file names may still be arbitrary bytes. `POSIXLY_CORRECT` is
   not honoured.
6. **Help page:** same layout and exit code 0, with these lines changed: the usage lines say
   `inputFile...` (several files) and `(-v | --version)`, the optimisation lines read
   `--opt-level=-1` … `--opt-level=69420`, then
   ` --compile-mode=<noob|bully|obfuscated> - Change the compile mode (default: noob)`,
   the `-g` line mentions DWARF (see above), ` --no-martyrdom`, the `[-d]` usage fix (#10)
   and the new last line
   ` --target=<linux|macos|windows> - Change the target operating system (default: <host>)`.

### Randomness

The C compiler seeded `rand()` with `time(NULL)` for the confused stonks label and for
"perfectly balanced as all things should be" (`randomCommands.c`), so two runs in the same
second made the same choices and a frozen clock (libfaketime) reproduced them. The Rust
compiler uses one OS-seeded generator per compilation (std's `RandomState` keys hashing a
counter, `src/frontend/random.rs`). The choices follow the same rules (one random line per
file for the label; random files and lines until enough lines are deleted, with fixes #7/#8),
but they are no longer tied to the clock, cannot be reproduced and differ from glibc's/MSVCRT's
`rand()` sequences. `tests/frontend_parity.sh` therefore removes the label from both outputs and
checks that every `jmp` to it has exactly one definition; for "perfectly balanced" programs the
Rust output must be a line-subsequence of the reference output of the program without those
lines, and the `-d` lines that show random choices are masked. `tests/e2e/run.py` compares the
Rust results with the set of reference results sampled at several frozen times.

### Other known divergences

- The `-S` header no longer contains the compile date: `# Generated by the MemeAssembly
  compiler v1.6` instead of `… v1.6 on Mon Jan  1 00:00:00 2024`. The output is the same on
  every run (apart from the random choices above).
- `-O`/executable output no longer pipes text through gcc. The frontend drives
  `backend::emitter::Emitter` directly, instruction for instruction like the `-S` text.
  - Assembler errors are reported as gas would report them (`{standard input}: Assembler
    messages:` / `{standard input}: Error: ...`), but without line numbers and only the first
    one, followed by the usual "Assembling/linking failed..." message (exit 1).
  - The output file name is no longer passed through the shell. C ran
    `popen("gcc ... -o" + name)`, so `-o 'a b'` failed and `-o 'x;touch pwned'` executed a
    command. Errors writing it are reported perror-style (`cannot write d: Is a directory`)
    instead of by gcc/ld, followed by the "Assembling/linking failed..." message.
  - Executables are linked statically without libc by the backend's writers. At `main`, rdi,
    rsi and rdx hold argc/argv/envp and rax holds the address of `main`; every other
    register is 0 instead of whatever the C runtime left behind. Programs that read
    uninitialised registers can behave differently.
  - Windows executables are relocatable and use ASLR, as mingw's defaults do. Their PE
    `CheckSum` is 0 (Windows only checks it for drivers and system DLLs).
  - Object files (`-O`) have the same sections, symbols and relocations as
    gas / mingw-as / llvm-mc output, but are not byte-identical: no empty `.bss`, no COFF
    `.file` symbol or padding to the section alignment, Mach-O references to temporary labels
    are non-extern (section-relative) relocations, and `-g` DWARF uses inline strings without
    `.debug_str`/`.debug_line_str`/`.debug_aranges`.
  - Instructions are encoded by a small x86-64 encoder in Rust (`backend/emitter/x86.rs`),
    with the same instruction forms gas chooses. It is checked against a table of encodings
    generated with gas (`tests/emitter_golden.py`) and was checked against the fadec encoder
    it replaced.
  - The `-O-s` alignment padding after the final `ret` is filled with single-byte NOPs
    (`0x90`) instead of gas's multi-byte NOPs. The padding is never executed, so the
    program behaves the same, but `.text` differs byte for byte from gas's output.
- Write errors are no longer ignored: a `-S` output that cannot be written completely (e.g.
  `-o /dev/full`) gives `Failed to write output file: No space left on device` (exit 1), and
  a failed write to stdout gives `Failed to write to stdout: ...` (exit 1). If stdout is a pipe
  whose reader has gone away (`memeasm -h | head -1`), the compiler stops quietly with exit
  code 141, the status a shell reports for a process killed by SIGPIPE. C was killed by the
  signal itself; Rust ignores SIGPIPE, and the compiler no longer restores it.
- stdout is not buffered like stdio any more: every message is written (and flushed) as it is
  produced. When stdout and stderr go to the same file, messages therefore appear in the order
  they were produced; C's fully buffered stdout could appear after later stderr messages.
- On Windows hosts the compiler no longer emulates the Microsoft C runtime's text mode: stdout,
  stderr and `-S` files get `\n` line endings instead of `\r\n`, and a Ctrl-Z (0x1A) byte in an
  input file no longer ends the file. (`\r\n` input lines are accepted on every host, as
  before.) I/O error messages use the operating system's text (e.g. `Failed to open input
  file: The system cannot find the file specified.`) instead of the CRT's `strerror` text
  (`No such file or directory`); on Unix hosts they are unchanged.
- Bytes of input file names and lines are treated as unsigned (0-255) where they feed bully
  mode's and the random error messages' pseudo-random index (`computedIndex`). C used `char`,
  which is signed on x86 hosts, so for names or lines with non-ASCII bytes the bully
  replacements and error messages chosen on x86 hosts differ; on aarch64 (unsigned `char`)
  nothing changes.
- Decimal parameters and ASCII codes are parsed with `str::parse::<i64>` instead of
  `strtoll`/`strtol`. The only visible difference: a leading vertical tab (0x0B) no longer
  counts as whitespace (other leading ASCII whitespace still does). Out-of-range numbers are
  still detected (#14).
- Input lines are read into heap memory: C copied each line into a VLA
  (`fileParser.c:170`), so lines larger than the stack (about 8 MB) crashed the compiler.
