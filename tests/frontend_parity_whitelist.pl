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
}

if ($mode eq 'bully') {
    # translator.c:173 `index + 1 % 2`: a pointer second operand next to an immediate got no
    # operand size (which gas rejects). The size is the same for all bully-sized pointers of a
    # compilation (computedIndex no longer changes during translation).
    my $size = $ENV{BULLY_SIZE} // '';
    if ($size ne '') {
        $s =~ s{^(\t\w+ .*(?:0x[0-9A-F]+|'.+?').*)$}{
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

print $s;

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
