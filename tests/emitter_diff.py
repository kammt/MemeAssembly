#!/usr/bin/env python3
"""Differential test: the Rust Emitter vs. GNU as.

For every corpus file (Intel-syntax assembly, exactly what the reference MemeAssembly compiler
writes with -S, or generated instruction lines), GNU as and the Emitter (driven through the
test-only text shim in src/backend/emitter/tests.rs) must agree on:
  * accept/reject, and the set of (line, error message) pairs,
  * .text/.data bytes, section alignment,
  * relocations (offset, kind, target, addend; normalised so that "section symbol + offset"
    and "temporary label" forms compare equal),
  * the emitted (non-temporary) symbols.

Corpus:
  ref.*   reference compilers (Linux, Windows, macOS build) with -S and all option combinations,
          on examples/**, .github/workflows/** and generated MemeAssembly programs exercising
          every command with every legal parameter type/value (+ bully mode replacements).
  mx.*    generated matrix: every instruction pattern x operand combination (valid & invalid).
  neg.*   hand-written negative / edge cases.
For files gas rejects, a `.ok.s` twin with the offending lines commented out is added, so the
remaining lines are still compared byte for byte.

Usage: tests/emitter_diff.sh [--out DIR] [--quick] [--no-gen]
"""
import argparse
import glob
import os
import re
import shutil
import struct
import subprocess
import sys
import zlib
import threading
from concurrent.futures import ThreadPoolExecutor

BIG_SEM = threading.Semaphore(2)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GAS_LINUX = "x86_64-linux-gnu-as"
GAS_WIN = "x86_64-w64-mingw32-as"
REF_LINUX = os.environ.get("MEMEASM_REF_LINUX", "/opt/memeasm-ref/memeasm-ref")
REF_WIN = os.environ.get("MEMEASM_REF_WIN", "/opt/memeasm-ref/memeasm-ref-win")

# --------------------------------------------------------------------------------------------
# Object file parsing / normalisation
# --------------------------------------------------------------------------------------------


def cstr(b, off):
    end = b.index(b"\0", off)
    return b[off:end].decode("latin-1")


def crc_of(b):
    return zlib.crc32(b) & 0xFFFFFFFF


