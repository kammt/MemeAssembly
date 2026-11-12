#!/usr/bin/env perl
# Rewrites the (normalised) -S output of the C reference compiler into what the Rust port
# intentionally generates instead. Every rule corresponds to one entry of
# docs/rust-rewrite-changes.md ("Fixed bugs"); nothing else may be rewritten here.
#
# Usage: frontend_parity_whitelist.pl <target> <mode> <opt-level|""> < ref.S > expected.S
#   BULLY_SIZE  operand size (e.g. "QWORD") the Rust output uses for bully-sized pointers
use strict;
use warnings;
no warnings 'portable';

my ($target, $mode, $opt) = @ARGV;
$opt //= '';
local $/;
my $s = <STDIN>;
$s = '' unless defined $s;

# compiler.c:333 "wait, that's illegal": missing comma in `xor r13 r13`.
$s =~ s/^\txor r13 r13$/\txor r13, r13/mg;

# commands.h:68 o_s == o_2: -O-2 also emitted the -O-s alignment and vice versa.
if ($opt eq '-O-2') {
    $s =~ s/^\.align 536870912\n//mg;
} elsif ($opt eq '-O-s') {
    $s =~ s/^\tpush rax\n\tpop rax\n//mg;
}

# translator.c:289 killParent on macOS used the Linux exit syscall number.
if ($target eq 'macos') {
    $s =~ s/^    mov rax, 60$/    mov rax, 0x2000001/mg;
    # translator.c:472 `.align n` means 2**n on Mach-O: -O-s never assembled for macOS.
    $s =~ s/^\.align 536870912$/.p2align 29/mg;
}

# parameters.c:119-122/:207/:354 character parameters gas reads differently: '\v' (gas has no
# \v escape), '\' (the escaped quote) and ASCII codes with leading zeros (octal to gas). The
# real value is written as a decimal number.
$s =~ s/'\\v'/11/g;
$s =~ s/'\\'(?!')/92/g;
$s =~ s{^(\t[a-z]+ [^\n]*, )[-+]?0+(\d+)$}{$1$2}mg;

# compiler.c:172-186 division: `cqo` sign-extended rax before the dividend was loaded, and a
# dividend in rax/rdx was clobbered.
$s =~ s{^\tmov QWORD PTR \[rip \+ \.Ltmp64\], (.+)\n\tpush rdx\n\tcqo\n\tpush rax\n\tmov rax, (.+)\n\tidiv QWORD PTR \[rip \+ \.Ltmp64\]\n\tpush rax\n\tmov rax, \[rsp \+ 8\]\n\tpop \2\n\tadd rsp, 8\n\tpop rdx$}{\tmov QWORD PTR [rip + .Ltmp64], $1\n\tpush $2\n\tpush rdx\n\tpush rax\n\tmov rax, QWORD PTR [rsp + 16]\n\tcqo\n\tidiv QWORD PTR [rip + .Ltmp64]\n\tmov QWORD PTR [rsp + 16], rax\n\tpop rax\n\tpop rdx\n\tpop $2}mg;

# compiler.c:188-207 power: an exponent of 1 or below 0 looped (almost) forever.
$s =~ s{^\tmov QWORD PTR \[rip \+ \.Ltmp64\], (.+)\n\tcmp QWORD PTR \[rip \+ \.Ltmp64\], 0\n\tjne 2f\n\txor (.+), \2\n\tinc \2\n\tjmp 4f\n\t2: push \2\n\tdec QWORD PTR \[rip \+ \.Ltmp64\]\n\t3: imul \2, \[rsp\]\n\tdec QWORD PTR \[rip \+ \.Ltmp64\]\n\tjnz 3b\n\tadd rsp, 8\n\t4:$}{power($1, $2)}mge;

# compiler.c:69 a function whose name gas reads as something else (`$`, a register, `byte`, ...)
# is called by its quoted name.
$s =~ s{^\tcall (\S+)$}{"\tcall " . (misread($1) ? "\"$1\"" : $1)}mge;

# translator.c:355-412 Windows writechar/readchar: the 5th API argument overwrote the saved r9
# and r10/r11 were not preserved.
if ($target eq 'windows') {
    for (['writechar', -11, 'WriteFile'], ['readchar', -10, 'ReadFile']) {
        my ($name, $handle, $api) = @$_;
        my $new = "$name:\n" . join('', map { "\t$_\n" } 'push rcx', 'push rax', 'push rdx', 'push r8', 'push r9',
            'push r10', 'push r11', 'sub rsp, 48', "mov rcx, $handle", 'call GetStdHandle', 'mov rcx, rax',
            'lea rdx, [rip + .LCharacter]', 'mov r8, 1', 'lea r9, [rip + .Ltmp64]', 'mov QWORD PTR [rsp + 32], 0',
            "call $api", 'add rsp, 48', 'pop r11', 'pop r10', 'pop r9', 'pop r8', 'pop rdx', 'pop rax', 'pop rcx', 'ret');
        $s =~ s/^$name:\n.*?^\tret\n/$new/ms;
    }
}

if ($mode eq 'bully') {
    # translator.c:173 `index + 1 % 2`: a pointer second operand next to an immediate got no
    # operand size (which gas rejects). The same holds for the one-operand commands (not, upvote,
    # downvote, shl, shr, it's over 9000), whose pointer never got a size. The size is the same
    # for all bully-sized pointers of a compilation (computedIndex no longer changes during
    # translation).
    my $size = $ENV{BULLY_SIZE} // '';
    if ($size ne '') {
        $s =~ s{^(\t\w+ .*(?:0x[0-9A-F]+|'.+?').*|\tnot \[[^\]]*\]|\t(?:add|sub|shl|shr) \[[^\]]*\], 1|\tcmp \[[^\]]*\], 9000)$}{
            my $line = $1;
            $line =~ s/(?<!PTR )\[/$size PTR [/g;
            $line
        }mge;
    }

    # parameters.c:545 the bully "sign-extendable from 32 bits" fix truncated the decimal
    # string at computedIndex % 32 only; numbers that were still too large were kept (and gas
    # rejected them). Digits are now dropped until the number fits.
    $s =~ s{^(\t(?:add|sub|imul|and|or|xor|cmp) r(?:[abcd]x|[sd]i|[sb]p|8|9|1[0-5]), )0x([0-9A-F]+)$}{
        $1 . fit32($2)
    }mge;
}

