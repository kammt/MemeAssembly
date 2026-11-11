#!/usr/bin/env bash
# Compares the Rust frontend against the original C compiler.
#
# For every input × compile mode × optimisation level × extra flags × target it runs both
# compilers with `-S` and diffs the generated assembly (minus the timestamp line), stdout,
# stderr and the exit code. Erroneous inputs are additionally run without `-S` (the compile
# errors happen before any code generation). A set of command line cases checks the CLI-level
# messages of memeasm.c that were kept.
#
# Usage: tests/frontend_parity.sh [-v] [filter-regex]
#   MEMEASM   Rust compiler          (default: target/debug/memeasm)
#   REF_LINUX C compiler, -D LINUX   (default: /opt/memeasm-ref/memeasm-linux)
#   REF_WIN   C compiler, -D WINDOWS (default: /opt/memeasm-ref/memeasm-win)
#   REF_MAC   C compiler, -D MACOS   (optional; build with
#             gcc -std=gnu17 -D MACOS -O2 -o memeasm-mac $(FILES from the Makefile))
#
# Both binaries are started with argv[0] = "memeasm" and from the repository root, so file
# names in messages are identical. Confused stonks / perfectly balanced use time(NULL) as seed,
# so each pair of runs is retried until both happened within the same second.
#
# Intentional divergences that are filtered/skipped here:
#  * -g: the C compiler writes STABS, the Rust one gas DWARF directives (.file/.loc). The
#    debug directives are removed from both outputs before diffing.
#  * the help page (-h) and clap's argument errors are not compared.
#  * bugs of the C compiler that the Rust port fixes (docs/rust-rewrite-changes.md): the
#    reference output is rewritten by tests/frontend_parity_whitelist.pl, one rule per fix;
#    cases where the C compiler crashes or hangs are checked separately (check_fixed).

set -u
ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT" || exit 2

VERBOSE=0
if [ "${1:-}" = "-v" ]; then VERBOSE=1; shift; fi
FILTER=${1:-.}

MINE=${MEMEASM:-$ROOT/target/debug/memeasm}
REF_LINUX=${REF_LINUX:-/opt/memeasm-ref/memeasm-linux}
REF_WIN=${REF_WIN:-/opt/memeasm-ref/memeasm-win}
REF_MAC=${REF_MAC:-}

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/ref" "$WORK/mine"

PASS=0
FAIL=0
SKIP=0
FAILED_CASES=()

ref_for_target() {
    case "$1" in
        linux) echo "$REF_LINUX" ;;
        windows) echo "$REF_WIN" ;;
        macos) echo "$REF_MAC" ;;
    esac
}

# Directory with the cross gcc a reference compiler pipes its assembly into.
ref_gcc_dir() {
    case "$1" in
        linux) echo /opt/memeasm-ref/bin-linux ;;
        windows) echo /opt/memeasm-ref/bin-win ;;
        macos) echo /nonexistent ;;
    esac
}

# run <dir> <PATH prefix> <binary> <args...>: runs with argv[0]=memeasm, output in
# <dir>/{stdout,stderr,code}
run() {
    local dir=$1 path=$2 bin=$3
    shift 3
    rm -f "$dir/out.S"
    (PATH="${path:+$path:}$PATH" exec -a memeasm "$bin" "$@") >"$dir/stdout" 2>"$dir/stderr" </dev/null
    echo $? >"$dir/code"
}

# Removes the timestamp and (for -g) the debug directives of both debug formats.
normalize_asm() {
    [ -f "$1" ] || return 0
    LC_ALL=C sed -e '2s/ on .*$//' \
        -e '/^$/{N;/^\n\.LEOF:$/d;P;D}' \
        -e '/^\.stabs /d' -e '/^\t\.stabn /d' -e '/^\.stabn /d' \
        -e '/^\t\.Lcmd_[0-9]*:$/d' -e '/^\t\.Lret_.*:$/d' -e '/^\.LEOF:$/d' \
        -e '/^\.file [0-9]* "/d' -e '/^\t\.loc [0-9]* [0-9]*$/d' "$1"
}

