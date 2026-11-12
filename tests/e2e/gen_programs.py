#!/usr/bin/env python3
"""Generates the e2e test corpus under tests/e2e/programs/ and programs/manifest.json.

run.py runs this at the start of every run (by hand: `python3 -I tests/e2e/gen_programs.py`); the
generated files are not committed. Every program is described once here; the
manifest tells run.py which checks to run for it (see run.py for the meaning of the fields).

Conventions for runtime programs: values are printed with the `printhex` (16 hex digits) or
`printnum` (signed decimal) library functions that are appended to the file. The library reserves
the comparison labels r8..r15 and "9" ("r8 wins", ..., "9 wins") and monke labels starting with
"auua"/"uauu", so test bodies must not use those.
"""
import json
import os
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "programs"

FUN = "I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun "
WIN = "I see this as an absolute win"
RET = "right back at ya, buckaroo"


def func(name, body, ret=WIN):
    lines = [FUN + name]
    lines += ["    " + l if l else "" for l in body]
    if ret:
        lines.append("    " + ret)
    return "\n".join(lines) + "\n"


def call(name):
    return f"{name}: whomst has summoned the almighty one"


LIB_HEX = func("printhex", [
    "What the hell happened here? prints rax as 16 hex digits, preserves all registers",
    "stonks rax", "stonks rbx", "stonks rcx", "stonks r8", "stonks r9", "stonks r10", "stonks r11",
    "sneak 100 r10",
    "r8 is brilliant, but I like 16",
    "r11 is brilliant, but I like 16",
    "rcx is brilliant, but I like rax",
    "monke auuaauauaa",
    "rbx is brilliant, but I like rcx",
    "bitconneeeeeeect rbx 15",
    "stonks rbx",
    "they had us in the first half, not gonna lie rcx",
    "they had us in the first half, not gonna lie rcx",
    "they had us in the first half, not gonna lie rcx",
    "they had us in the first half, not gonna lie rcx",
    "downvote r8",
    "who would win? r8 or r10",
    "monke auuaauauau",
    "not stonks rbx",
    "r9 is brilliant, but I like rbx",
    "who would win? r9 or 9",
    "9 wins",
    "rbx units are ready, with 48 more well on the way",
    "return to monke auuaauaaua",
    "r9 wins",
    "rbx units are ready, with 87 more well on the way",
    "monke auuaauaaua",
    "what can I say except bl",
    "downvote r11",
    "who would win? r11 or r10",
    "not stonks r11", "not stonks r10", "not stonks r9", "not stonks r8",
    "not stonks rcx", "not stonks rbx", "not stonks rax",
    RET,
    "r8 wins",
    "return to monke auuaauauaa",
    "r11 wins",
    "return to monke auuaauauau",
    "r10 wins",
], ret=RET)

LIB_NUM = func("printnum", [
    "What the hell happened here? prints rax as signed decimal, preserves all registers",
    "stonks rax", "stonks rbx", "stonks rdx", "stonks r12", "stonks r13", "stonks r14", "stonks r15",
    "sneak 100 r14",
    "sneak 100 r12",
    "r15 is brilliant, but I like rax",
    "who would win? r15 or r14",
    "return to monke uauuuauuaa",
    "r15 wins",
    "return to monke uauuuauuaa",
    "r14 wins",
    "what can I say except -",
    "rax \\s",
    "upvote rax",
    "monke uauuuauuaa",
    "rbx is brilliant, but I like rax",
    "look at what 10 needs to mimic a fraction of rax",
    "rdx is brilliant, but I like rax",
    "rdx is getting out of hand, now there are 10 of them",
    "parry rdx you filthy casual rbx",
    "rbx units are ready, with 48 more well on the way",
    "stonks rbx",
    "upvote r12",
    "r13 is brilliant, but I like rax",
    "who would win? r13 or r14",
    "monke uauuuauuau",
    "not stonks rbx",
    "what can I say except bl",
    "downvote r12",
    "who would win? r12 or r14",
    "not stonks r15", "not stonks r14", "not stonks r13", "not stonks r12",
    "not stonks rdx", "not stonks rbx", "not stonks rax",
    RET,
    "r13 wins",
    "return to monke uauuuauuaa",
    "r12 wins",
    "return to monke uauuuauuau",
], ret=RET)


def show(reg="rax", lib="printhex"):
    """Print a 64-bit register (hex by default) followed by a newline; preserves rax."""
    out = ["stonks rax"]
    if reg != "rax":
        out.append(f"rax is brilliant, but I like {reg}")
    out += [call(lib), "what can I say except \\n", "not stonks rax"]
    return out


def shownum(reg="rax"):
    return show(reg, "printnum")


def say(text):
    m = {" ": "space", "\n": "\\n", "\t": "\\t"}
    return [f"what can I say except {m.get(c, c)}" for c in text]


def runprog(body, hexlib=True, numlib=False, extra=""):
    t = func("main", body)
    t += extra
    if hexlib:
        t += "\n" + LIB_HEX
    if numlib:
        t += "\n" + LIB_NUM
    return t


PROGRAMS = {}   # relative path -> text (bytes or str)
MANIFEST = []   # entries


ALL_KINDS = ["lin-exe", "lin-asm", "lin-obj", "win-asm", "win-obj", "win-exe"]


def add(name, text, category, kinds=None, files=None, **meta):
    """Register a single-file program programs/<name>.memeasm (or a multi-file program)."""
    entry = {"name": name, "category": category, "kinds": kinds or ALL_KINDS}
    if files is None:
        path = f"{name}.memeasm"
        PROGRAMS[path] = text
        entry["files"] = [f"tests/e2e/programs/{path}"]
    else:
        for fname, ftext in files.items():
            PROGRAMS[f"{name}/{fname}"] = ftext
        entry["files"] = [f"tests/e2e/programs/{name}/{f}" for f in files]
    if "stdin" in meta:
        sp = f"{name}.stdin"
        PROGRAMS[sp] = meta.pop("stdin")
        entry["stdin"] = f"tests/e2e/programs/{sp}"
    entry.update(meta)
    MANIFEST.append(entry)