# compiler.c:342-346 `it's over 9000` with an 8-bit operand: 9000 is truncated to 40 (gas only
# warns), so the comparison was meaningless. 9000 never fits, so the command always halts.
$s =~ s{^\tcmp (?:[abcd]l|[abcd]h|spl|bpl|sil|dil|r\d+b|BYTE PTR \[[^\]]*\]), 9000\n\tjg 1f\n\thlt\n\t1:$}{\thlt}mg;

print $s;

sub power {
    my ($y, $x) = @_;
    return join("\n", map { "\t$_" } "mov QWORD PTR [rip + .Ltmp64], $y", "push $x", "mov $x, 1",
        'cmp QWORD PTR [rip + .Ltmp64], 0', 'jge 2f', 'and QWORD PTR [rip + .Ltmp64], 1',
        'cmp QWORD PTR [rsp], 1', 'je 2f', 'cmp QWORD PTR [rsp], -1', 'je 2f', "xor $x, $x", 'jmp 3f',
        '2: test QWORD PTR [rip + .Ltmp64], 1', 'jz 1f', "imul $x, QWORD PTR [rsp]",
        '1: shr QWORD PTR [rip + .Ltmp64], 1', 'jz 3f', 'push rax', 'mov rax, QWORD PTR [rsp + 8]',
        'imul rax, rax', 'mov QWORD PTR [rsp + 8], rax', 'pop rax', 'jmp 2b', '3: add rsp, 8');
}

# Mirrors gas_reads_as_symbol in src/frontend/asm_text.rs (negated).
sub misread {
    my $l = lc shift;
    return 1 if $l eq '$' || $l eq '.';
    return 1 if $l =~ /^(?:[re]?(?:ax|cx|dx|bx|sp|bp|si|di)|[abcd]l|[abcd]h|spl|bpl|sil|dil)$/;
    return 1 if $l =~ /^r(8|9|[12]\d|3[01])[bwd]?$/;
    return 1 if $l =~ /^(?:[xyz]mm(\d|[12]\d|3[01])|tmm[0-7]|mm[0-7]|k[0-7]|cr(\d|1[0-5])|dr(\d|1[0-5]))$/;
    return 1 if $l =~ /^(?:st|es|cs|ss|ds|fs|gs|rip|eip|flat|and|or|xor|mod|shl|shr|eq|ne|lt|le|gt|ge|not|byte|word|dword|fword|qword|mmword|tbyte|oword|xmmword|ymmword|zmmword|offset|short|near|far)$/;
    return 0;
}

sub fit32 {
    my ($hex) = @_;
    my $v = unpack('q', pack('Q', hex($hex)));
    return "0x$hex" if $v >= -2147483648 && $v <= 2147483647;
    my $digits = "$v";
    while (1) {
        my $n = ($digits eq '' || $digits eq '-') ? 0 : $digits + 0;
        last if $n >= -2147483648 && $n <= 2147483647;
        chop $digits;
    }
    my $n = ($digits eq '' || $digits eq '-') ? 0 : $digits + 0;
    return sprintf('0x%X', unpack('Q', pack('q', $n)));
}