# -d prints parameters[0] of commands without parameters, which is an uninitialised pointer in
# C (garbage or a stale string); the Rust port prints "(null)". Those lines are masked.
normalize_stdout() {
    LC_ALL=C sed -e 's/^\t\tComparing against parameter .*$/\t\tComparing against parameter <masked>/' "$1"
}

# The reference -S output with the intentional differences applied (see the whitelist script).
# Uses CASE_TARGET, CASE_MODE and CASE_OPT of the current case.
expected_asm() {
    [ -f "$WORK/ref/out.S" ] || return 0
    local size
    size=$(grep -m1 -oE '(BYTE|WORD|DWORD|QWORD) PTR \[[a-z0-9]+\]' "$WORK/mine/out.S" 2>/dev/null | cut -d' ' -f1)
    normalize_asm "$WORK/ref/out.S" |
        BULLY_SIZE=$size perl "$ROOT/tests/frontend_parity_whitelist.pl" "$CASE_TARGET" "$CASE_MODE" "$CASE_OPT"
}

compare_dirs() {
    local name=$1
    local ok=1
    cmp -s <(normalize_stdout "$WORK/ref/stdout") <(normalize_stdout "$WORK/mine/stdout") || ok=0
    cmp -s "$WORK/ref/stderr" "$WORK/mine/stderr" || ok=0
    cmp -s "$WORK/ref/code" "$WORK/mine/code" || ok=0
    local ref_exists=0 mine_exists=0
    [ -f "$WORK/ref/out.S" ] && ref_exists=1
    [ -f "$WORK/mine/out.S" ] && mine_exists=1
    [ $ref_exists = $mine_exists ] || ok=0
    if ! cmp -s <(expected_asm) <(normalize_asm "$WORK/mine/out.S"); then ok=0; fi
    if [ $ok = 1 ]; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        FAILED_CASES+=("$name")
        if [ $VERBOSE = 1 ]; then
            echo "=== FAIL: $name"
            diff "$WORK/ref/code" "$WORK/mine/code" | head -5
            diff <(normalize_stdout "$WORK/ref/stdout") <(normalize_stdout "$WORK/mine/stdout") | head -20
            diff "$WORK/ref/stderr" "$WORK/mine/stderr" | head -20
            diff <(expected_asm) <(normalize_asm "$WORK/mine/out.S") | head -20
        fi
    fi
}

# check [--needs-ref-failure] <name> <target> <args...>
# (args may contain OUT, which is replaced by the output path). With --needs-ref-failure the
# case is skipped if the reference got as far as running gcc (code generation is the backend's
# job and compared elsewhere).
check() {
    local needs_failure=0
    if [ "$1" = --needs-ref-failure ]; then needs_failure=1; shift; fi
    local name=$1 target=$2
    shift 2
    [[ "$name" =~ $FILTER ]] || return 0
    local ref
    ref=$(ref_for_target "$target")
    if [ -z "$ref" ]; then SKIP=$((SKIP + 1)); return 0; fi
    local ref_args=() mine_args=("-ftarget=$target") a prev=
    CASE_TARGET=$target CASE_MODE=noob CASE_OPT=
    for a in "$@"; do
        case "$prev" in -fcompile-mode | -fc) CASE_MODE=$a ;; esac
        case "$a" in
            -fcompile-mode=*) CASE_MODE=${a#*=} ;;
            -O-[0-9s] | -O69420) CASE_OPT=$a ;;
        esac
        prev=$a
        ref_args+=("${a//OUT/$WORK/ref/out.S}")
        mine_args+=("${a//OUT/$WORK/mine/out.S}")
    done
    local attempt
    for attempt in 1 2 3 4 5; do
        local t0 t1
        t0=$(date +%s)
        run "$WORK/ref" "$(ref_gcc_dir "$target")" "$ref" "${ref_args[@]}"
        run "$WORK/mine" "" "$MINE" "${mine_args[@]}"
        t1=$(date +%s)
        [ "$t0" = "$t1" ] && break
    done
    if [ $needs_failure = 1 ] && { [ "$(cat "$WORK/ref/code")" = 0 ] || grep -q "^gcc exited unexpectedly" "$WORK/ref/stderr"; }; then
        SKIP=$((SKIP + 1))
        return 0
    fi
    compare_dirs "$name"
}

