#!/usr/bin/env python3
"""Expected instruction encodings, generated with GNU as.

Run by the test `backend::emitter::x86_tests::encodings_match_gas`, which writes the input:
one Intel-syntax instruction per line. By hand:

    python3 tests/emitter_golden.py cases.txt > golden.txt

Output: `text<TAB>hex`, with `<TAB>reloc offset<TAB>addend` appended for an instruction with an
R_X86_64_PC32 relocation, or `text<TAB>error` for a line gas rejects.
Needs x86_64-linux-gnu-as and x86_64-linux-gnu-objdump (override with $AS / $OBJDUMP).
"""
import os
import re
import subprocess
import sys
import tempfile

AS = os.environ.get("AS", "x86_64-linux-gnu-as")
OBJDUMP = os.environ.get("OBJDUMP", "x86_64-linux-gnu-objdump")


def assemble(lines, workdir):
    """Assemble `lines` (each preceded by a label c<i>); returns (error line indices, object)."""
    src = os.path.join(workdir, "golden.s")
    obj = os.path.join(workdir, "golden.o")
    with open(src, "w") as f:
        f.write(".intel_syntax noprefix\n.text\n")
        for i, line in lines:
            f.write(f"c{i}: {line}\n")
        f.write("cend:\n")
    p = subprocess.run([AS, src, "-o", obj], capture_output=True, text=True)
    errors = set()
    for m in re.finditer(r":(\d+): Error:", p.stderr):
        errors.add(lines[int(m.group(1)) - 3][0])
    if p.returncode != 0 and not errors:
        sys.exit(p.stderr)
    return errors, obj if p.returncode == 0 else None


def main():
    cases = [l.rstrip("\n") for l in open(sys.argv[1]) if l.strip()]
    with tempfile.TemporaryDirectory() as d:
        pending = list(enumerate(cases))
        errors, obj = assemble(pending, d)
        if errors:
            errors2, obj = assemble([c for c in pending if c[0] not in errors], d)
            assert not errors2 and obj
        # label offsets
        syms = {}
        for line in subprocess.run([OBJDUMP, "-t", obj], capture_output=True, text=True, check=True).stdout.splitlines():
            f = line.split()
            if len(f) >= 2 and re.fullmatch(r"c\d+|cend", f[-1]) and ".text" in f:
                syms[f[-1]] = int(f[0], 16)
        # .text bytes
        text = bytearray()
        dump = subprocess.run([OBJDUMP, "-s", "-j", ".text", obj], capture_output=True, text=True, check=True).stdout
        for line in dump.splitlines():
            m = re.match(r"^ ([0-9a-f]+) ((?:[0-9a-f]{2,8} ){1,4})", line)
            if m:
                text += bytes.fromhex(m.group(2).replace(" ", ""))
        # relocations: offset -> addend
        relocs = {}
        for line in subprocess.run([OBJDUMP, "-r", "-j", ".text", obj], capture_output=True, text=True, check=True).stdout.splitlines():
            m = re.match(r"^([0-9a-f]+) R_X86_64_PC32\s+sym([+-]0x[0-9a-f]+)?$", line)
            if m:
                relocs[int(m.group(1), 16)] = int(m.group(2) or "0", 16)
            elif re.match(r"^[0-9a-f]+ R_", line):
                sys.exit(f"unexpected relocation: {line}")
        ok = sorted((syms[f"c{i}"], i) for i in range(len(cases)) if i not in errors)
        ends = [o for o, _ in ok[1:]] + [syms["cend"]]
        out = {}
        for (start, i), end in zip(ok, ends):
            line = f"{cases[i]}\t{text[start:end].hex()}"
            r = [(o - start, a) for o, a in relocs.items() if start <= o < end]
            if r:
                line += f"\t{r[0][0]}\t{r[0][1]}"
            out[i] = line
        for i in errors:
            out[i] = f"{cases[i]}\terror"
        for i in range(len(cases)):
            print(out[i])


if __name__ == "__main__":
    main()
