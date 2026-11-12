#!/usr/bin/env bash
# Compares the Rust frontend against the original C compiler.
#
# For every input × compile mode × optimisation level × extra flags × target it runs both
# compilers with `-S` and diffs the generated assembly (minus the header comment), stdout,
# stderr and the exit code. Erroneous inputs are additionally run without `-S` (the compile
# errors happen before any code generation). A set of command line cases checks the CLI-level
# messages of memeasm.c that were kept.
#
# The cases are written with the C compiler's (getopt) option spelling; the Rust compiler gets
# the clap spelling of the same options (see new_spelling).
#
# Usage: tests/frontend_parity.sh [-v] [filter-regex]
#   MEMEASM   Rust compiler          (default: target/debug/memeasm)
#   REF_LINUX C compiler, -D LINUX   (default: /opt/memeasm-ref/memeasm-linux)
#   REF_WIN   C compiler, -D WINDOWS (default: /opt/memeasm-ref/memeasm-win)
#   REF_MAC   C compiler, -D MACOS   (optional; build with
#             gcc -std=gnu17 -D MACOS -O2 -o memeasm-mac $(FILES from the Makefile))
#
# Both binaries are started with argv[0] = "memeasm" and from the repository root, so file
# names in messages are identical.
#
# Random choices: the C compiler seeds rand() with time(NULL), the Rust compiler uses an
# OS-seeded generator, so their choices cannot be compared. The confused stonks label (emitted
# for every file) is removed from both outputs, and the Rust output must contain it exactly
# once for every `jmp` to it. For "perfectly balanced" programs the Rust output must be a
# line-subsequence of the (whitelisted) reference output of the same program without its
# "perfectly balanced" lines. With -d, the debug lines that show random choices are masked.
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

# The Rust compiler's spelling of a C compiler (getopt) argument.
new_spelling() {
    case "$1" in
        -fcompile-mode) echo --compile-mode ;;
        -fcompile-mode=*) echo "--compile-mode=${1#*=}" ;;
        -fno-martyrdom) echo --no-martyrdom ;;
        -ftarget=*) echo "--target=${1#*=}" ;;
        -O?*) echo "--opt-level=${1#-O}" ;;
        *) printf '%s\n' "$1" ;;
    esac
}