# check_fixed <name> <target> <ref-outcome> <args...>: a C bug where the reference crashes or
# hangs, so nothing can be diffed. Passes if the reference shows the bug (<ref-outcome> "hang":
# killed after 5 s; "crash": died from a signal; "crash-or-ok": a signal or success, for random
# crashes) and the Rust compiler succeeds and writes OUT.
check_fixed() {
    local name=$1 target=$2 expect=$3
    shift 3
    [[ "$name" =~ $FILTER ]] || return 0
    local ref
    ref=$(ref_for_target "$target")
    if [ -z "$ref" ]; then SKIP=$((SKIP + 1)); return 0; fi
    local ref_args=() mine_args=("-ftarget=$target") a
    for a in "$@"; do
        ref_args+=("${a//OUT/$WORK/ref/out.S}")
        mine_args+=("${a//OUT/$WORK/mine/out.S}")
    done
    rm -f "$WORK/mine/out.S"
    local ref_code
    ref_code=$(exec 2>/dev/null; timeout 5 "$ref" "${ref_args[@]}" </dev/null >/dev/null; echo $?)
    "$MINE" "${mine_args[@]}" >/dev/null 2>"$WORK/mine/stderr" </dev/null
    local mine_code=$?
    local ok=1
    case "$expect" in
        hang) [ $ref_code = 124 ] || ok=0 ;;
        crash) [ $ref_code -gt 128 ] && [ $ref_code != 124 ] || ok=0 ;;
        crash-or-ok) [ $ref_code -gt 128 ] && [ $ref_code != 124 ] || [ $ref_code = 0 ] || ok=0 ;;
    esac
    [ $mine_code = 0 ] && [ -s "$WORK/mine/out.S" ] || ok=0
    if [ $ok = 1 ]; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        FAILED_CASES+=("$name")
        [ $VERBOSE = 1 ] && echo "=== FAIL: $name (reference exit $ref_code, rust exit $mine_code)"
    fi
}

TARGETS=(linux windows)
[ -n "$REF_MAC" ] && TARGETS+=(macos)