def parse_elf(data):
    shoff = struct.unpack_from("<Q", data, 0x28)[0]
    shentsize, shnum, shstrndx = struct.unpack_from("<HHH", data, 0x3A)
    secs = []
    for i in range(shnum):
        f = struct.unpack_from("<IIQQQQIIQQ", data, shoff + i * shentsize)
        secs.append(dict(name_off=f[0], type=f[1], off=f[4], size=f[5], link=f[6], info=f[7], align=f[8]))
    sh = secs[shstrndx]
    for s in secs:
        s["name"] = cstr(data, sh["off"] + s["name_off"])
    names = {".text": "text", ".data": "data"}
    model = {"sec": {}, "align": {}, "relocs": [], "syms": set()}
    secname = {}
    for i, s in enumerate(secs):
        if s["name"] in names:
            n = names[s["name"]]
            secname[i] = n
            model["sec"][n] = bytearray(data[s["off"]:s["off"] + s["size"]])
            model["align"][n] = max(1, s["align"])
    syms = []
    for s in secs:
        if s["type"] == 2:  # SYMTAB
            strtab = secs[s["link"]]
            for k in range(s["size"] // 24):
                name, info, other, shndx, value, size = struct.unpack_from("<IBBHQQ", data, s["off"] + 24 * k)
                syms.append(dict(name=cstr(data, strtab["off"] + name), bind=info >> 4, type=info & 0xF,
                                 shndx=shndx, value=value))
    for sy in syms[1:]:
        if sy["type"] in (3, 4):  # SECTION, FILE
            continue
        where = "UND" if sy["shndx"] == 0 else secname.get(sy["shndx"], "other")
        bind = "GLOBAL" if sy["bind"] == 1 else "LOCAL"
        model["syms"].add((sy["name"], bind, where, sy["value"] if where != "UND" else 0))
    kinds = {1: ("abs64", 8), 2: ("pc", 4), 4: ("branch", 4), 10: ("abs32", 4), 11: ("abs32s", 4)}
    for s in secs:
        if s["type"] != 4 or s["info"] not in secname:
            continue
        tsec = secname[s["info"]]
        b = model["sec"][tsec]
        for k in range(s["size"] // 24):
            off, info, addend = struct.unpack_from("<QQq", data, s["off"] + 24 * k)
            sy = syms[info >> 32]
            kind, _ = kinds.get(info & 0xFFFFFFFF, ("type%d" % (info & 0xFFFFFFFF), 4))
            if sy["type"] == 3:
                target = ("sec", secname.get(sy["shndx"], "other"), addend)
            elif sy["bind"] == 0 and sy["shndx"] != 0:
                target = ("sec", secname.get(sy["shndx"], "other"), sy["value"] + addend)
            else:
                if sy["shndx"] != 0 and secname.get(sy["shndx"]) == tsec and kind in ("pc", "branch"):
                    # gas keeps relocations against global symbols defined in the same section;
                    # the Emitter resolves them (module contract) – apply them here.
                    struct.pack_into("<i", b, off, sy["value"] + addend - off)
                    continue
                target = ("sym", sy["name"], addend)
            if target[0] == "sec" and kind == "branch":
                kind = "pc"
            model["relocs"].append((tsec, off, kind, target))
    return finish_model(model)


def parse_coff(data):
    machine, nsec, _, symptr, nsym, optsize, _ = struct.unpack_from("<HHIIIHH", data, 0)
    strtab_off = symptr + 18 * nsym

    def name8(raw):
        if raw[:4] == b"\0\0\0\0":
            return cstr(data, strtab_off + struct.unpack_from("<I", raw, 4)[0])
        return raw.rstrip(b"\0").decode("latin-1")

    secs = []
    for i in range(nsec):
        f = struct.unpack_from("<8sIIIIIIHHI", data, 20 + optsize + 40 * i)
        secs.append(dict(name=name8(f[0]), rawsize=f[3], rawptr=f[4], relocptr=f[5], nreloc=f[7], chars=f[9]))
    names = {".text": "text", ".data": "data"}
    model = {"sec": {}, "align": {}, "relocs": [], "syms": set()}
    secname = {}
    for i, s in enumerate(secs):
        if s["name"] in names:
            n = names[s["name"]]
            secname[i + 1] = n
            model["sec"][n] = bytearray(data[s["rawptr"]:s["rawptr"] + s["rawsize"]])
            a = (s["chars"] >> 20) & 0xF
            model["align"][n] = 1 << (a - 1) if a else 16
    syms = {}
    k = 0
    while k < nsym:
        raw, value, secnum, typ, sclass, naux = struct.unpack_from("<8sIhHBB", data, symptr + 18 * k)
        syms[k] = dict(name=name8(raw), value=value, secnum=secnum, sclass=sclass, naux=naux)
        k += 1 + naux
    for sy in syms.values():
        if sy["sclass"] == 103 or (sy["naux"] and sy["name"].startswith(".")):
            continue
        where = "UND" if sy["secnum"] == 0 else secname.get(sy["secnum"], "other")
        if where == "other" and sy["sclass"] == 3:
            continue
        bind = "GLOBAL" if sy["sclass"] == 2 else "LOCAL"
        model["syms"].add((sy["name"], bind, where, sy["value"] if where != "UND" else 0))
    for i, s in enumerate(secs):
        if (i + 1) not in secname:
            continue
        tsec = secname[i + 1]
        b = model["sec"][tsec]
        for r in range(s["nreloc"]):
            off, symidx, typ = struct.unpack_from("<IIH", data, s["relocptr"] + 10 * r)
            sy = syms.get(symidx, dict(name="*ABS*", value=0, secnum=-1, sclass=2))
            if typ == 1:
                kind, size = "abs64", 8
                f = struct.unpack_from("<q", b, off)[0]
                addend = f
            elif 4 <= typ <= 9:
                kind, size = "pc", 4
                f = struct.unpack_from("<i", b, off)[0]
                addend = f - 4 - (typ - 4)
            elif typ in (2, 3):
                kind, size = "abs32", 4
                f = struct.unpack_from("<i", b, off)[0]
                addend = f
            else:
                kind, size, addend = "type%d" % typ, 4, 0
            b[off:off + size] = bytes(size)
            if sy["sclass"] != 2 and sy["secnum"] > 0:
                target = ("sec", secname.get(sy["secnum"], "other"), sy["value"] + addend)
            else:
                if sy["secnum"] > 0 and secname.get(sy["secnum"]) == tsec and kind == "pc":
                    struct.pack_into("<i", b, off, sy["value"] + addend - off)
                    continue
                target = ("sym", sy["name"], addend)
            model["relocs"].append((tsec, off, kind, target))
    return finish_model(model, coff=True)


def finish_model(model, coff=False):
    out = {"relocs": sorted(model["relocs"]), "syms": model["syms"], "align": model["align"]}
    for n in ("text", "data"):
        b = bytes(model["sec"].get(n, b""))
        out[n] = b if len(b) <= (1 << 20) else ("CRC", len(b), crc_of(b))
    if coff:
        out["relocs"] = sorted((s, o, "pc" if k == "branch" else k, t) for s, o, k, t in out["relocs"])
    return out


def parse_emit(text, coff):
    errors = []
    model = {"relocs": [], "syms": set(), "align": {}}
    have = False
    for line in text.splitlines():
        tag, _, rest = line.partition(" ")
        if tag == "ERR":
            ln, _, msg = rest.partition(" ")
            errors.append((int(ln), msg))
        elif tag == "SEC":
            have = True
            name, align, length, payload = rest.split(" ", 3) if rest.count(" ") >= 3 else (rest.split(" ") + [""])[:4]
            model["align"][name] = int(align)
            if payload.startswith("CRC:"):
                model[name] = ("CRC", int(length), int(payload[4:], 16))
            else:
                model[name] = bytes.fromhex(payload)
        elif tag == "REL":
            sec, off, kind, name, addend, scope, defsec, defoff = rest.split(" ")
            name = name.replace("\\x20", " ")
            kind = {"Pc32": "pc", "Branch32": "branch", "Abs64": "abs64", "Abs32": "abs32"}[kind]
            if scope == "local" and defsec != "UND":
                target = ("sec", defsec, int(defoff) + int(addend))
                if kind == "branch":
                    kind = "pc"
            else:
                target = ("sym", name, int(addend))
            if coff and kind == "branch":
                kind = "pc"
            model["relocs"].append((sec, int(off), kind, target))
        elif tag == "SYM":
            name, bind, where, val = rest.split(" ")
            model["syms"].add((name.replace("\\x20", " "), bind, where, int(val)))
    model["relocs"].sort()
    return errors, (model if have else None)


GAS_ERR = re.compile(r"^(?:[^:]*):(\d+): (?:Error|Fatal error): (.*)$")
GAS_ERR0 = re.compile(r"^(?:[^:]*): (?:Error|Fatal error): (?:\S+\.o: )?(.*)$")
GAS_BFD = re.compile(r"^\S*-as: \S+\.o: (.*)$")


def run_gas(path):
    """Returns (errors, model)."""
    win = ".win." in os.path.basename(path)
    obj = path[:-2] + ".gas.o"
    p = subprocess.run([GAS_WIN if win else GAS_LINUX, "-w", path, "-o", obj],
                       capture_output=True, text=True, errors="replace")
    errors = []
    for line in p.stderr.splitlines():
        m = GAS_ERR.match(line)
        if m:
            errors.append((int(m.group(1)), m.group(2)))
            continue
        m = GAS_ERR0.match(line) or GAS_BFD.match(line)
        if m:
            errors.append((0, m.group(1)))
    if p.returncode != 0 or errors:
        if not errors:
            errors.append((0, "gas failed: " + p.stderr.strip()[:200]))
        if os.path.exists(obj):
            os.remove(obj)
        return errors, None
    big = os.path.getsize(obj) > (64 << 20)
    if big:
        BIG_SEM.acquire()
    try:
        with open(obj, "rb") as f:
            data = f.read()
        os.remove(obj)
        return [], (parse_coff(data) if win else parse_elf(data))
    finally:
        if big:
            BIG_SEM.release()


# --------------------------------------------------------------------------------------------
# Corpus generation
# --------------------------------------------------------------------------------------------

R64 = ["rax", "rbx", "rcx", "rdx", "rdi", "rsi", "rsp", "rbp"] + ["r%d" % i for i in range(8, 16)]
R32 = ["eax", "ebx", "ecx", "edx", "edi", "esi", "esp", "ebp"] + ["r%dd" % i for i in range(8, 16)]
R16 = ["ax", "bx", "cx", "dx", "di", "si", "sp", "bp"] + ["r%dw" % i for i in range(8, 16)]
R8 = ["al", "ah", "bl", "bh", "cl", "ch", "dl", "dh", "dil", "sil", "spl", "bpl"] + ["r%db" % i for i in range(8, 16)]
REGS = R64 + R32 + R16 + R8

# MemeAssembly parameter values
DECS = ["0", "1", "-1", "2", "7", "127", "128", "-128", "-129", "255", "256", "-256", "32767", "32768",
        "-32768", "-32769", "65535", "65536", "-65536", "2147483647", "2147483648", "-2147483648",
        "-2147483649", "4294967295", "4294967296", "-4294967296", "9223372036854775807",
        "-9223372036854775808", "99999999999999999999", "-99999999999999999999", "+5", "-0", "007",
        "0100", "9000", "123456789", "-123456789", "0x10"]
CHARS = ["a", "Z", "!", "#", ";", "'", ",", "[", "]", "\\", "~", "`", '"', "@", "$", ".", "-", "+",
         "\\n", "\\s", "space", "\\t", "\\f", "\\b", "\\v", '\\"', "\\?", "\\\\"]
ASCII = ["65", "065", "0", "255", "128", "+65", "00", "010", "32", "1", "9"]

REG_T, DEC_T, CHAR_T = "R", "D", "C"


def values(types):
    out = []
    if "64" in types:
        out += R64
    if "32" in types:
        out += R32
    if "16" in types:
        out += R16
    if "8" in types:
        out += R8
    if "D" in types:
        out += DECS
    if "C" in types:
        out += CHARS
        if "D" not in types:
            out += ASCII
    return out


ALLREG = "64 32 16 8"
TEMPLATES = [
    ("stonks {0}", ["64 D C"]),
    ("not stonks {0}", ["64"]),
    ("bitconneeeeeeect {0} {1}", [ALLREG, ALLREG + " D C"]),
    ("{0} \\s", [ALLREG]),
    ("sneak 100 {0}", [ALLREG]),
    ("{0} is brilliant, but I like {1}", [ALLREG, ALLREG + " D C"]),
    ("upvote {0}", [ALLREG]),
    ("downvote {0}", [ALLREG]),
    ("parry {0} you filthy casual {1}", [ALLREG + " D C", ALLREG]),
    ("{0} units are ready, with {1} more well on the way", [ALLREG, ALLREG + " D C"]),
    ("upgrades, people. Upgrades {0}", [ALLREG]),
    ("they had us in the first half, not gonna lie {0}", [ALLREG]),
    ("{0} is getting out of hand, now there are {1} of them", ["64 32", "64 32 D C"]),
    ("look at what {0} needs to mimic a fraction of {1}", ["64 D C", "64"]),
    ("{0} UNLIMITED POWER {1}", ["64", "64 D C"]),
    ("who would win? {0} or {1}\n{0} wins\n{1} wins", [ALLREG + " D C", ALLREG + " D C"]),
    ("corporate needs you to find the difference between {0} and {1}\nthey're the same picture",
     [ALLREG + " D C", ALLREG + " D C"]),
    ("what can I say except {0}", ["8 C"]),
    ("let me in. LET ME IIIIIIIIN {0}", ["8"]),
    ("it's over 9000 {0}", [ALLREG]),
    ("it's dangerous to go alone, take {0}", ["64 32 16"]),
]
FIXED = ["guess I'll die", "confused stonks", "perfectly balanced as all things should be",
         "wait, that's illegal", "oh no! anyway", "refuses to elaborate and leaves", "you shall not pass!",
         "Houston, we have a problem", "we need air support", "it's a trap", "upgrade\nfuck go back",
         "banana\nwhere banana", "monke uaua\nreturn to monke uaua", "return to monke au\nmonke au",
         "upvote rax or draw 25", "right back at ya, buckaroo", "no, I don't think I will"]
PTR = " do you know de wey"
FUNC_NAMES = ["foo", "rax", "eax", "ax", "al", "r8", "r15", "r8w", "r16", "r31", "xmm0", "st", "cs", "cr0",
              "mm0", "k0", "ymm0", "es", "fs", "sil", "ah", "rip", "byte", "near", "far", "offset", "short",
              "dword", "and", "or", "xor", "not", "mod", "shl", "eq", "flat", "ptr", "$", "a$b", "_", "nop",
              "ret", "mov", "st0", "bnd0", "r8l", "L1", "a.b", ".Lfoo", "RAX", "Rax", "R8", "Byte", "AND",
              "x87", "b", "f", "$foo", "jmp", "call", "dr0", "tmm0", "zmm0", "r8b", "spl", "eip", "tr0", "st1",
              "bnd1", "rflags", "k8", "mm8", "r16w", "R16", "Not", "Mod", "FLAT", "fword", "r31w", "r16d"]


def fun(name):
    return "I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun " + name


def program(lines, name="main"):
    return "\n".join([fun(name)] + ["    " + l for l in "\n".join(lines).split("\n")] +
                     ["    I see this as an absolute win", ""])


def gen_programs(quick):
    """Generated MemeAssembly programs: list of (name, units, bully). A unit is one command (one or
    more lines); units the reference compiler rejects are filtered out before use."""
    progs = []
    for ti, (tmpl, types) in enumerate(TEMPLATES):
        lists = [values(t) for t in types]
        if quick:
            lists = [l[::5] for l in lists]
        units = []
        if len(lists) == 1:
            for a in lists[0]:
                units.append(tmpl.format(a))
                if a in REGS:
                    units.append(tmpl.format(a + PTR))
        else:
            for a in lists[0]:
                for b in lists[1]:
                    units.append(tmpl.format(a, b))
            # pointers on either side
            for a in lists[0]:
                for b in lists[1]:
                    if a in REGS and (b in REGS[::3] or b in DECS[:6] or b in CHARS[:3]):
                        units.append(tmpl.format(a + PTR, b))
                    if b in REGS and (a in REGS[::3] or a in DECS[:6] or a in CHARS[:3]):
                        units.append(tmpl.format(a, b + PTR))
        for ci in range(0, len(units), 400):
            progs.append(("t%02d_%03d" % (ti, ci // 400), units[ci:ci + 400], False))
    progs.append(("fixed", FIXED, False))
    # bully mode: wrong parameter types get replaced pseudo-randomly; pointers to anything.
    bully = []
    for tmpl, types in TEMPLATES:
        n = 2 if "{1}" in tmpl else 1
        for a in ["rax", "eax", "ax", "al", "ah", "5", "a", "uaua", "xyz", "[rbx]", "r8b", "-7", "\\n"]:
            for b in ["rbx", "ebx", "bx", "bl", "bh", "300", "z", "foo", "1x", "r9d"]:
                bully.append(tmpl.format(a, b))
                bully.append(tmpl.format(a + PTR, b))
                if n == 2:
                    bully.append(tmpl.format(a, b + PTR))
    for ci in range(0, len(bully), 300):
        progs.append(("bully_%03d" % (ci // 300), bully[ci:ci + 300], True))
    return progs


REF_ERR = re.compile(r"^[^:]*:(\d+): ")


def filter_units(units, bully, path):
    """Drop the units the reference compiler rejects; returns the final source (or None)."""
    units = list(units)
    for _ in range(8):
        starts = []
        line = 2
        for u in units:
            starts.append(line)
            line += u.count("\n") + 1
        with open(path, "w") as f:
            f.write(program(units))
        flags = ["-fcompile-mode", "bully"] if bully else []
        p = subprocess.run([REF_LINUX] + flags + ["-S", "-o", os.devnull, path], capture_output=True, text=True,
                           errors="replace")
        if p.returncode == 0:
            return units
        bad = set()
        for l in p.stdout.splitlines() + p.stderr.splitlines():
            m = REF_ERR.match(l)
            if m:
                ln = int(m.group(1))
                for i, st in enumerate(starts):
                    if st <= ln < st + units[i].count("\n") + 1:
                        bad.add(i)
        if not bad:
            return None
        units = [u for i, u in enumerate(units) if i not in bad]
    return None


def strip_stabs(asm):
    return "".join(l for l in asm.splitlines(True) if not l.lstrip().startswith((".stabs", ".stabn")))


def compile_ref(compiler, src_files, flags, out):
    p = subprocess.run([compiler] + flags + ["-S", "-o", out] + src_files, capture_output=True, text=True,
                       errors="replace")
    if p.returncode != 0 or not os.path.exists(out):
        if os.path.exists(out):
            os.remove(out)
        return False
    with open(out, errors="surrogateescape") as f:
        asm = f.read()
    with open(out, "w", errors="surrogateescape") as f:
        f.write(strip_stabs(asm))
    return True


def build_mac_ref(outdir):
    exe = os.path.join(outdir, "memeasm-ref-mac")
    if os.path.exists(exe):
        return exe
    srcs = glob.glob(os.path.join(ROOT, "compiler", "**", "*.c"), recursive=True)
    p = subprocess.run(["gcc", "-std=gnu17", "-w", "-DMACOS", "-o", exe] + srcs, capture_output=True, text=True)
    return exe if p.returncode == 0 else None


BIG_OK = ("HelloWorld", "multiple_files")
FLAGSETS = [[], ["-g"], ["-O-1"], ["-O-2"], ["-O-3"], ["-O69420"], ["-fcompile-mode", "bully"], ["-fno-martyrdom"],
            ["-fcompile-mode", "bully", "-fno-martyrdom"], ["-fcompile-mode", "obfuscated"]]


def gen_ref(corpus, quick, stats):
    srcdir = os.path.join(corpus, "..", "memeasm")
    os.makedirs(srcdir, exist_ok=True)
    inputs = []
    for f in sorted(glob.glob(os.path.join(ROOT, "examples", "**", "*.memeasm"), recursive=True) +
                    glob.glob(os.path.join(ROOT, ".github", "workflows", "**", "*.memeasm"), recursive=True)):
        inputs.append((os.path.basename(f)[:-8].replace(".", "_"), [f]))
    mf = sorted(glob.glob(os.path.join(ROOT, ".github", "workflows", "multiple_files", "*.memeasm")))
    if mf:
        inputs.append(("multiple_files", mf))
    def make(prog):
        name, units, bully = prog
        p = os.path.join(srcdir, name + ".memeasm")
        units = filter_units(units, bully, p)
        return ("gen_" + name, [p], bully, len(units or []), units is not None)

    with ThreadPoolExecutor(os.cpu_count()) as ex:
        made = list(ex.map(make, gen_programs(quick)))
    stats["memeasm_commands_generated"] = sum(m[3] for m in made)
    bully_inputs = set()
    for name, files, bully, n, ok in made:
        if ok:
            inputs.append((name, files))
            if bully:
                bully_inputs.add(name)
    for i, fname in enumerate(FUNC_NAMES):
        p = os.path.join(srcdir, "call%02d.memeasm" % i)
        with open(p, "w") as f:
            f.write(program(["%s: whomst has summoned the almighty one" % fname]) +
                    program(["right back at ya, buckaroo"], fname))
        inputs.append(("gen_call%02d" % i, [p]))
    compilers = [("lin", REF_LINUX), ("win", REF_WIN)]
    mac = build_mac_ref(os.path.join(corpus, ".."))
    if mac:
        compilers.append(("mac", mac))
    jobs = []
    for cname, comp in compilers:
        for name, files in inputs:
            for flags in FLAGSETS:
                if name.startswith("gen_") and flags not in ([], ["-fcompile-mode", "bully"], ["-O-3"], ["-g"]):
                    continue
                if name in bully_inputs and "bully" not in flags:
                    continue
                if flags == ["-O-2"] and name not in BIG_OK:
                    continue  # -O-2 aligns to 512 MiB; keep those objects few
                tag = "".join(f.strip("-").replace(" ", "") for f in flags) or "plain"
                jobs.append((comp, files, flags, os.path.join(corpus, "ref.%s.%s.%s.s" % (cname, name, tag))))
        for name, files in inputs:
            if name in BIG_OK:
                jobs.append((comp, files, ["-O-s"], os.path.join(corpus, "ref.%s.%s.Os.s" % (cname, name))))
    with ThreadPoolExecutor(os.cpu_count()) as ex:
        res = list(ex.map(lambda j: compile_ref(*j), jobs))
    stats["ref_compiled"] = sum(res)
    stats["ref_rejected_by_memeasm"] = len(res) - sum(res)


# ---- instruction matrix ----

MEMS_BASE = ["[rax]", "[rbx]", "[rsp]", "[rbp]", "[r8]", "[r12]", "[r13]", "[r15]", "[eax]", "[esp]", "[ebp]",
             "[r13d]", "[ax]", "[al]", "[rsp + 8]", "[rsp + 32]", "[rip + .Ld]", "[rip + d0]", "[rip + und]",
             "[rip + .Ltxt]", "[69]", "[rip + gtxt]", "[rip + .Ld + 3]", "[rbp + 200]"]
SIZES = ["", "BYTE PTR ", "WORD PTR ", "DWORD PTR ", "QWORD PTR ", "XMMWORD PTR "]
IMMS = ["0", "1", "0x1", "0x7F", "0x80", "0xFF", "0x100", "0x7FFF", "0x8000", "0xFFFF", "0x10000", "0x1FFFF",
        "0x7FFFFFFF", "0x80000000", "0xFFFFFFFF", "0x100000000", "0x1FFFFFFFF", "0x7FFFFFFFFFFFFFFF",
        "0x8000000000000000", "0xFFFFFFFFFFFFFF80", "0xFFFFFFFFFFFFFF7F", "0xFFFFFFFFFFFF8000",
        "0xFFFFFFFFFFFF7FFF", "0xFFFFFFFF80000000", "0xFFFFFFFF7FFFFFFF", "0xFFFFFFFFFFFFFFFF", "-1", "-128",
        "-129", "+65", "065", "-0", "9000", "25", "8", "32", "0x2A", "200", "-5", "1000"]
PRELUDE = (".intel_syntax noprefix\n.data\n.Ld: .byte 0, 0, 0, 0, 0, 0, 0, 0\nd0: .quad 0\n.text\n"
           ".global gtxt\ngtxt: nop\n.Ltxt: nop\n")


def gen_matrix(corpus, quick):
    lines = []
    regs = REGS + ["xmm0", "xmm8"]
    mems = [s + m for m in MEMS_BASE for s in SIZES]
    imms = IMMS
    if quick:
        regs, mems, imms = regs[::4], mems[::7], imms[::3]
    dst = regs + mems
    src = regs + mems + imms
    for op in ["mov", "add", "sub", "and", "xor", "cmp", "test", "imul", "lea", "movups"]:
        for a in dst:
            for b in src:
                lines.append("%s %s, %s" % (op, a, b))
    for op in ["not", "inc", "dec", "push", "pop", "idiv", "rdrand", "imul"]:
        for a in regs + mems + imms:
            lines.append("%s %s" % (op, a))
    for op in ["shl", "shr"]:
        for a in dst:
            for b in ["1", "0x1", "2", "0", "8", "255", "256", "-1", "-128", "-129", "0x1FF", "0xFFFFFFFF",
                      "0xFFFFFFFFFFFFFFFF", "cl"]:
                lines.append("%s %s, %s" % (op, a, b))
    for op in ["cqo", "ret", "nop", "hlt", "int3", "syscall"]:
        lines.append(op)
    for i in range(0, len(lines), 2000):
        with open(os.path.join(corpus, "mx.%05d.s" % (i // 2000)), "w") as f:
            f.write(PRELUDE + "\n".join(lines[i:i + 2000]) + "\n")
    return len(lines)


def gen_branches(corpus):
    """Branch relaxation and symbol cases."""
    cases = {}
    conds = ["jmp", "je", "jne", "jz", "jnz", "jg", "jl", "jge", "jle", "ja", "jb"]
    for n in [0, 1, 50, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 200, 1000]:
        for c in conds:
            body = "nop\n" * n
            cases["br_fwd_%s_%d" % (c, n)] = "%s 1f\n%s1:\nret\n" % (c, body)
            cases["br_bwd_%s_%d" % (c, n)] = "1:\n%s%s 1b\nret\n" % (body, c)
            cases["br_lbl_%s_%d" % (c, n)] = ".Lt:\n%s%s .Lt\n%s .Lu\n%s.Lu: ret\n" % (body, c, c, body)
    # chains where relaxing one branch pushes another out of range
    chain = []
    for i in range(40):
        chain.append("jz .La%d" % i)
        chain.append("mov rax, 1")
    for i in range(40):
        chain.append(".La%d: jmp .La%d" % (i, (i * 7) % 40))
        chain.append("mov rax, 0x123456789")
    cases["br_chain"] = "\n".join(chain) + "\n"
    cases["br_misc"] = "\n".join([
        ".global f", "f: ret", "jmp f", "jz f", "call f", "lea rax, [rip + f]", "g: jmp g", "call g",
        "lea rax, [rip + g]", "jmp .Ldat", "call .Ldat", "jmp und", "jz und", "call und", "call .Lund",
        "jmp .Lund", "call $", "jmp $", "jmp .", "call rax", "call r8", "call ax", "call r9w", "jmp rax",
        "jmp r12", "jmp ax", "call r16", "call r31", "jmp r20", ".data", ".Ldat: .byte 1", "dd: .quad 5", ".text",
        "lea rax, [rip + dd]", "call dd", "jmp dd", "1: 1: jmp 1b", "jmp 1f", "1:", "2: jz 2b", "jnz 2f", "2: nop",
        ".global gu", ".global gd", "gd: nop", ".extern ext1", ".extern ext2", "call ext2",
        "mov BYTE PTR [rip + .Ldat], 0x61", "mov QWORD PTR [rip + dd], 5", "cmp QWORD PTR [rip + dd], 0",
        "mov WORD PTR [rip + dd], 0x1234", "mov DWORD PTR [rip + .Lund2], 0x12345678",
    ]) + "\n"
    cases["neg_dup"] = "a:\na:\n.Lb:\n.Lb:\n"
    cases["neg_fb"] = "jmp 3b\njmp 4f\n"
    cases["neg_textlabel"] = ".text:\n"
    cases["neg_badlabels"] = "\n".join([
        ".L[rax]Wins_0:", "jl .L[rax]Wins_0", ".L-5Wins_0:", "jg .L-5Wins_0", ".LBYTE PTR [rax]Wins_0:",
        "jl .LBYTE PTR [rax]Wins_0", "xor r13 r13", "\t0", "jmp .L+5Wins_0", ".L+5Wins_0:"]) + "\n"
    calls = []
    for n in FUNC_NAMES:
        calls.append("call %s" % n)
        calls.append("jmp %s" % n)
        calls.append("jz %s" % n)
    cases["neg_calls"] = "\n".join(calls) + "\n"
    cases["align_text"] = "nop\n.align 16\nret\n.align 128\nmov rax, 1\n.align 256\n.data\n.byte 1\n.align 8\n"
    for k in range(0, 140, 7):
        cases["align_%d" % k] = "nop\n" * k + "ret\n.align 128\n"
    for name, body in cases.items():
        for tgt in ("lin", "win"):
            with open(os.path.join(corpus, "neg.%s.%s.s" % (tgt, name)), "w") as f:
                f.write(".intel_syntax noprefix\n" + body)
    return len(cases) * 2


# --------------------------------------------------------------------------------------------
# Driver
# --------------------------------------------------------------------------------------------


def comment_out(path, err_lines, newpath):
    with open(path, errors="surrogateescape") as f:
        lines = f.read().split("\n")
    for ln in err_lines:
        if 1 <= ln <= len(lines):
            lines[ln - 1] = "# " + lines[ln - 1]
    with open(newpath, "w", errors="surrogateescape") as f:
        f.write("\n".join(lines))


def gas_phase(corpus):
    files = sorted(glob.glob(os.path.join(corpus, "*.s")))
    results = {}

    def one(path):
        out = [(path, run_gas(path))]
        errs, _ = out[0][1]
        cur = path
        for it in range(3):
            if not errs:
                break
            lines = {l for l, _ in errs if l > 0}
            if not lines:
                break
            okp = path[:-2] + ".ok%d.s" % it if it else path[:-2] + ".ok.s"
            comment_out(cur, lines, okp)
            r = run_gas(okp)
            out.append((okp, r))
            errs, cur = r[0], okp
        return out

    with ThreadPoolExecutor(os.cpu_count()) as ex:
        for res in ex.map(one, files):
            for p, r in res:
                results[p] = r
    return results


def run_emitter(corpus):
    env = dict(os.environ, EMITTER_CORPUS=os.path.abspath(corpus))
    p = subprocess.run(["cargo", "test", "--release", "--quiet", "--", "--ignored", "--exact",
                        "backend::emitter::tests::dump_corpus", "--nocapture"],
                       cwd=ROOT, env=env, capture_output=True, text=True)
    if p.returncode != 0:
        print(p.stdout[-3000:], p.stderr[-3000:])
        sys.exit("emitter dump failed")


def describe_diff(a, b):
    if isinstance(a, tuple) or isinstance(b, tuple):
        return "%r vs %r" % (a, b)
    for i in range(min(len(a), len(b))):
        if a[i] != b[i]:
            return "first diff at 0x%x: gas %s / emitter %s (len %d/%d)" % (
                i, a[i:i + 12].hex(), b[i:i + 12].hex(), len(a), len(b))
    return "length gas %d emitter %d" % (len(a), len(b))


def compare(path, gas, mine_text):
    coff = ".win." in os.path.basename(path)
    gerr, gmodel = gas
    merr, mmodel = parse_emit(mine_text, coff)
    problems = []
    if sorted(set(gerr)) != sorted(set(merr)):
        only_g = sorted(set(gerr) - set(merr))
        only_m = sorted(set(merr) - set(gerr))
        problems.append("errors differ: gas-only %s emitter-only %s" % (only_g[:5], only_m[:5]))
    if gmodel and mmodel:
        for sec in ("text", "data"):
            if gmodel[sec] != mmodel[sec]:
                problems.append(".%s bytes: %s" % (sec, describe_diff(gmodel[sec], mmodel[sec])))
            if gmodel["align"].get(sec, 1) != mmodel["align"].get(sec, 1):
                problems.append(".%s align gas %s emitter %s" % (sec, gmodel["align"].get(sec), mmodel["align"].get(sec)))
        if gmodel["relocs"] != mmodel["relocs"]:
            g, m = set(gmodel["relocs"]), set(mmodel["relocs"])
            problems.append("relocs: gas-only %s emitter-only %s" % (sorted(g - m)[:4], sorted(m - g)[:4]))
        if gmodel["syms"] != mmodel["syms"]:
            g, m = gmodel["syms"], mmodel["syms"]
            problems.append("symbols: gas-only %s emitter-only %s" % (sorted(g - m)[:4], sorted(m - g)[:4]))
    return problems


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=os.path.join(ROOT, "target", "emitter-diff"))
    ap.add_argument("--quick", action="store_true")
    ap.add_argument("--no-gen", action="store_true", help="reuse the existing corpus")
    ap.add_argument("--show", type=int, default=40)
    args = ap.parse_args()
    corpus = os.path.join(args.out, "corpus")
    stats = {}
    if not args.no_gen:
        shutil.rmtree(args.out, ignore_errors=True)
        os.makedirs(corpus)
        gen_ref(corpus, args.quick, stats)
        stats["matrix_lines"] = gen_matrix(corpus, args.quick)
        stats["edge_files"] = gen_branches(corpus)
    else:
        for f in glob.glob(os.path.join(corpus, "*.ok*.s")) + glob.glob(os.path.join(corpus, "*.emit")):
            os.remove(f)
    results = gas_phase(corpus)
    run_emitter(corpus)
    failures = []
    n_err = n_bytes = 0
    for path, gas in sorted(results.items()):
        with open(path[:-2] + ".emit") as f:
            mine = f.read()
        probs = compare(path, gas, mine)
        if gas[1] is not None:
            n_bytes += 1
        else:
            n_err += 1
        if probs:
            failures.append((path, probs))
    total_lines = 0
    for path in results:
        if ".ok" not in path:
            with open(path, errors="replace") as f:
                total_lines += f.read().count("\n")
    for k, v in stats.items():
        print("%s: %s" % (k, v))
    print("files compared: %d (%d byte-compared, %d error-compared), source lines: %d" % (
        len(results), n_bytes, n_err, total_lines))
    for path, probs in failures[:args.show]:
        print("FAIL", os.path.relpath(path, corpus))
        for p in probs:
            print("    " + p)
    print("%d/%d files match" % (len(results) - len(failures), len(results)))
    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    main()