# set_args <target> <args...>: REF_ARGS / MINE_ARGS for the two compilers (OUT replaced by the
# output path of each side). Also sets CASE_TARGET, CASE_MODE, CASE_OPT and CASE_PB (an input
# uses "perfectly balanced").
set_args() {
    local target=$1 a prev=
    shift
    REF_ARGS=() MINE_ARGS=("--target=$target")
    CASE_TARGET=$target CASE_MODE=noob CASE_OPT= CASE_PB=0
    for a in "$@"; do
        case "$prev" in -fcompile-mode | -fc) CASE_MODE=$a ;; esac
        case "$a" in
            -fcompile-mode=*) CASE_MODE=${a#*=} ;;
            -O-[0-9s] | -O69420) CASE_OPT=$a ;;
            *.memeasm) grep -q 'perfectly balanced as all things should be' "$a" 2>/dev/null && CASE_PB=1 ;;
        esac
        prev=$a
        REF_ARGS+=("${a//OUT/$WORK/ref/out.S}")
        MINE_ARGS+=("$(new_spelling "${a//OUT/$WORK/mine/out.S}")")
    done
}

# Removes the header comment, the randomly placed confused stonks labels and (for -g) the debug
# directives of both debug formats.
normalize_asm() {
    [ -f "$1" ] || return 0
    LC_ALL=C sed -e '2s/ on .*$//' -e '/^[ \t]*\.LConfusedStonks_[0-9]*:[ \t]*$/d' \
        -e '/^$/{N;/^\n\.LEOF:$/d;P;D}' \
        -e '/^\.stabs /d' -e '/^\t\.stabn /d' -e '/^\.stabn /d' \
        -e '/^\t\.Lcmd_[0-9]*:$/d' -e '/^\t\.Lret_.*:$/d' -e '/^\.LEOF:$/d' \
        -e '/^\.file [0-9]* "/d' -e '/^\t\.loc [0-9]* [0-9]*$/d' "$1"
}

# -d prints parameters[0] of commands without parameters, which is an uninitialised pointer in
# C (garbage or a stale string); the Rust port prints "(null)". Those lines are masked, and so
# are the random choices. With "perfectly balanced", how often -O69420 skips a command depends
# on which lines were deleted, so that message is dropped too.
normalize_stdout() {
    local pb_skip=()
    [ "${CASE_PB:-0}" = 1 ] && pb_skip=(-e '/^\tCommand is not a function declaration, abort\.$/d')
    LC_ALL=C sed "${pb_skip[@]}" \
        -e 's/^\t\tComparing against parameter .*$/\t\tComparing against parameter <masked>/' \
        -e 's/^Chose random line for jump marker: [0-9]*$/Chose random line for jump marker: <N>/' \
        -e 's/^\x1b\[0m\(\tGenerated random line \)/\1/' \
        -e '/^\tGenerated random line [0-9]* in file [0-9]*$/d' \
        -e '/^\t\tLine was already generated$/d' -e '/^\t\tLine was added to the list$/d' "$1"
}

# Every confused stonks label that is jumped to must be defined exactly once, the others at
# most once.
stonks_labels_ok() {
    [ -f "$1" ] || return 0
    LC_ALL=C perl -ne '
        $def{$1}++ if /^[ \t]*\.LConfusedStonks_(\d+):[ \t]*$/;
        $use{$1} = 1 if /\bjmp \.LConfusedStonks_(\d+)$/;
        END { for (keys %use) { exit 1 if ($def{$_} // 0) != 1 } for (values %def) { exit 1 if $_ > 1 } }' "$1"
}

# is_subsequence <small> <big>: the non-blank lines of <small> are a subsequence of those of
# <big> (deleted lines leave blank or tab-only lines behind). The instructions -O-1/-2/-3 append
# to every command are removed from both first: the "perfectly balanced" line itself gets them
# too, but is blank in <big>.
is_subsequence() {
    LC_ALL=C perl -e '
        my $opt = shift;
        sub lines {
            open my $f, "<", $_[0] or exit 1;
            local $/;
            my $s = <$f> // "";
            $s =~ s/^\tnop\n//mg if $opt eq "-O-1";
            $s =~ s/^\tpush rax\n\tpop rax\n//mg if $opt eq "-O-2";
            $s =~ s/^\tmovups \[rsp \+ 8\], xmm0\n\tmovups xmm0, \[rsp \+ 8\]\n//mg if $opt eq "-O-3";
            return grep { /\S/ } split /^/m, $s;
        }
        my @big = lines($ARGV[1]);
        my $i = 0;
        for my $l (lines($ARGV[0])) { $i++ while $i < @big && $big[$i] ne $l; exit 1 if $i++ >= @big }' -- "$CASE_OPT" "$1" "$2"
}

# reference_without_pb <reference compiler>: writes the reference -S output for the current
# case's inputs without their "perfectly balanced" lines to $WORK/full.S (line numbers stay).
reference_without_pb() {
    local ref=$1 i=0 a f full_args=()
    rm -rf "$WORK/full" "$WORK/full.S"
    for a in "${REF_ARGS[@]}"; do
        if [ "$a" = "$WORK/ref/out.S" ]; then
            a=$WORK/full.S
        elif [[ $a == *.memeasm ]] && [ -f "$a" ]; then
            i=$((i + 1))
            f=$WORK/full/$i/$(basename "$a")
            mkdir -p "$WORK/full/$i"
            LC_ALL=C sed -e 's/^.*perfectly balanced as all things should be.*$//' "$a" >"$f"
            a=$f
        fi
        full_args+=("$a")
    done
    (PATH="$(ref_gcc_dir "$CASE_TARGET"):$PATH" exec -a memeasm "$ref" "${full_args[@]}") >/dev/null 2>&1 </dev/null
}

# The reference -S output ($1, default $WORK/ref/out.S) with the intentional differences
# applied (see the whitelist script). Uses CASE_TARGET, CASE_MODE and CASE_OPT of the current case.
expected_asm() {
    local file=${1:-$WORK/ref/out.S}
    [ -f "$file" ] || return 0
    local size
    # (power's own `QWORD PTR [rsp]` operands are not bully pointers)
    size=$(grep -oE '(BYTE|WORD|DWORD|QWORD) PTR \[[a-z0-9]+\]' "$WORK/mine/out.S" 2>/dev/null | grep -v -m1 '^QWORD PTR \[rsp\]$' | cut -d' ' -f1)
    normalize_asm "$file" |
        BULLY_SIZE=$size perl "$ROOT/tests/frontend_parity_whitelist.pl" "$CASE_TARGET" "$CASE_MODE" "$CASE_OPT"
}

compare_dirs() {
    local name=$1 ref=$2
    local ok=1
    cmp -s <(normalize_stdout "$WORK/ref/stdout") <(normalize_stdout "$WORK/mine/stdout") || ok=0
    cmp -s "$WORK/ref/stderr" "$WORK/mine/stderr" || ok=0
    cmp -s "$WORK/ref/code" "$WORK/mine/code" || ok=0
    local ref_exists=0 mine_exists=0
    [ -f "$WORK/ref/out.S" ] && ref_exists=1
    [ -f "$WORK/mine/out.S" ] && mine_exists=1
    [ $ref_exists = $mine_exists ] || ok=0
    stonks_labels_ok "$WORK/mine/out.S" || ok=0
    if [ $CASE_PB = 1 ] && [ $ref_exists = 1 ]; then
        reference_without_pb "$ref"
        is_subsequence <(normalize_asm "$WORK/mine/out.S") <(expected_asm "$WORK/full.S") || ok=0
    elif ! cmp -s <(expected_asm) <(normalize_asm "$WORK/mine/out.S"); then
        ok=0
    fi
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
            [ $CASE_PB = 1 ] && echo "(perfectly balanced: the Rust output must be a subsequence of the reference without those lines)"
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
    set_args "$target" "$@"
    run "$WORK/ref" "$(ref_gcc_dir "$target")" "$ref" "${REF_ARGS[@]}"
    run "$WORK/mine" "" "$MINE" "${MINE_ARGS[@]}"
    if [ $needs_failure = 1 ] && { [ "$(cat "$WORK/ref/code")" = 0 ] || grep -q "^gcc exited unexpectedly" "$WORK/ref/stderr"; }; then
        SKIP=$((SKIP + 1))
        return 0
    fi
    compare_dirs "$name" "$ref"
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
    set_args "$target" "$@"
    rm -f "$WORK/mine/out.S"
    local ref_code
    ref_code=$(exec 2>/dev/null; timeout 5 "$ref" "${REF_ARGS[@]}" </dev/null >/dev/null; echo $?)
    "$MINE" "${MINE_ARGS[@]}" >/dev/null 2>"$WORK/mine/stderr" </dev/null
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

# The assembler the -S output of <target> is meant for (none for macOS here).
gas_for_target() {
    case "$1" in
        linux) echo x86_64-linux-gnu-as ;;
        windows) echo x86_64-w64-mingw32-as ;;
    esac
}

# Pass/fail bookkeeping for the checks below: record <name> <ok> [message]
record() {
    if [ "$2" = 1 ]; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        FAILED_CASES+=("$1")
        [ $VERBOSE = 1 ] && echo "=== FAIL: $1 ${3:-}"
    fi
    return 0
}

# check_new_error <name> <target> <message> <args...>: input the C compiler accepted, although
# its output does not assemble or does not mean what the program says (docs: fixed bugs). The
# reference must succeed; the Rust port must fail with <message> (and only that error).
check_new_error() {
    local name=$1 target=$2 message=$3
    shift 3
    [[ "$name" =~ $FILTER ]] || return 0
    local ref
    ref=$(ref_for_target "$target")
    if [ -z "$ref" ]; then SKIP=$((SKIP + 1)); return 0; fi
    set_args "$target" "$@"
    run "$WORK/ref" "$(ref_gcc_dir "$target")" "$ref" "${REF_ARGS[@]}"
    run "$WORK/mine" "" "$MINE" "${MINE_ARGS[@]}"
    local ok=1
    [ "$(cat "$WORK/ref/code")" = 0 ] || ok=0
    [ "$(cat "$WORK/mine/code")" = 1 ] || ok=0
    [ "$(grep -c 'error: ' "$WORK/mine/stdout")" = 1 ] || ok=0
    grep -qF -- "$message" "$WORK/mine/stdout" || ok=0
    grep -q 'Compilation failed with 1 error(s)' "$WORK/mine/stderr" || ok=0
    record "$name" $ok "(reference exit $(cat "$WORK/ref/code"), rust exit $(cat "$WORK/mine/code")): $(head -c 300 "$WORK/mine/stdout")"
}

# check_assembles [--ref-fails] <name> <target> <args...>: the Rust -S output must assemble
# without errors or warnings. With --ref-fails, the reference output must fail to assemble
# (bully mode now fixes up what gas rejected).
check_assembles() {
    local ref_fails=0
    if [ "$1" = --ref-fails ]; then ref_fails=1; shift; fi
    local name=$1 target=$2
    shift 2
    [[ "$name" =~ $FILTER ]] || return 0
    local ref gas
    ref=$(ref_for_target "$target")
    gas=$(gas_for_target "$target")
    if [ -z "$ref" ] || [ -z "$gas" ]; then SKIP=$((SKIP + 1)); return 0; fi
    set_args "$target" "$@"
    run "$WORK/ref" "$(ref_gcc_dir "$target")" "$ref" "${REF_ARGS[@]}"
    run "$WORK/mine" "" "$MINE" "${MINE_ARGS[@]}"
    local ok=1 msgs
    [ "$(cat "$WORK/mine/code")" = 0 ] || ok=0
    msgs=$("$gas" -o "$WORK/mine.o" "$WORK/mine/out.S" 2>&1) && [ -z "$msgs" ] || ok=0
    if [ $ref_fails = 1 ] && "$gas" -o "$WORK/ref.o" "$WORK/ref/out.S" 2>/dev/null; then ok=0; fi
    record "$name" $ok "(rust exit $(cat "$WORK/mine/code")): $msgs"
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
        # Inputs the C compiler accepted although gas rejects its output (or the decimal
        # overflowed): an error in noob/obfuscated mode, a bully fix-up in bully mode.
        for input in tests/frontend_cases/fixes/{high_byte_with_*,pointer_*,push_imm*,decimal_overflow}.memeasm; do
            if [ "$mode" = bully ]; then
                flag=--ref-fails
                [[ $input == */decimal_overflow.memeasm ]] && flag=
                for seed in "" -O-1 -d; do
                    # shellcheck disable=SC2086
                    check_assembles $flag "$target/$mode/$input/assembles$seed" "$target" -fcompile-mode bully $seed -S -o OUT "$input"
                done
            else
                case "$input" in
                    *high_byte_with_r8b*) message="invalid parameter combination: 'ah' cannot be combined with 'r8b'" ;;
                    *high_byte_with_spl*) message="invalid parameter combination: 'ah' cannot be combined with 'spl'" ;;
                    *pointer_cmp_9000* | *pointer_upvote* | *pointer_not* | *pointer_shl*) message="invalid parameter combination: operand size unknown" ;;
                    *pointer_8bit_base*) message="invalid parameter combination: a pointer must be a 64 or 32 bit register" ;;
                    *pointer_*) message="invalid parameter combination: this command cannot use a pointer here" ;;
                    *push_imm*) message="invalid parameter combination: pushed decimal numbers must be sign-extendable from 32 Bits" ;;
                    *decimal_overflow*) message='invalid parameter provided: "99999999999999999999" does not fit into 64 Bits' ;;
                esac
                # obfuscated mode replaces every message with a random one
                [ "$mode" = obfuscated ] && message="error: "
                check_new_error "$target/$mode/$input/new-error" "$target" "$message" -fcompile-mode "$mode" -S -o OUT "$input"
            fi
        done
        # Without -S, compile errors (incl. "no main function" for executables) stop before
        # code generation, so these runs compare exactly as well.
        if [ "$mode" != bully ]; then
            for input in "${ERR_INPUTS[@]}" tests/frontend_cases/ok/no_main.memeasm; do
                check --needs-ref-failure "$target/$mode/$input/exe" "$target" -fcompile-mode "$mode" -o OUT "$input"
                check --needs-ref-failure "$target/$mode/$input/obj" "$target" -fcompile-mode "$mode" -O -o OUT "$input"
            done
        fi
    done

    # Command lines that mean the same in both spellings (argument errors and the help page
    # come from clap now and are not compared)
    in=tests/frontend_cases/ok/simple.memeasm
    check "$target/cli/cluster" "$target" -dSo OUT "$in"
    check "$target/cli/attached-o" "$target" -S -oOUT "$in"
    check "$target/cli/permute" "$target" "$in" -S -o OUT -O-3
    check "$target/cli/last-O-wins" "$target" -O-1 -O69420 -S -o OUT "$in"
    check "$target/cli/terminator" "$target" -S -o OUT -- "$in"
    check "$target/cli/-g" "$target" -g -S -o OUT "$in"
    check "$target/cli/-g-bully" "$target" -g -fcompile-mode bully -S -o OUT "$in"
    # memeasm.c's own CLI messages
    check "$target/cli/version" "$target" -v
    check "$target/cli/no-output" "$target" "$in"
    check "$target/cli/no-input" "$target" -o OUT
    check "$target/cli/no-args" "$target"
    check "$target/cli/missing-file" "$target" -o OUT does-not-exist.memeasm
    check "$target/cli/missing-second-file" "$target" -S -o OUT "$in" does-not-exist.memeasm
    check "$target/cli/directory" "$target" -o OUT tests
done

echo "parity: $PASS passed, $FAIL failed, $SKIP skipped"
if [ $FAIL -gt 0 ]; then
    printf '  %s\n' "${FAILED_CASES[@]}" | head -50
    exit 1
fi