OK_INPUTS=(examples/*.memeasm examples/*/*.memeasm .github/workflows/*.memeasm tests/frontend_cases/ok/*.memeasm)
ERR_INPUTS=(tests/frontend_cases/err/*.memeasm)
BULLY_INPUTS=(tests/frontend_cases/bully/*.memeasm)
OPTS=("" -O-1 -O-2 -O-3 -O-s -O69420)
EXTRA=("-g" "-fno-martyrdom" "-d" "-g -fno-martyrdom -O-1" "-d -O69420")

for target in "${TARGETS[@]}"; do
    for mode in noob bully obfuscated; do
        for input in "${OK_INPUTS[@]}" "${ERR_INPUTS[@]}" "${BULLY_INPUTS[@]}"; do
            for opt in "${OPTS[@]}"; do
                # shellcheck disable=SC2086
                check "$target/$mode/$input/S$opt" "$target" -fcompile-mode "$mode" $opt -S -o OUT "$input"
            done
            for extra in "${EXTRA[@]}"; do
                # shellcheck disable=SC2086
                check "$target/$mode/$input/S $extra" "$target" -fcompile-mode "$mode" $extra -S -o OUT "$input"
            done
        done
        # multiple files
        check "$target/$mode/multi/ab" "$target" -fcompile-mode "$mode" -S -o OUT tests/frontend_cases/multi/a.memeasm tests/frontend_cases/multi/b.memeasm
        check "$target/$mode/multi/ab -d -g" "$target" -fcompile-mode "$mode" -d -g -S -o OUT tests/frontend_cases/multi/a.memeasm tests/frontend_cases/multi/b.memeasm
        check "$target/$mode/multi/abc" "$target" -fcompile-mode "$mode" -S -o OUT tests/frontend_cases/multi/a.memeasm tests/frontend_cases/multi/b.memeasm tests/frontend_cases/multi/c_dup.memeasm
        check "$target/$mode/multi/github" "$target" -fcompile-mode "$mode" -S -o OUT .github/workflows/multiple_files/*.memeasm
        check "$target/$mode/multi/empty+ok" "$target" -fcompile-mode "$mode" -S -o OUT tests/frontend_cases/err/empty.memeasm tests/frontend_cases/ok/simple.memeasm
        # Fixed C bugs with a hanging/crashing reference (bully mode accepts these inputs)
        if [ "$mode" = bully ]; then
            for opt in "" -d; do
                # shellcheck disable=SC2086
                check_fixed "$target/$mode/fixed/perfectly-balanced-hang$opt" "$target" hang -fcompile-mode bully $opt -S -o OUT tests/frontend_cases/fixes/perfectly_balanced_hang.memeasm
                # shellcheck disable=SC2086
                check_fixed "$target/$mode/fixed/perfectly-balanced-empty-file$opt" "$target" crash-or-ok -fcompile-mode bully $opt -S -o OUT tests/frontend_cases/err/empty.memeasm tests/frontend_cases/fixes/perfectly_balanced_twice.memeasm
            done
        fi
        # Without -S, compile errors (incl. "no main function" for executables) stop before
        # code generation, so these runs compare exactly as well.
        if [ "$mode" != bully ]; then
            for input in "${ERR_INPUTS[@]}" tests/frontend_cases/ok/no_main.memeasm; do
                check --needs-ref-failure "$target/$mode/$input/exe" "$target" -fcompile-mode "$mode" -o OUT "$input"
                check --needs-ref-failure "$target/$mode/$input/obj" "$target" -fcompile-mode "$mode" -O -o OUT "$input"
            done
        fi
    done

    # getopt_long_only spellings that must keep working
    in=tests/frontend_cases/ok/simple.memeasm
    check "$target/cli/single-dash-long" "$target" -fcompile-mode=obfuscated -S -output OUT "$in"
    check "$target/cli/abbrev" "$target" -fc bully -out OUT -S "$in"
    check "$target/cli/double-dash" "$target" --output=OUT --fno-martyrdom -S "$in"
    check "$target/cli/cluster" "$target" -dSo OUT "$in"
    check "$target/cli/attached-o" "$target" -S -oOUT "$in"
    check "$target/cli/permute" "$target" "$in" -S -o OUT -O-3
    check "$target/cli/last-O-wins" "$target" -O-1 -O69420 -S -o OUT "$in"
    check "$target/cli/S-after-O" "$target" -O -S -o OUT "$in"
    check "$target/cli/terminator" "$target" -S -o OUT -- "$in"
    check "$target/cli/-g" "$target" -g -S -o OUT "$in"
    check "$target/cli/-g-bully" "$target" -g -fcompile-mode bully -S -o OUT "$in"
    check "$target/cli/-g-twice" "$target" -g -g -S -o OUT "$in"
    # memeasm.c's own CLI messages
    check "$target/cli/version" "$target" -v
    check "$target/cli/version-before-error" "$target" -v -O-7
    check "$target/cli/error-before-version" "$target" -O-7 -v
    check "$target/cli/no-output" "$target" "$in"
    check "$target/cli/no-input" "$target" -o OUT
    check "$target/cli/no-args" "$target"
    check "$target/cli/missing-file" "$target" -o OUT does-not-exist.memeasm
    check "$target/cli/missing-second-file" "$target" -S -o OUT "$in" does-not-exist.memeasm
    check "$target/cli/directory" "$target" -o OUT tests
    check "$target/cli/bad-O" "$target" -O-4 -o OUT "$in"
    check "$target/cli/bad-O-text" "$target" -Oabc -o OUT "$in"
    check "$target/cli/bad-O-trailing" "$target" -O-1x -o OUT "$in"
    check "$target/cli/bad-O-space" "$target" "-O -1" -S -o OUT "$in"
    check "$target/cli/bad-O-range" "$target" -O99999999999999999999999 -o OUT "$in"
    check "$target/cli/bad-mode" "$target" -fcompile-mode foo -o OUT "$in"
    check "$target/cli/g-then-bad-mode" "$target" -g -fcompile-mode foo -o OUT "$in"
done

echo "parity: $PASS passed, $FAIL failed, $SKIP skipped"
if [ $FAIL -gt 0 ]; then
    printf '  %s\n' "${FAILED_CASES[@]}" | head -50
    exit 1
fi