# --------------------------------------------------------------------------------------------
# cmd/: one program per command and parameter type (runtime-checked)
# --------------------------------------------------------------------------------------------
R64 = ["rax", "rbx", "rcx", "rdx", "rdi", "rsi", "rbp", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"]
R32 = ["eax", "ebx", "ecx", "edx", "edi", "esi", "ebp", "r8d", "r9d", "r10d", "r11d", "r12d", "r13d", "r14d", "r15d"]
R16 = ["ax", "bx", "cx", "dx", "di", "si", "bp", "r8w", "r9w", "r10w", "r11w", "r12w", "r13w", "r14w", "r15w"]
R8L = ["al", "bl", "cl", "dl", "dil", "sil", "bpl", "r8b", "r9b", "r10b", "r11b", "r12b", "r13b", "r14b", "r15b"]
R8H = ["ah", "bh", "ch", "dh"]
PARENT = {}
for i, r in enumerate(R64):
    PARENT[r] = r
    PARENT[R32[i]] = r
    PARENT[R16[i]] = r
    PARENT[R8L[i]] = r
for h, p in zip(R8H, ["rax", "rbx", "rcx", "rdx"]):
    PARENT[h] = p

BIG = "-6148914691236517206"   # 0xAAAAAAAAAAAAAAAA


def fill(regs, value=BIG):
    return [f"{PARENT[r]} is brilliant, but I like {value}" for r in regs]


def cmd(name, body, **kw):
    add(f"cmd/{name}", runprog(body, numlib=kw.pop("numlib", False)), "cmd", **kw)


# mov ---------------------------------------------------------------------------------------
b = []
vals = ["0", "1", "-1", "9223372036854775807", "-9223372036854775808", "4294967296", "305419896",
        "-2", "81985529216486895", "1311768467463790320", "-81985529216486896", "2147483648",
        "-2147483649", "65535"]
for r, v in zip(R64, vals):
    b += [f"{r} is brilliant, but I like {v}"] + show(r)
cmd("mov_r64_decimal", b)

b = []
for r in R32:
    b += fill([r]) + [f"{r} is brilliant, but I like 4294967295"] + show(PARENT[r])
    b += [f"{r} is brilliant, but I like -2147483648"] + show(PARENT[r])
cmd("mov_r32_decimal", b)

b = []
for r in R16:
    b += fill([r]) + [f"{r} is brilliant, but I like 4660"] + show(PARENT[r])
    b += [f"{r} is brilliant, but I like -32768"] + show(PARENT[r])
cmd("mov_r16_decimal", b)

b = []
for r in R8L + R8H:
    b += fill([r]) + [f"{r} is brilliant, but I like 255"] + show(PARENT[r])
    b += [f"{r} is brilliant, but I like -128"] + show(PARENT[r])
cmd("mov_r8_decimal", b)

b = []
chars = ["a", "Z", "0", "~", "#", ";", "'", ",", "\\n", "\\s", "space", "\\t", "\\f", "\\b", "\\\"",
         "\\?", "\\\\", "65", "255", "128", "7"]
for c in chars:
    b += ["rax is brilliant, but I like -1", f"al is brilliant, but I like {c}"] + show("rax")
for c in ["a", "\\n", "200", "q"]:
    b += ["rdx is brilliant, but I like -1", f"rbx is brilliant, but I like {c}", f"ecx is brilliant, but I like {c}",
          f"dx is brilliant, but I like {c}"] + show("rbx") + show("rcx") + show("rdx")
cmd("mov_char", b)

b = []
pairs = [("rax", "rbx"), ("r15", "rax"), ("eax", "r9d"), ("r10d", "esi"), ("ax", "r12w"), ("di", "bx"),
         ("al", "r13b"), ("ah", "bl"), ("bh", "ch"), ("r8b", "sil"), ("dil", "bpl"), ("dh", "al")]
for d, s in pairs:
    b += fill([d]) + [f"{PARENT[s]} is brilliant, but I like 1311768467463790320",
                      f"{d} is brilliant, but I like {s}"] + show(PARENT[d])
cmd("mov_reg_reg", b)

b = ["rbx is brilliant, but I like 1311768467463790320", "stonks rbx",
     "rax is brilliant, but I like rsp do you know de wey"] + show("rax") + [
     "sneak 100 rcx", "ecx is brilliant, but I like rsp do you know de wey"] + show("rcx") + [
     "rdx is brilliant, but I like -1", "dx is brilliant, but I like rsp do you know de wey"] + show("rdx") + [
     "rdx is brilliant, but I like -1", "dh is brilliant, but I like rsp do you know de wey"] + show("rdx") + [
     "r9 is brilliant, but I like -1", "r9b is brilliant, but I like rsp do you know de wey"] + show("r9") + [
     "rsi is brilliant, but I like 81985529216486895",
     "rsp do you know de wey is brilliant, but I like rsi", "not stonks rax"] + show("rax") + [
     "stonks rbx", "rsp do you know de wey is brilliant, but I like si", "not stonks rax"] + show("rax") + [
     "stonks rbx", "rsp do you know de wey is brilliant, but I like al", "not stonks rax"] + show("rax") + [
     "stonks rbx", "r10 is brilliant, but I like 4886718345",
     "rsp do you know de wey is brilliant, but I like r10d", "not stonks rax"] + show("rax")
cmd("mov_pointer", b)

# stack --------------------------------------------------------------------------------------
b = ["stonks 5", "stonks -5", "stonks 2147483647", "stonks -2147483648", "stonks a", "stonks \\n",
     "stonks 200", "stonks 0", "stonks space"]
for _ in range(9):
    b += ["not stonks rax"] + show("rax")
b += ["r13 is brilliant, but I like -81985529216486896", "stonks r13", "not stonks r14"] + show("r14")
b += ["rbx is brilliant, but I like 4919", "stonks rbx", "stonks rsp do you know de wey",
      "not stonks rcx", "not stonks rdx"] + show("rcx") + show("rdx")
b += ["stonks 7", "stonks 8", "not stonks rsp do you know de wey", "not stonks rax"] + show("rax")
cmd("stack", b)

# and ----------------------------------------------------------------------------------------
b = []
for d, s in [("rax", "-252645136"), ("eax", "4042322160"), ("ax", "3855"), ("al", "60"),
             ("ah", "15"), ("r9b", "-16"), ("r12", "-256"), ("rbx", "a"), ("ecx", "\\n"),
             ("r14d", "r15d"), ("rdi", "rsi"), ("si", "dx"), ("bh", "dl")]:
    b += fill([d]) + ["r15 is brilliant, but I like 1085102592571150095",
                      "rsi is brilliant, but I like 1311768467463790320",
                      "rdx is brilliant, but I like 47802"]
    b += [f"bitconneeeeeeect {d} {s}"] + show(PARENT[d])
b += ["rax is brilliant, but I like 255", "stonks -1", "bitconneeeeeeect rsp do you know de wey rax",
      "not stonks rbx"] + show("rbx")
b += ["stonks -1", "rcx is brilliant, but I like 0", "bitconneeeeeeect cl rsp do you know de wey",
      "not stonks rbx"] + show("rcx")
cmd("and", b)

# not ----------------------------------------------------------------------------------------
b = []
for r in ["rax", "eax", "ax", "al", "ah", "r9b", "r10w", "r11d", "rdi", "bh", "sil"]:
    b += ["rax is brilliant, but I like 0"] + fill([r], "1311768467463790320") + [f"{r} \\s"] + show(PARENT[r])
cmd("not", b)

# xor (sneak 100) ------------------------------------------------------------------------------
b = []
for r in ["rax", "ebx", "cx", "dl", "dh", "r8", "r9d", "r10w", "r11b", "r15b", "bpl"]:
    b += fill([r]) + [f"sneak 100 {r}"] + show(PARENT[r])
cmd("xor_zero", b)

# add/sub/inc/dec ------------------------------------------------------------------------------
b = []
for r in ["rax", "eax", "ax", "al", "ah", "r10b", "r13w", "r14d", "r15"]:
    b += [f"{PARENT[r]} is brilliant, but I like -1", f"upvote {r}"] + show(PARENT[r])
    b += [f"{PARENT[r]} is brilliant, but I like 0", f"downvote {r}"] + show(PARENT[r])
    b += [f"{PARENT[r]} is brilliant, but I like 1311768467463790320", f"upvote {r}", f"upvote {r}",
          f"downvote {r}"] + show(PARENT[r])
cmd("inc_dec", b)

b = []
for d, s in [("rax", "1"), ("rax", "-5"), ("rax", "2147483647"), ("rax", "-2147483648"), ("eax", "4294967295"),
             ("ax", "65535"), ("al", "255"), ("ah", "-128"), ("r8", "r9"), ("r8d", "r9d"), ("r8w", "r9w"),
             ("r8b", "r9b"), ("rbx", "a"), ("bl", "\\n"), ("cx", "space"), ("rsi", "rsi")]:
    b += [f"{PARENT[d]} is brilliant, but I like 1311768467463790320",
          "r9 is brilliant, but I like 81985529216486895",
          f"{d} units are ready, with {s} more well on the way"] + show(PARENT[d])
cmd("add", b)

b = []
for s, d in [("1", "rax"), ("-5", "rax"), ("2147483647", "rax"), ("-2147483648", "rcx"),
             ("4294967295", "eax"), ("65535", "ax"), ("255", "al"), ("100", "ah"), ("r9", "r8"),
             ("r9d", "r8d"), ("r9w", "r8w"), ("r9b", "r8b"), ("a", "rbx"), ("\\t", "bl"),
             ("rdi", "rdi"), ("97", "dil")]:
    b += [f"{PARENT[d]} is brilliant, but I like 81985529216486895",
          "r9 is brilliant, but I like 1311768467463790320",
          f"parry {s} you filthy casual {d}"] + show(PARENT[d])
b += ["stonks 1000", "rax is brilliant, but I like 1", "parry rax you filthy casual rsp do you know de wey",
      "not stonks rax"] + show("rax")
cmd("sub", b)

# shifts ---------------------------------------------------------------------------------------
b = []
for r in ["rax", "eax", "ax", "al", "ah", "r11", "r12d", "r13w", "r14b", "sil"]:
    b += fill([r], "-6148914691236517207") + [f"upgrades, people. Upgrades {r}"] + show(PARENT[r])
    b += fill([r], "-6148914691236517207") + [f"they had us in the first half, not gonna lie {r}"] + show(PARENT[r])
cmd("shifts", b)

# imul -----------------------------------------------------------------------------------------
b = []
for d, s in [("rax", "3"), ("rax", "-7"), ("rax", "2147483647"), ("eax", "100000"), ("eax", "-1"),
             ("rbx", "rcx"), ("ebx", "ecx"), ("r12", "r13"), ("r12d", "r13d"), ("rdx", "a"),
             ("edx", "\\n"), ("rsi", "255")]:
    b += [f"{PARENT[d]} is brilliant, but I like 1311768467463790320",
          "rcx is brilliant, but I like -3", "r13 is brilliant, but I like 1000000007",
          f"{d} is getting out of hand, now there are {s} of them"] + show(PARENT[d])
b += ["stonks 6", "rax is brilliant, but I like 7",
      "rax is getting out of hand, now there are rsp do you know de wey of them", "not stonks rbx"] + show("rax")
cmd("imul", b)

# idiv -----------------------------------------------------------------------------------------
b = []
for dvs, dvd, a, v in [("7", "rax", "0", "100"), ("-7", "rax", "0", "100"), ("7", "rax", "0", "-100"),
                       ("rcx", "rbx", "0", "1000000"), ("a", "rbx", "0", "9700"), ("\\n", "rdx", "0", "12345"),
                       ("7", "rbx", "1", "100"), ("3", "r15", "-5", "-9"), ("rbx", "rbx", "0", "77"),
                       ("1", "rax", "0", "-9223372036854775807"), ("rax", "rcx", "50", "1000")]:
    b += [f"rax is brilliant, but I like {a}", "rcx is brilliant, but I like 33",
          f"{PARENT[dvd]} is brilliant, but I like {v}" if dvd != "rax" else f"rax is brilliant, but I like {v}",
          "rdx is brilliant, but I like 12345" if dvd == "rdx" else "rdx is brilliant, but I like 5",
          f"look at what {dvs} needs to mimic a fraction of {dvd}"] + show(dvd) + show("rdx")
cmd("idiv", b, ref_bug={
    "desc": "division with the dividend in rdx (docs/rust-rewrite-changes.md #12): the reference's final 'pop rdx' "
            "restores the old rdx, so 'look at what \\n needs to mimic a fraction of rdx' leaves rdx = 12345 "
            "instead of 1234",
    "bug_kinds": ["lin-exe", "win-run"],
    "correct": {"kind": "runs", "oracle": "tests/e2e/programs/cmd/idiv.oracle.memeasm"}})
# oracle: the rdx case divides rbx instead and copies the quotient into rdx (correct in the reference, too)
PROGRAMS["cmd/idiv.oracle.memeasm"] = runprog(sum([
    ["rbx is brilliant, but I like 12345", "look at what \\n needs to mimic a fraction of rbx",
     "rdx is brilliant, but I like rbx"] if l == "look at what \\n needs to mimic a fraction of rdx" else
    [] if l == "rdx is brilliant, but I like 12345" else [l] for l in b], []))

# power ----------------------------------------------------------------------------------------
b = []
for base, exp in [("2", "10"), ("3", "5"), ("-2", "3"), ("7", "0"), ("2", "63"), ("2", "64"), 
                  ("2", "a"), ("3", "\\t"), ("10", "rcx"), ("0", "0")]:
    b += [f"rbx is brilliant, but I like {base}", "rcx is brilliant, but I like 18",
          f"rbx UNLIMITED POWER {exp}"] + show("rbx") + show("rcx")
b += ["r9 is brilliant, but I like 3", "r9 UNLIMITED POWER r9"] + show("r9")
cmd("power", b)

# jumps ----------------------------------------------------------------------------------------
b = ["rcx is brilliant, but I like 0", "upgrade", "upvote rcx"] + show("rcx") + [
     "corporate needs you to find the difference between rcx and 5", "fuck go back",
     "they're the same picture"] + say("done\n")
cmd("upgrade_loop", b)

b = ["rcx is brilliant, but I like 3", "banana"] + say("b") + ["downvote rcx",
     "corporate needs you to find the difference between rcx and 0", "where banana",
     "they're the same picture"] + say("\n")
cmd("banana_loop", b)

b = ["return to monke uuaauaaau"] + say("skipped") + ["monke uuaauaaau"] + say("monke\n") + [
     "rdx is brilliant, but I like 2", "monke ua", "downvote rdx"] + say("x") + [
     "corporate needs you to find the difference between rdx and 0", "return to monke ua",
     "they're the same picture"] + say("\n")
cmd("monke", b)

b = []
for i, (x, y, xv, yv) in enumerate([("rax", "rbx", "5", "3"), ("rcx", "rdx", "3", "5"), ("rsi", "rdi", "4", "4")]):
    b += [f"{x} is brilliant, but I like {xv}", f"{y} is brilliant, but I like {yv}", f"who would win? {x} or {y}"]
    b += say("=") + [f"return to monke uaaaau{'a' * i}u", f"{x} wins"] + say(">") + [
        f"return to monke uaaaau{'a' * i}u", f"{y} wins"] + say("<") + [f"monke uaaaau{'a' * i}u"]
b += say("\n")
# comparisons with decimal / char / 8-bit / 32-bit / 16-bit operands
for i, (x, y, xv) in enumerate([("eax", "-5", "-6"), ("ax", "1000", "1001"), ("bl", "a", "97"),
                                 ("ecx", "\\n", "10"), ("dx", "255", "-1"), ("sil", "7", "-1")]):
    b += [f"{PARENT[x]} is brilliant, but I like {xv}", f"who would win? {x} or {y}"] + say("=") + [
        f"return to monke uaaaaau{'a' * i}u", f"{x} wins"] + say(">") + [
        f"return to monke uaaaaau{'a' * i}u", f"{y} wins"] + say("<") + [f"monke uaaaaau{'a' * i}u"]
b += say("\n")
cmd("who_would_win", b)

b = ["rax is brilliant, but I like 7", "corporate needs you to find the difference between rax and 8"] + say("ne ") + [
     "corporate needs you to find the difference between al and 7"] + say("unreachable") + [
     "they're the same picture"] + say("eq\n")
cmd("corporate", b)

b = ["sneak 100 rax", "bitconneeeeeeect rax 0", "corporate needs you to find the difference between ebx and ecx",
     "corporate needs you to find the difference between r8w and 5", "corporate needs you to find the difference between bh and a",
     "corporate needs you to find the difference between rdi and \\n"] + say("no\n") + ["they're the same picture"]
cmd("corporate_variants", b, kinds=["lin-asm", "lin-obj", "win-asm", "win-obj"])

# IO -------------------------------------------------------------------------------------------
b = []
for c in chars + ["!", "\\s", "0", "10", "126"]:
    b += [f"what can I say except {c}"]
b += say("\n")
b += ["rax is brilliant, but I like 18519", "rbx is brilliant, but I like 17220", "rcx is brilliant, but I like 17734",
      "rdx is brilliant, but I like 18248", "rsi is brilliant, but I like 74", "rdi is brilliant, but I like 75",
      "rbp is brilliant, but I like 76"]
for r in ["al", "ah", "bl", "bh", "cl", "ch", "dl", "dh", "sil", "dil", "bpl"]:
    b += [f"what can I say except {r}"]
for i, r in enumerate(["r8b", "r9b", "r10b", "r11b", "r12b", "r13b", "r14b", "r15b"]):
    b += [f"{PARENT[r]} is brilliant, but I like {77 + i}", f"what can I say except {r}"]
b += say("\n")
cmd("print", b)

b = []
for r in ["al", "ah", "bl", "bh", "r8b", "sil", "dil", "r15b"]:
    b += [f"{PARENT[r]} is brilliant, but I like -1", f"let me in. LET ME IIIIIIIIN {r}"] + show(PARENT[r])
b += ["What the hell happened here? stdin is exhausted now: the old character stays"]
b += ["rax is brilliant, but I like 0", "let me in. LET ME IIIIIIIIN al"] + show("rax")
b += ["what can I say except Q", "let me in. LET ME IIIIIIIIN al"] + show("rax")
cmd("read", b, stdin=b"AbC\xffxyz\n")

# random/misc commands --------------------------------------------------------------------------
cmd("nop", ["oh no! anyway"] + say("nop\n") + ["oh no! anyway"])
cmd("over9000_ok", ["rax is brilliant, but I like 9001", "it's over 9000 rax", "ax is brilliant, but I like 9001",
                    "it's over 9000 ax", "ebx is brilliant, but I like 100000", "it's over 9000 ebx",
                    "r12w is brilliant, but I like 32767", "it's over 9000 r12w",
                    "What the hell happened here? 8-bit values are never over 9000 (always halt, see crash/over9000_hlt8)"]
    + say("ok\n"))
cmd("rdrand", ["rax is brilliant, but I like -1", "it's dangerous to go alone, take eax"] + [
    "they had us in the first half, not gonna lie rax"] * 32 + show("rax") + [
    "rbx is brilliant, but I like -1", "it's dangerous to go alone, take bx"] + [
    "they had us in the first half, not gonna lie rbx"] * 16 + show("rbx") + [
    "it's dangerous to go alone, take r12", "it's dangerous to go alone, take r12d",
    "it's dangerous to go alone, take r12w"] + say("ok\n"),
    win_run_unsupported="box64 does not implement 'rdrand r16' (66 0F C7 /6): SIGILL at 'rdrand bx', also in a "
                        "mingw-gcc build of the same -S text")
cmd("syscall_exit", say("bye\n") + ["rax is brilliant, but I like 60", "rdi is brilliant, but I like 42",
                                    "we need air support"] + say("unreachable"))
cmd("syscall_write", ["stonks 2660", "rax is brilliant, but I like 1", "rdi is brilliant, but I like 1",
                      "rsi is brilliant, but I like rsp", "rdx is brilliant, but I like 2", "we need air support",
                      "not stonks rbx"] + show("rax"))
cmd("or_draw_25", ["rax is brilliant, but I like -1", "upvote rax or draw 25"] + show("rax") + [
    "rax is brilliant, but I like 100", "what can I say except a or draw 25"] + show("rax") + [
    "guess I'll die or draw 25", "stonks rax or draw 25", "it's a trap or draw 25",
    "Houston, we have a problem or draw 25"] + show("rax"))
cmd("return_values", ["What the hell happened here? exit code of main = al",
                      "rax is brilliant, but I like 300", RET])
cmd("return_no", say("no\n") + ["no, I don't think I will"])
add("cmd/functions", runprog(
    [call("f1")] + show("rax") + [call("f2")] + show("rax") + [call("f3")] + show("rax") +
    ["rbx is brilliant, but I like 10", call("fact")] + shownum("rax") + [call("empty")] + say("end\n"),
    numlib=True,
    extra="\n" + func("f1", ["rax is brilliant, but I like 77"], ret=RET) + "\n" +
          func("f2", ["rax is brilliant, but I like 77"], ret="no, I don't think I will") + "\n" +
          func("f3", ["rax is brilliant, but I like 77"], ret=WIN) + "\n" +
          func("empty", [], ret=RET) + "\n" +
          func("fact", ["What the hell happened here? rax = rbx! (recursive)",
                        "corporate needs you to find the difference between rbx and 1",
                        "stonks rbx", "downvote rbx", call("fact"), "not stonks rbx",
                        "rax is getting out of hand, now there are rbx of them", RET,
                        "they're the same picture", "rax is brilliant, but I like 1"], ret=RET)),
    "cmd")

# runtime crashes / non-termination -------------------------------------------------------------
for name, body in [
    ("die", say("before\n") + ["guess I'll die"] + say("after\n")),
    ("houston", say("before\n") + ["Houston, we have a problem"] + say("after\n")),
    ("over9000_hlt", say("before\n") + ["rax is brilliant, but I like 9000", "it's over 9000 rax"] + say("after\n")),
    ("over9000_hlt8", say("before\n") + ["al is brilliant, but I like 40", "it's over 9000 al"] + say("after\n")),
    ("trap", say("before\n") + ["it's a trap"] + say("after\n")),
    ("shall_not_pass", say("before\n") + ["you shall not pass!"] + say("after\n")),
    ("refuses", say("before\n") + ["refuses to elaborate and leaves"] + say("after\n")),
    ("divzero", say("before\n") + ["rax is brilliant, but I like 5", "look at what 0 needs to mimic a fraction of rax"] + say("after\n")),
    ("divoverflow", say("before\n") + ["rax is brilliant, but I like -9223372036854775808",
                                       "look at what -1 needs to mimic a fraction of rax"] + say("after\n")),
    ("stack_overflow", ["monke uauauau", "stonks rax", "return to monke uauauau"]),
    ("print_forever", ["upgrade"] + say("y") + ["fuck go back"]),
]:
    add(f"crash/{name}", runprog(body, hexlib=False), "runtime", timeout=3)

# --------------------------------------------------------------------------------------------
# calc/: programs computing things
# --------------------------------------------------------------------------------------------
add("calc/factorials", runprog(
    ["rcx is brilliant, but I like 1", "rax is brilliant, but I like 1", "upgrade",
     "rax is getting out of hand, now there are rcx of them"] + shownum("rcx") + say("! = ") + shownum("rax") + [
     "upvote rcx", "corporate needs you to find the difference between rcx and 21", "fuck go back",
     "they're the same picture"], hexlib=False, numlib=True), "calc")
add("calc/fibonacci", runprog(
    ["rax is brilliant, but I like 0", "rbx is brilliant, but I like 1", "rcx is brilliant, but I like 0", "upgrade"]
    + shownum("rax") + ["rdx is brilliant, but I like rax", "rdx units are ready, with rbx more well on the way",
                        "rax is brilliant, but I like rbx", "rbx is brilliant, but I like rdx", "upvote rcx",
                        "corporate needs you to find the difference between rcx and 92", "fuck go back",
                        "they're the same picture"] + shownum("rax"), hexlib=False, numlib=True), "calc")
add("calc/powers", runprog(
    ["What the hell happened here? x UNLIMITED POWER 1 loops ~2^64 times in the reference, so start at 2",
     "rcx is brilliant, but I like 2", "upgrade", "rax is brilliant, but I like 3", "rax UNLIMITED POWER rcx"]
    + shownum("rax") + ["rax is brilliant, but I like -2", "rax UNLIMITED POWER rcx"] + shownum("rax") + [
     "upvote rcx", "corporate needs you to find the difference between rcx and 41", "fuck go back",
     "they're the same picture"], hexlib=False, numlib=True), "calc")
add("calc/divisions", runprog(
    sum([[f"rax is brilliant, but I like {n}", f"look at what {d} needs to mimic a fraction of rax"] + shownum("rax")
         for n, d in [(100, 7), (-100, 7), (100, -7), (-100, -7), (9223372036854775807, 10), (1, 2), (0, 5),
                      (-1, 2), (123456789, 1000), (65, "a")]], []), hexlib=False, numlib=True), "calc")
add("calc/primes", runprog(
    ["What the hell happened here? trial division primes below 200",
     "rbx is brilliant, but I like 2",
     "monke uuuaau",
     "rcx is brilliant, but I like 2",
     "monke uuuaua",
     "corporate needs you to find the difference between rcx and rbx",
     "rax is brilliant, but I like rbx",
     "look at what rcx needs to mimic a fraction of rax",
     "rax is getting out of hand, now there are rcx of them",
     "parry rbx you filthy casual rax",
     "sneak 100 rdi",
     "who would win? rax or rdi",
     "return to monke uuuuau",
     "rax wins",
     "rdi wins",
     "upvote rcx",
     "return to monke uuuaua",
     "they're the same picture"] + shownum("rbx") + [
     "monke uuuuau",
     "upvote rbx",
     "rdx is brilliant, but I like rbx",
     "who would win? rdx or 200",
     "200 wins",
     "return to monke uuuaau",
     "rdx wins"], hexlib=False, numlib=True), "calc")
add("calc/sum_stdin", runprog(
    ["What the hell happened here? sums the decimal numbers on stdin (one per line, terminated by a line '.')",
     "sneak 100 rbx", "sneak 100 rcx",
     "monke uuauauuu",
     "sneak 100 rax",
     "let me in. LET ME IIIIIIIIN al",
     "corporate needs you to find the difference between al and .",
     "who would win? al or \\n",
     "What the hell happened here? newline: add rcx to the total",
     "rbx units are ready, with rcx more well on the way",
     "sneak 100 rcx",
     "return to monke uuauauuu",
     "al wins",
     "\\n wins",
     "parry 48 you filthy casual rax",
     "rcx is getting out of hand, now there are 10 of them",
     "rcx units are ready, with rax more well on the way",
     "return to monke uuauauuu",
     "they're the same picture"] + say("sum=") + shownum("rbx"), hexlib=False, numlib=True), "calc",
    stdin=b"12\n30\n1000\n7\n9999999\n.\n")
add("calc/upper_stdin", runprog(
    ["What the hell happened here? uppercases stdin up to the first newline",
     "monke uaauauau",
     "let me in. LET ME IIIIIIIIN al",
     "corporate needs you to find the difference between al and \\n",
     "bl is brilliant, but I like al",
     "who would win? bl or z",
     "bl wins",
     "return to monke uaauauuu",
     "z wins",
     "who would win? al or a",
     "al wins",
     "parry 32 you filthy casual al",
     "a wins",
     "monke uaauauuu",
     "what can I say except al",
     "return to monke uaauauau",
     "they're the same picture"] + say("\n"), hexlib=False), "calc", stdin=b"Hello, World! abc xyz {|}\nignored\n")
add("calc/bits", runprog(
    ["What the hell happened here? popcount of a few numbers using and/shr"] + sum([[
        f"rax is brilliant, but I like {v}", "sneak 100 rcx", f"rdx is brilliant, but I like 64", f"monke uuaaa{'u' * i}a",
        "rbx is brilliant, but I like rax", "bitconneeeeeeect rbx 1", "rcx units are ready, with rbx more well on the way",
        "they had us in the first half, not gonna lie rax", "downvote rdx",
        "corporate needs you to find the difference between rdx and 0" if i == 0 else "oh no! anyway",
        f"return to monke uuaaa{'u' * i}a" if i else "return to monke uuaaaa"] for i, v in enumerate(["-1"])], []) + [
        "they're the same picture"] + shownum("rcx"), hexlib=False, numlib=True), "calc")

# --------------------------------------------------------------------------------------------
# rand/: confused stonks / perfectly balanced (non-deterministic in the reference)
# --------------------------------------------------------------------------------------------
add("rand/confused_stonks", func("main", say("ab") + ["confused stonks"] + say("cd\n")), "random", timeout=2,
    random=True)
add("rand/confused_stonks_loop", func("main", ["rcx is brilliant, but I like 3", "upgrade"] + say("x") + [
    "downvote rcx", "corporate needs you to find the difference between rcx and 0", "confused stonks",
    "fuck go back", "they're the same picture"] + say("\n")), "random", timeout=2, random=True)
add("rand/perfectly_balanced", func("main", say("ab") + ["perfectly balanced as all things should be"] + say("\n")),
    "random", timeout=2, random=True)
add("rand/perfectly_balanced_twice", func("main", ["perfectly balanced as all things should be",
                                                   "perfectly balanced as all things should be"] + say("xyz\n")),
    "random", timeout=2, random=True)
add("rand/confused_multi", None, "random", files={
    "a.memeasm": func("main", say("A") + [call("other")] + say("\n")),
    "b.memeasm": func("other", say("B") + ["confused stonks"] + say("C"), ret=RET)}, timeout=2, random=True)

# --------------------------------------------------------------------------------------------
# misc/: lexical edge cases, comments, art output
# --------------------------------------------------------------------------------------------
add("misc/crlf", (func("main", say("crlf\n"))).replace("\n", "\r\n"), "misc")
add("misc/no_trailing_newline", func("main", say("x\n")).rstrip("\n"), "misc")
add("misc/tabs_and_comments", "What the hell happened here? leading comment\n\n\t\n" +
    "I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun main   \t\n"
    "\twhat can I say except t\t \n"
    "\t  What the hell happened here? indented comment\n"
    "  \t what can I say except \\n\n"
    "\n\n    I see this as an absolute win\t\t\n"
    "What the hell happened here? trailing comment\n", "misc")
add("misc/nice", runprog(["rax is brilliant, but I like 69", "rbx is brilliant, but I like 420",
                          "rcx units are ready, with 69 more well on the way", "stonks 420", "not stonks rdx"]
                         + show("rax") + show("rbx")), "misc")
add("misc/main_not_first", func("helper", say("h"), ret=RET) + "\n" + func("main", [call("helper")] + say("m\n")), "misc")
add("misc/only_main_def", FUN + "main\n    " + WIN + "\n", "misc")
add("misc/many_functions", "".join(func(f"f{i}", say(chr(97 + i)) + ([call(f"f{i + 1}")] if i < 20 else []),
                                        ret=RET) + "\n" for i in range(21)) + func("main", [call("f0")] + say("\n")),
    "misc")
add("misc/escape_labels", func("main", ["al is brilliant, but I like \\\"", "who would win? al or \\\"",
                                        "al wins", "\\\" wins", "bl is brilliant, but I like 5", "who would win? bl or 5",
                                        "bl wins", "5 wins", "who would win? cl or ;", "cl wins", "; wins"] + say("ok\n")),
    "misc")
add("misc/weird_function_names", func("main", [call("a.b$c_9"), call("_"), call("x$")] + say("\n")) + "\n" +
    func("a.b$c_9", say("1"), ret=RET) + "\n" + func("_", say("2"), ret=RET) + "\n" + func("x$", say("3"), ret=RET),
    "misc")
add("misc/no_main_obj", func("helper", say("h"), ret=RET), "misc", kinds=["lin-asm", "lin-obj", "win-asm", "win-obj"])

# --------------------------------------------------------------------------------------------
# multi/: multiple input files
# --------------------------------------------------------------------------------------------
add("multi/three_files", None, "multi", files={
    "main.memeasm": func("main", [call("alpha"), call("beta")] + say("\n") + ["upgrade", "upvote rcx",
                     "corporate needs you to find the difference between rcx and 3", "fuck go back",
                     "they're the same picture"]),
    "alpha.memeasm": func("alpha", say("alpha ") + ["rcx is brilliant, but I like 0", "upgrade", "upvote rcx",
                          "corporate needs you to find the difference between rcx and 2", "fuck go back",
                          "they're the same picture"], ret=RET),
    "beta.memeasm": func("beta", say("beta") + ["monke uauauauu"], ret=RET)})
add("multi/lib_print", None, "multi", files={
    "main.memeasm": func("main", ["rax is brilliant, but I like 1234567"] + shownum("rax") + show("rax")),
    "lib.memeasm": LIB_NUM + "\n" + LIB_HEX})

# --------------------------------------------------------------------------------------------
# obj/: functions linked against C drivers (tests/e2e/drivers)
# --------------------------------------------------------------------------------------------
add("obj/sum3", func("sum3", ["What the hell happened here? long sum3(long a, long b, long c)",
                              "rax is brilliant, but I like rdi", "rax units are ready, with rsi more well on the way",
                              "rax units are ready, with rdx more well on the way"], ret=RET) + "\n" +
    func("hello", say("hello from memeasm\n"), ret=RET), "obj",
    kinds=["lin-obj", "lin-asm", "win-asm", "win-obj"], driver="tests/e2e/drivers/sum3_driver.c")

# --------------------------------------------------------------------------------------------
# martyrdom/
# --------------------------------------------------------------------------------------------
add("martyrdom/loop", func("main", say("R\n") + ["you shall not pass!"]), "martyrdom",
    kinds=["lin-exe", "lin-asm", "win-asm"], martyrdom=True, timeout=3)
add("martyrdom/bully_no_main", func("loop", say("R\n") + ["you shall not pass!"]), "martyrdom",
    kinds=["lin-exe", "lin-asm", "win-asm"], martyrdom=True, timeout=3, flags=["-fcompile-mode", "bully"],
    martyrdom_variants=[[]])

# --------------------------------------------------------------------------------------------
# errors/: compile-time diagnostics (exact match, noob + obfuscated)
# --------------------------------------------------------------------------------------------
EK = ["lin-exe", "lin-asm", "win-asm"]


def err(name, text, kinds=EK, **kw):
    add(f"errors/{name}", text, "errors", kinds=kinds, obfuscated=True, **kw)


err("invalid_command", func("main", ["this is not a meme", "what can I say", "upvote", "stonks"]))
err("standalone_or_draw_25", func("main", ["or draw 25"]))
err("two_pointers", func("main", ["rax do you know de wey is brilliant, but I like rbx do you know de wey"]))
err("decimal_pointer", func("main", ["rax is brilliant, but I like 5 do you know de wey", "stonks 7 do you know de wey"]))
err("char_pointer", func("main", ["rax is brilliant, but I like a do you know de wey",
                                   "rax is brilliant, but I like \\n do you know de wey",
                                   "rbx is brilliant, but I like 200 do you know de wey",
                                   "what can I say except b do you know de wey"]))
err("monke_pointer", func("main", ["monke uau do you know de wey", "return to monke uau"]))
err("function_pointer", func("main", ["main do you know de wey: whomst has summoned the almighty one"]))
err("invalid_parameters", func("main", ["stonks al", "not stonks 5", "what can I say except rax", "upvote 5",
                                         "sneak 100 xyz", "let me in. LET ME IIIIIIIIN rax", "monke abc",
                                         "rax is getting out of hand, now there are al of them",
                                         "al is getting out of hand, now there are 5 of them",
                                         "look at what eax needs to mimic a fraction of rax",
                                         "it's dangerous to go alone, take al", "it's over 9000 5",
                                         "rax UNLIMITED POWER eax", "rax is brilliant, but I like abc",
                                         "rax is brilliant, but I like 256x", "what can I say except 256",
                                         "what can I say except -1", "1abc: whomst has summoned the almighty one",
                                         "a-b: whomst has summoned the almighty one"]) + "\n" + func("9lives", []))
err("does_not_fit", func("main", ["al is brilliant, but I like 256", "ax is brilliant, but I like 65536",
                                   "eax is brilliant, but I like 4294967296", "al is brilliant, but I like -129",
                                   "bitconneeeeeeect ah 300", "parry 70000 you filthy casual cx"]))
err("not_sign_extendable", func("main", ["rax units are ready, with 2147483648 more well on the way",
                                          "parry -2147483649 you filthy casual rbx", "bitconneeeeeeect rcx 4294967295",
                                          "rax is getting out of hand, now there are 9223372036854775807 of them",
                                          "who would win? rax or 5000000000", "5000000000 wins", "rax wins",
                                          "rax is brilliant, but I like 5000000000"]))
err("register_size_mismatch", func("main", ["rax is brilliant, but I like eax", "al units are ready, with bx more well on the way",
                                             "parry r8d you filthy casual r9w", "bitconneeeeeeect ah rax"]))
err("operand_size_unknown", func("main", ["rax do you know de wey is brilliant, but I like 5",
                                           "rbx do you know de wey units are ready, with a more well on the way"]))
err("duplicate_function", func("main", []) + "\n" + func("main", []))
err("duplicate_markers", func("main", ["upgrade", "upgrade", "fuck go back", "banana", "banana", "where banana",
                                        "monke uau", "monke uau", "return to monke uau",
                                        "corporate needs you to find the difference between rax and 1",
                                        "they're the same picture", "they're the same picture",
                                        "who would win? rax or rbx", "rax wins", "rax wins", "rbx wins"]))
err("undefined_markers", func("main", ["fuck go back", "where banana", "return to monke uuaa",
                                        "corporate needs you to find the difference between rax and 1",
                                        "who would win? rax or rbx", "who would win? rcx or 5", "rcx wins"]))
err("undefined_function", func("main", [call("nowhere")]))
err("undefined_function_asm_ok", func("main", [call("nowhere")]), kinds=["lin-asm", "lin-obj", "win-asm"])
err("no_main", func("notmain", []))
err("no_return_before_def", FUN + "main\n    oh no! anyway\n" + func("f", []))
err("function_does_not_return", FUN + "main\n    what can I say except a\n")
err("orphan_commands", "what can I say except a\n" + func("main", []) + "what can I say except b\n")
err("empty_file", "")
err("comments_only", "What the hell happened here? nothing\n\n   \n")
err("many_errors", func("main", ["stonks al", "upvote 5", "fuck go back", "rax is brilliant, but I like eax",
                                  "nonsense line here", "rax is brilliant, but I like 69"]) + "\n" + func("main", []))
err("nice_and_error", func("main", ["rax is brilliant, but I like 69", "stonks 420", "upvote 69"]))
err("multi_file_duplicate", None, files={"one.memeasm": func("main", []), "two.memeasm": func("main", [])})
err("multi_file_empty_second", None, files={"one.memeasm": func("main", []), "two.memeasm": ""})

# errors raised by the assembler/linker in the reference (gcc fails). The new compiler must also
# fail (same exit code, same stdout); stderr is only compared loosely (see run.py).
AK = ["lin-exe", "lin-obj", "lin-asm", "win-asm", "win-obj"]
for name, line in [
    ("wait_thats_illegal", "wait, that's illegal"),
    ("push_imm64", "stonks 12345678901"),
    ("push_imm_2p31", "stonks 2147483648"),
    ("xor_mem_mem", "sneak 100 rax do you know de wey"),
    ("ambiguous_inc", "upvote rax do you know de wey"),
    ("ambiguous_not", "rax do you know de wey \\s"),
    ("ambiguous_shl", "upgrades, people. Upgrades rbx do you know de wey"),
    ("ambiguous_cmp9000", "it's over 9000 rax do you know de wey"),
    ("byte_reg_base", "what can I say except al do you know de wey"),
    ("ah_with_rex", "ah is brilliant, but I like spl"),
    ("ah_with_r8b", "bitconneeeeeeect ah r8b"),
    ("rdrand_mem", "it's dangerous to go alone, take rax do you know de wey"),
    ("idiv_mem_mem", "look at what rsp do you know de wey needs to mimic a fraction of rax"),
    ("power_mem_mem", "rax UNLIMITED POWER rbx do you know de wey"),
]:
    correct = {"kind": "no_toolchain_error"}
    if name == "wait_thats_illegal":  # fix #1: the -S text changes too
        correct = {"kind": "per_kind", "default": correct,
                   "lin-asm": {"kind": "asm", "present": [r"xor r13, r13"]},
                   "win-asm": {"kind": "asm", "present": [r"xor r13, r13"]}}
    # the fix (analyser error / bully fix-up / valid encoding) necessarily changes the -S output as well
    add(f"asmerr/{name}", func("main", [line]), "asm-errors", kinds=AK, ref_bug={
        "desc": f"reference emits assembly that gas rejects for '{line}' (gcc fails)",
        "bug_kinds": ["lin-exe", "lin-obj", "win-obj", "lin-asm", "win-asm"],
        "correct": correct})

# --------------------------------------------------------------------------------------------
# bully/: errors "fixed" by -fcompile-mode bully (deterministic pseudo-randomness)
# --------------------------------------------------------------------------------------------
BF = ["-fcompile-mode", "bully"]
# reference bug #5 (docs/rust-rewrite-changes.md): only the executable path injects a main, so -S/-O are unaffected
WIN_NO_MAIN = {"desc": "bully mode, no main, Windows executable: the injected main contains the martyrdom code "
                       "referencing killParent/.LsigStruct (never emitted on Windows), so linking always fails",
               "bug_kinds": ["win-exe", "win-run"],
               "correct": {"kind": "per_kind", "win-exe": {"kind": "compile_ok"}, "win-run": {"kind": "linux_runs"}}}


def bully(name, text, **kw):
    kw.setdefault("timeout", 3)
    kw.setdefault("flags", BF)
    add(f"bully/{name}", text, "bully", **kw)


bully("invalid_commands", func("main", say("a") + ["this is not a command", "neither is this one"] + say("b\n")))
bully("many_invalid_commands", func("main", say("<") + [f"garbage line number {i} {'x' * i}" for i in range(12)] + say(">\n")), random=True,
      note="bully replaces one of the invalid lines with 'confused stonks' (random jump target)")
bully("invalid_params", func("main", say("p") + ["stonks al", "not stonks 5", "upvote 12", "sneak 100 xyz",
                                                 "rax is brilliant, but I like abc", "monke abc",
                                                 "what can I say except rax", "let me in. LET ME IIIIIIIIN 5",
                                                 "it's dangerous to go alone, take 7"] + say("q\n")), stdin=b"zz")
bully("invalid_funcname", func("main", say("f") + ["1abc: whomst has summoned the almighty one"] + say("\n")) + "\n" +
      "".join(func(n, say(n[0]), ret=RET) + "\n" for n in ["mprotect", "kill", "signal", "raise", "dump", "atoi",
               "generateExcellence", "isExcellent", "memeify", "uwufy", "test", "helloWorld", "snake_case_sucks",
               "gets", "uwu", "skillIssue"]))
bully("pointers", func("main", ["rax is brilliant, but I like rsp", "stonks 1",
                                "rbx is brilliant, but I like 5 do you know de wey", "stonks a do you know de wey",
                                "monke uau do you know de wey", "rbx is brilliant, but I like rsp",
                                "rax do you know de wey is brilliant, but I like rbx do you know de wey",
                                "not stonks rcx", "not stonks rcx"] + say("ok\n")))
bully("operand_size_unknown", runprog(["stonks 0", "stonks 0", "rax is brilliant, but I like rsp",
                                            "rax do you know de wey is brilliant, but I like 65",
                                            "rax do you know de wey units are ready, with 1 more well on the way",
                                            "not stonks rbx", "not stonks rcx"] + show("rbx") + show("rcx")))
bully("does_not_fit", runprog(["rax is brilliant, but I like 0", "rbx is brilliant, but I like 0", "al is brilliant, but I like 300", "bx is brilliant, but I like 99999"] + show("rax") + show("rbx")))
bully("not_sign_extendable", runprog(["rax is brilliant, but I like 0",
                                           "rax units are ready, with 2147483648 more well on the way"] + show("rax")))
bully("not_sign_extendable_long", runprog(["rax is brilliant, but I like 0",
                                                "rax units are ready, with 12345678901234567 more well on the way"] + show("rax")))
bully("register_mismatch", func("main", ["rax is brilliant, but I like 5", "rbx is brilliant, but I like eax",
                                         "cl is brilliant, but I like r9w", "dx units are ready, with rsi more well on the way"]
                                + say("ok\n")))
bully("duplicate_markers", runprog(["rcx is brilliant, but I like 0", "upgrade", "upvote rcx", "upgrade", "upvote rcx",
                                         "corporate needs you to find the difference between rcx and 6", "fuck go back",
                                         "they're the same picture", "they're the same picture",
                                         "monke uau", "monke uau"] + show("rcx")))
bully("duplicate_function", func("main", say("1\n")) + "\n" + func("main", say("2\n")))
bully("undefined_markers", func("main", say("s") + ["fuck go back", "where banana", "return to monke uuaa",
                                                    "corporate needs you to find the difference between rax and 1",
                                                    "who would win? rax or rbx", "rax wins"] + say("e\n")))
bully("undefined_call", func("main", say("c") + [call("nowhere")] + say("\n")))
bully("no_main", func("first", say("first\n"), ret=WIN) + "\n" + func("second", say("second\n"), ret=RET),
      ref_bug=WIN_NO_MAIN)
bully("no_return", FUN + "main\n    rax is brilliant, but I like 7\n" + "\n".join("    " + l for l in say("nr\n")) + "\n")
bully("no_return_then_def", FUN + "main\n    what can I say except x\n    rax is brilliant, but I like 3\n" + func("g", say("g\n"), ret=RET))
bully("orphans", "what can I say except o\n" + func("main", say("m\n")) + "what can I say except z\nupvote rax\n")
bully("orphans_only", "what can I say except o\nwhat can I say except \\n\nrax is brilliant, but I like 5\n",
      ref_bug=WIN_NO_MAIN)
bully("mixed", func("main", say("x") + ["stonks al", "blub", "upvote 5", "rax is brilliant, but I like eax",
                                        "fuck go back", "upgrades, people. Upgrades 7"] + say("y\n")) + "\n" + func("main", []),
      ref_bug={"desc": "the bully fix-ups of this program read uninitialised registers: 'stonks al' becomes an unbalanced "
                       "'push rax' (rax as left by the start-up code) that main's 'ret' pops, and 'blub' becomes a "
                       "'syscall' with start-up register contents. The reference (glibc start-up) returns to garbage "
                       "after 'xy\\n'; new executables enter main with rax = &main (docs/rust-rewrite-changes.md, "
                       "'Other known divergences') and print 'xy\\n' twice. The -S/-O output is still compared exactly.",
               "bug_kinds": ["lin-exe"], "correct": {"kind": "unspecified"}})
add("bully/dot_path", None, "bully", flags=BF, cwd="tests/e2e/programs/bully/dot_path", files={
    "zz_main.memeasm": func("main", say("<") + ["some invalid thing", "another 1 2 3", "stonks al"] + say(">\n")),
    "aa_other.memeasm": func("other", ["what is this", "rax is brilliant, but I like xyz"], ret=RET)}, timeout=3,
    argfiles=["./zz_main.memeasm", "aa_other.memeasm"], random=True,
    note="bully seed depends on inputFileName[0] ('.'); the generated fixes include a random command")

# --------------------------------------------------------------------------------------------
# bugs/: inputs on which the reference compiler is buggy (crash/UB/unassemblable output).
# run.py records what the reference does; if the new compiler behaves like the reference on a
# "bug kind" the check is OLDBUG, if it satisfies "correct" it is FIXED.
#   correct.kind: per_kind ({"<check kind>": spec, "default": spec}) | no_toolchain_error | compile_ok | terminates (no compiler crash/hang) | runs (oracle program/flags give the correct
#   runtime behaviour) | asm (regexes present/absent in -S) | martyrdom | unspecified
# --------------------------------------------------------------------------------------------


def with_oracle(spec, path):
    """Attaches the oracle program to the "runs" spec, wherever it is nested (see run.py find_oracle_spec)."""
    if spec["kind"] == "runs":
        return dict(spec, oracle=path)
    if spec["kind"] == "any_of":
        return dict(spec, of=[with_oracle(s, path) for s in spec["of"]])
    if spec["kind"] == "per_kind":
        return {k: (v if k == "kind" else with_oracle(v, path)) for k, v in spec.items()}
    return spec


def bug(name, text, desc, bug_kinds, correct, kinds=None, oracle=None, **kw):
    if oracle is not None:
        PROGRAMS[f"bugs/{name}.oracle.memeasm"] = oracle
        correct = with_oracle(correct, f"tests/e2e/programs/bugs/{name}.oracle.memeasm")
    add(f"bugs/{name}", text, "bugs", kinds=kinds, ref_bug={"desc": desc, "bug_kinds": bug_kinds, "correct": correct}, **kw)


RUNS = {"kind": "runs"}
bug("char_backslash", runprog(["rax is brilliant, but I like -1", "al is brilliant, but I like \\"] + show("rax")
                              + ["what can I say except \\"] + say("\n")),
    "char parameter '\\' is emitted as '\\' in gas syntax, which gas reads as an escaped quote (0x27 instead of 0x5C)",
    ["lin-exe"], RUNS, kinds=["lin-exe", "lin-asm", "win-asm"],
    oracle=runprog(["rax is brilliant, but I like -1", "al is brilliant, but I like 92"] + show("rax")
                   + ["what can I say except 92"] + say("\n")))
bug("char_vtab", runprog(["rax is brilliant, but I like -1", "al is brilliant, but I like \\v"] + show("rax")
                         + ["what can I say except \\v"] + say("\n")),
    "escape '\\v' is emitted as gas '\\v', which gas does not know (assembles as 'v' = 0x76 instead of 0x0B)",
    ["lin-exe"], RUNS, kinds=["lin-exe", "lin-asm", "win-asm"],
    oracle=runprog(["rax is brilliant, but I like -1", "al is brilliant, but I like 11"] + show("rax")
                   + ["what can I say except 11"] + say("\n")))
IDIV_CASES = [("7", "rbx", "-1", "100"), ("3", "r15", "5", "-9"), ("-7", "rcx", "-3", "50")]
bug("idiv_sign", runprog(sum([[f"rax is brilliant, but I like {a}", "rdx is brilliant, but I like 5",
                               f"{d} is brilliant, but I like {v}", f"look at what {s} needs to mimic a fraction of {d}"]
                              + show(d) + show("rdx") for s, d, a, v in IDIV_CASES], [])),
    "division: cqo sign-extends the old rax instead of the dividend, so the result is wrong when their signs differ",
    ["lin-exe"], RUNS, kinds=["lin-exe", "lin-asm", "win-asm"],
    oracle=runprog(sum([[f"rax is brilliant, but I like {v}", "rdx is brilliant, but I like 5",
                         f"{d} is brilliant, but I like {v}", f"look at what {s} needs to mimic a fraction of {d}"]
                        + show(d) + show("rdx") for s, d, a, v in IDIV_CASES], [])))
bug("function_dollar", func("main", [call("$")] + say("\n")) + "\n" + func("$", say("3"), ret=RET),
    "a function named '$' is emitted as 'call $' which gas reads as 'call <current address>' (endless recursion, SIGSEGV)",
    ["lin-exe"], RUNS, kinds=["lin-exe", "lin-asm", "win-asm"], timeout=3,
    oracle=func("main", [call("dollar")] + say("\n")) + "\n" + func("dollar", say("3"), ret=RET))
bug("power_one", runprog(say("before\n") + ["rax is brilliant, but I like 5", "rax UNLIMITED POWER 1"] + show("rax")
                         + say("after\n")),
    "x UNLIMITED POWER 1 decrements the exponent twice before testing it and loops ~2^64 times",
    ["lin-exe"], RUNS, kinds=["lin-exe", "lin-asm", "win-asm"], timeout=3,
    oracle=runprog(say("before\n") + ["rax is brilliant, but I like 5"] + show("rax") + say("after\n")))
# x ** y for y < 0 (docs/rust-rewrite-changes.md #13): 1 for x = 1, +-1 by the parity of y for x = -1, 0 otherwise
POW_NEG = [("1", "-1", "1"), ("-1", "-3", "-1"), ("-1", "-2", "1"), ("5", "-1", "0"), ("0", "-4", "0")]
bug("power_negative", runprog(say("before\n") + sum([[f"rax is brilliant, but I like {x}", f"rax UNLIMITED POWER {y}"]
                                                     + show("rax") for x, y, _ in POW_NEG], []) + say("after\n")),
    "negative exponent loops ~2^64 times", ["lin-exe"], RUNS, kinds=["lin-exe", "lin-asm", "win-asm"], timeout=3,
    oracle=runprog(say("before\n") + sum([[f"rax is brilliant, but I like {r}"] + show("rax") for _, _, r in POW_NEG], [])
                   + say("after\n")))
bug("decimal_overflow", runprog(["rax is brilliant, but I like 99999999999999999999"] + show("rax")),
    "decimal literal larger than 2^64 is silently saturated to INT64_MAX by strtoll (docs #14: now a compile error "
    "in noob mode)", ["lin-exe", "lin-asm", "win-asm"], {"kind": "diagnostic"}, kinds=["lin-exe", "lin-asm", "win-asm"])
bug("over9000_8bit", runprog(say("before\n") + ["al is brilliant, but I like 41", "it's over 9000 al"] + say("after\n"),
                             hexlib=False),
    "it's over 9000 with an 8-bit register emits 'cmp al, 9000'; gas truncates 9000 to 0x28 (warning hidden by -w), "
    "so al=41 passes the check", ["lin-exe"],
    {"kind": "any_of", "of": [{"kind": "runs"}, {"kind": "diagnostic"}]},
    kinds=["lin-exe", "lin-asm", "win-asm"], timeout=3,
    oracle=runprog(say("before\n") + ["rax is brilliant, but I like 41", "it's over 9000 rax"] + say("after\n"),
                   hexlib=False))
bug("bully_operand_size", runprog(["stonks 0", "rax is brilliant, but I like rsp",
                                   "parry 5 you filthy casual rax do you know de wey", "not stonks rbx"] + show("rbx")),
    "bully mode operand-size check uses paramTypes[index + 1 % 2] (= index+1), so a pointer in the 2nd parameter "
    "gets no size prefix and gas fails with 'ambiguous operand size'",
    ["lin-exe", "lin-obj", "win-obj", "lin-asm", "win-asm"],
    {"kind": "any_of", "of": [{"kind": "no_toolchain_error"},
                              {"kind": "asm", "present": [r"(?i)sub\s+(byte|word|dword|qword) ptr \[rax\], (0x)?5\b"]}]},
    kinds=["lin-exe", "lin-asm", "lin-obj", "win-asm", "win-obj"], flags=BF, timeout=3, exit_code_unspecified=True,
    note="the operand size is the bully's choice (Rust: BYTE PTR), so only successful assembly is required")
# kept on purpose (rust-rewrite-changes.md "Possible bugs, kept"): bully mode inserts the martyrdom code into the
# injected main even with -fno-martyrdom -> exact match
bully("no_main_fno_martyrdom", func("first", say("first\n"), ret=WIN),
      kinds=["lin-exe", "lin-asm", "win-asm"], flags=BF + ["-fno-martyrdom"])
bully("no_main_fno_martyrdom_sigint", func("loop", say("R\n") + ["you shall not pass!"]),
      kinds=["lin-exe"], martyrdom=True, flags=BF + ["-fno-martyrdom"], martyrdom_variants=[[]])
bug("bully_empty_file", "",
    "bully mode with no functions at all: the generated main falls through into writechar and returns an "
    "uninitialised rax (undefined behaviour); docs #16: the injected main now returns 0", ["lin-exe"], RUNS,
    kinds=["lin-exe", "lin-asm", "win-asm"], flags=BF, timeout=3, oracle=func("main", []))
bug("multi_line_comment", func("main", ["Why, why?", "comment", "Oh, that's why"]),
    "multi-line comments ('Why, why?' ... \"Oh, that's why\") are defined in commands.h but never implemented",
    ["lin-exe", "lin-asm", "win-asm"], {"kind": "unspecified"}, kinds=["lin-exe", "lin-asm", "win-asm"])
bug("opt_minus_2", runprog(say("x\n"), hexlib=False),
    "-O-2 and -O-s share enum value -2 (o_s is not given an explicit value), so -O-2 also emits '.align 536870912' "
    "(~1.6 GB executable)", ["lin-asm"],
    {"kind": "asm", "present": [r"\tpush rax\n\tpop rax\n"], "absent": [r"\.align 536870912"]},
    kinds=["lin-exe", "lin-asm"], flags=["-O-2"], heavy=True)
bug("opt_minus_s", runprog(say("x\n"), hexlib=False),
    "-O-s shares the enum value of -O-2, so it also inserts push rax/pop rax after every command", ["lin-asm"],
    {"kind": "asm", "present": [r"\.align 536870912"], "absent": [r"\tpush rax\n\tpop rax\n"]},
    kinds=["lin-exe", "lin-asm"], flags=["-O-s"], heavy=True)
bug("win_print_clobbers_regs", func("main", ["r8 is brilliant, but I like 65", "r9 is brilliant, but I like 66",
                                            "r10 is brilliant, but I like 67", "r11 is brilliant, but I like 68",
                                            "rcx is brilliant, but I like 69", "rdx is brilliant, but I like 70"]
                                   + say("x") + [f"what can I say except {r}" for r in ("r8b", "r9b", "r10b", "r11b", "cl", "dl")]
                                   + say("\n")),
    "Windows: writechar zeroes the saved r9 (5th WriteFile argument stored at [rsp+32] after 'sub rsp, 32') and does "
    "not preserve r10/r11; prints must not modify registers (Linux prints 'xABCDEF')", ["win-run"], {"kind": "linux_runs"},
    kinds=["lin-exe", "win-asm", "win-exe"])
bug("bully_no_main_windows", func("first", say("first\n"), ret=WIN), WIN_NO_MAIN["desc"], WIN_NO_MAIN["bug_kinds"],
    WIN_NO_MAIN["correct"], kinds=["win-asm", "win-obj", "win-exe"], flags=BF, timeout=3)
add("bugs/pb_empty_file", None, "bugs", kinds=["lin-asm", "lin-exe"], flags=BF, timeout=3, files={
    "empty.memeasm": "",
    "pb.memeasm": func("main", ["perfectly balanced as all things should be"] + say("ab\n"))},
    exit_code_unspecified=True,
    ref_bug={"desc": "perfectly balanced with several input files, one of them empty (bully mode): rand() % 0 on the "
                     "empty file, the compiler crashes (SIGSEGV on aarch64, SIGFPE on x86-64)",
             "bug_kinds": ["lin-asm", "lin-exe"],
             "correct": {"kind": "any_of", "of": [{"kind": "compile_ok"}, {"kind": "terminates"}]}},
    note="perfectly balanced may delete the function header; then linking fails legitimately, so only "
         "'the compiler terminates normally' is required")
bug("pb_hang", func("main", ["fuck go back"] * 4 + ["perfectly balanced as all things should be"]),
    "bully mode, perfectly balanced when bully fixes already dropped more lines than remain: the deletion loop "
    "never ends (compiler hangs)", ["lin-asm", "lin-exe"], {"kind": "terminates"},
    kinds=["lin-asm", "lin-exe"], flags=BF, timeout=3, compile_timeout=15, exit_code_unspecified=True)
bug("bully_decimal_truncation", runprog(["rbx is brilliant, but I like 0",
                                         "rbx units are ready, with 77777777777 more well on the way",
                                         "rbx units are ready, with 12345678901234567 more well on the way"]
                                        + say("ok\n"), hexlib=False),
    "bully mode, non-mov 64-bit arithmetic with a decimal that does not fit a sign-extended imm32: the string is "
    "cut at computedIndex % 32 (out-of-bounds write when shorter) and gas rejects the too large immediate",
    ["lin-exe", "lin-asm", "lin-obj", "win-asm", "win-obj"],
    {"kind": "any_of", "of": [{"kind": "no_toolchain_error"},
                              {"kind": "asm", "absent": [r"(?i)add\s+rbx,\s*(0x[0-9a-f]{9,}|\d{11,})"]}]},
    kinds=["lin-exe", "lin-asm", "lin-obj", "win-asm", "win-obj"], flags=BF, timeout=3)

# --------------------------------------------------------------------------------------------
# repository examples / CI workflow programs (manifest-only entries, files are not generated)
# --------------------------------------------------------------------------------------------


def ext(name, files, category, **meta):
    entry = {"name": name, "category": category, "kinds": meta.pop("kinds", ALL_KINDS), "files": files}
    runs = meta.pop("runs", None)
    if runs is not None:
        entry["runs"] = []
        for i, r in enumerate(runs):
            r = dict(r)
            if "stdin" in r:
                sp = f"{name}.{i}.stdin"
                PROGRAMS[sp] = r.pop("stdin")
                r["stdin"] = f"tests/e2e/programs/{sp}"
            entry["runs"].append(r)
    entry.update(meta)
    MANIFEST.append(entry)


BF_HELLO = ("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.")
ext("examples/HelloWorld", ["examples/HelloWorld.memeasm"], "examples")
ext("examples/alphabet", ["examples/alphabet.memeasm"], "examples")
ext("examples/brainfuck", ["examples/brainfuck.memeasm"], "examples", timeout=10,
    win_program_bug="reads argv from rsi (Linux System V ABI); on Windows the argument is in rdx, so it crashes",
    runs=[{"args": [BF_HELLO]}, {"args": ["++++++[>+++++++++++<-]>.+.+.,.,.", ], "stdin": b"xy"}])
ext("examples/is_nice", ["examples/is_nice.memeasm"], "examples",
    runs=[{"stdin": b"69\n"}, {"stdin": b"420\n"}, {"stdin": b"5\n"}, {"stdin": b"abc\n"}, {"stdin": b""}])
ext("examples/reverse-string", ["examples/reverse-string.memeasm"], "examples",
    runs=[{"stdin": b"Hello, MemeAssembly!\n"}, {"stdin": b"x\n"}])
ext("examples/the-link", ["examples/the-link.memeasm"], "examples")
ext("examples/rot13", ["examples/rot13/rot13.memeasm"], "examples", kinds=["lin-obj", "lin-asm", "win-asm", "win-obj"],
    driver="examples/rot13/rot13_test.c")
ext("examples/Stalinsort", ["examples/Stalinsort/stalin-sort.memeasm"], "examples",
    kinds=["lin-obj", "lin-asm", "win-asm", "win-obj"], driver="examples/Stalinsort/stalinsort_test.c")
ext("examples/toupper", ["examples/toupper/toupper.memeasm"], "examples", kinds=["lin-obj", "lin-asm", "win-asm", "win-obj"],
    driver="examples/toupper/toupper_test.c")
ext("workflows/example", [".github/workflows/example.memeasm"], "examples",
    kinds=["lin-asm", "lin-obj", "win-asm", "win-obj"], random=True,
    note="contains confused stonks; CI compiles it with -S -d")
ext("workflows/example_d", [".github/workflows/example.memeasm"], "examples", kinds=["lin-asm"], flags=["-d"], random=True)
ext("workflows/runnable_example", [".github/workflows/runnable_example.memeasm"], "examples")
ext("workflows/runnable_example_d", [".github/workflows/runnable_example.memeasm"], "examples", kinds=["lin-exe"], flags=["-d"])
ext("workflows/multiple_files", [".github/workflows/multiple_files/main.memeasm",
                                 ".github/workflows/multiple_files/function.memeasm"], "examples")
ext("workflows/multiple_files_fail_undefined", [".github/workflows/multiple_files/main.memeasm"], "examples",
    kinds=["lin-exe", "lin-asm", "win-asm"], expect_fail=True)
ext("workflows/multiple_files_fail_no_main", [".github/workflows/multiple_files/function.memeasm"], "examples",
    kinds=["lin-exe", "lin-asm", "win-asm"], expect_fail=True)

# --------------------------------------------------------------------------------------------
# write everything
# --------------------------------------------------------------------------------------------


def main():
    import shutil
    for d in OUT.iterdir() if OUT.exists() else []:
        if d.is_dir():
            shutil.rmtree(d)
        else:
            d.unlink()
    for path, text in PROGRAMS.items():
        p = OUT / path
        p.parent.mkdir(parents=True, exist_ok=True)
        data = text if isinstance(text, bytes) else text.encode()
        p.write_bytes(data)
    with open(OUT / "manifest.json", "w") as f:
        json.dump(MANIFEST, f, indent=1)
        f.write("\n")
    print(f"wrote {len(PROGRAMS)} files, {len(MANIFEST)} programs")


if __name__ == "__main__":
    main()
