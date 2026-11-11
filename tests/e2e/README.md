# End-to-end suite: new `memeasm` vs. the C reference

    NEW=$PWD/target/release/memeasm WINE_RUN=/path/to/run-exe.sh tests/e2e/run.sh [FILTER ...]

`gen_programs.py` writes the corpus (`programs/`, `manifest.json`). `run.py` compiles every program with
both compilers and compares the results per check kind (`lin-asm`, `lin-obj`, `lin-exe`, `win-asm`,
`win-obj`, `win-exe`, `win-run`, `martyrdom`, ...). Results go to `out/` (`details.txt`,
`results.json`). Reference outcomes are recomputed live when `/opt/memeasm-ref` exists. With
`--expected` they are read from `expected/` instead (written by `--update-expected`). `out/` and
`expected/` are not tracked.

## Statuses

| status  | meaning |
|---------|---------|
| PASS    | same as the reference |
| FIXED   | differs from the reference exactly as a documented fix (docs/rust-rewrite-changes.md) says it should |
| WARN    | equal after normalisation (encodings/addresses), or a random sample that could not be matched |
| CHANGED | the reference behaviour is undefined for this program (`correct: unspecified`) and the new one differs |
| SKIP    | not applicable (tool missing, check not meaningful for this program) |
| OLDBUG  | the new compiler reproduces a documented reference bug (fails; `--allow-old-bugs` turns it into PASS) |
| FAIL    | a real difference |
| ERROR   | a suite problem (e.g. the assembler self-check below failed, outdated `expected/`) |

`FAIL`, `OLDBUG` and `ERROR` make the run fail.

## How documented fixes are judged

* asm (`-S`): the reference text is rewritten by `tests/frontend_parity_whitelist.pl` before it is
  compared. The suite runs the script itself and gives it the same context that `tests/frontend_parity.sh`
  gives it: target, compile mode, `-O` flag, and `BULLY_SIZE` taken from the new output. The rules
  live only in that script. With `-g`, both sides have their STABS/DWARF directives stripped, as in
  the parity script's `normalize_asm`.
  * new == whitelisted != reference: FIXED.
  * new == unrewritten reference: OLDBUG.
  * anything else: FAIL.
* obj (`-O`): the whitelisted reference `-S` text is assembled with the cross gas, then compared with
  the new object using the same objdump info (globals, `.data`, disassembly), plus the mingw link.
  * The cross gas is `x86_64-linux-gnu-as -W --64` or `x86_64-w64-mingw32-as -W`, which is what the
    reference's `gcc -w -c -x assembler -` runs.
  * Self-check: gas applied to the unrewritten reference `-S` text must reproduce the reference's own
    `-O` object. Otherwise the check is ERROR.
  * If the reference `-O` fails in gas but the whitelisted text assembles to the new object, the
    check is FIXED.
* exe/run: programs that hit a reference runtime bug carry a `ref_bug` entry in `gen_programs.py`.
  It has `bug_kinds` (the checks the bug shows in) and a precise `correct` spec. Usually that spec is
  an *oracle* program, built with the reference, that prints the correct result. The other spec kinds
  are `diagnostic`, `unspecified` and so on.
  * A difference on a kind outside `bug_kinds` is judged like any other program, so it stays FAIL.
  * A FIXED result on a kind inside `bug_kinds` still has to satisfy the `correct` spec.

Mutation check: a wrapper that turns `idiv` into `div` in the new compiler's output makes 38
asm/obj checks FAIL, including the ones on `cmd/idiv` and `bugs/idiv_sign`.

## Process hygiene

Every child process runs in its own session with a timeout and rlimits. Its process group is
SIGKILLed when it finishes or times out. Remaining groups are killed at exit and on
SIGTERM/SIGINT/SIGHUP. The C reference hangs on some inputs.

## Known expected non-PASS results

* `lin-exe:bully/mixed` and its `modes/` variants are CHANGED, because the program is undefined.
  `stonks al` pushes `rax` without a matching pop, and `blub` is a `syscall`. As a result, `ret`
  returns to whatever start-up left in `rax`. In the new static executables that value is `&main`,
  so the program prints `xy` twice before it crashes. The reference prints it once.
* `win-run:cmd/rdrand` is SKIP. box64 does not implement `rdrand r16` and raises SIGILL, and the same
  happens for a mingw-gcc build of the same text.
* Random programs: when an asm sample cannot be matched within the sampling budget, the result is WARN.
