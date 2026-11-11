#!/usr/bin/env python3
"""End-to-end behavioural test suite: new MemeAssembly compiler vs the original C compiler.

Usage:  tests/e2e/run.sh [options] [FILTER ...]     (FILTER = substring/regex of check ids)

Environment:
  NEW       new compiler (default: target/release/memeasm)
  NEW_WIN   compiler to use for Windows targets instead of "$NEW -ftarget=windows"
            (auto-set to REF_WIN when NEW is the reference -> sanity mode)
  REF       reference Linux compiler   (default /opt/memeasm-ref/memeasm-ref)
  REF_WIN   reference Windows compiler (default /opt/memeasm-ref/memeasm-ref-win)

Reference outcomes are computed live when the reference compilers exist (and can be stored with
--update-expected), otherwise they are read from tests/e2e/expected/.  See README-like notes at the
bottom of this file (STATUS meanings) and tests/e2e/gen_programs.py for the corpus.
"""
import argparse
import ctypes
import datetime
import difflib
import hashlib
import json
import os
import re
import select
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
import traceback
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
EXPECTED = HERE / "expected"
OUTDIR = HERE / "out"
MANIFEST = HERE / "programs" / "manifest.json"

QEMU_PREFIX = "/usr/x86_64-linux-gnu"
CC_LIN, OBJDUMP_LIN, OBJCOPY_LIN = "x86_64-linux-gnu-gcc", "x86_64-linux-gnu-objdump", "x86_64-linux-gnu-objcopy"
CC_WIN, OBJDUMP_WIN = "x86_64-w64-mingw32-gcc", "x86_64-w64-mingw32-objdump"
FAKETIME_LIBS = ["/usr/lib/aarch64-linux-gnu/faketime/libfaketime.so.1",
                 "/usr/lib/x86_64-linux-gnu/faketime/libfaketime.so.1",
                 "/usr/lib/faketime/libfaketime.so.1"]
REF_TIME = "2024-01-01 00:00:00"        # frozen clock for the reference (srand(time(NULL)))
REF_TIME2 = "2024-06-15 12:34:56"       # second clock used to detect non-determinism
RANDOM_SAMPLES_ASM = 48
RANDOM_SAMPLES_EXE = 64
NEW_RANDOM_RUNS = 3
COMPILE_TIMEOUT = 180
STDOUT_CAP = 1 << 20
PREFIX_CMP = 1024

STATUSES = ["PASS", "FIXED", "WARN", "CHANGED", "SKIP", "OLDBUG", "FAIL", "ERROR"]
FAILING = {"FAIL", "ERROR", "OLDBUG"}


class Cfg:
    pass


CFG = Cfg()
HEAVY = threading.Semaphore(1)


def sh_quote(argv):
    return " ".join(a if re.fullmatch(r"[\w./=:,+-]+", a) else repr(a) for a in argv)


# ------------------------------------------------------------------------------------------------
# process helpers
# ------------------------------------------------------------------------------------------------
def _limits():
    import resource
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))


def base_env():
    env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "LANG": "C", "LC_ALL": "C", "TZ": "UTC",
           "QEMU_LD_PREFIX": QEMU_PREFIX, "HOME": os.environ.get("HOME", "/tmp")}
    if "TMPDIR" in os.environ:
        env["TMPDIR"] = os.environ["TMPDIR"]
    return env


def run_proc(argv, cwd=None, env=None, timeout=60, stdin=None):
    """Run a (compile/tool) command, return dict rc/stdout/stderr (bytes) or rc='timeout'."""
    try:
        p = subprocess.run(argv, cwd=cwd, env=env or base_env(), timeout=timeout, capture_output=True,
                           stdin=subprocess.DEVNULL if stdin is None else stdin, preexec_fn=_limits)
        return {"rc": p.returncode, "stdout": p.stdout, "stderr": p.stderr}
    except subprocess.TimeoutExpired as e:
        return {"rc": "timeout", "stdout": e.stdout or b"", "stderr": e.stderr or b""}
    except OSError as e:
        return {"rc": "oserror", "stdout": b"", "stderr": str(e).encode()}


def status_of(rc):
    if rc is None:
        return "nonterm"
    return f"exit:{rc}" if rc >= 0 else f"signal:{-rc}"


def run_exe(argv, cwd, stdin_path=None, timeout=5.0):
    """Run a test program (x86-64 via qemu binfmt).  Captures at most STDOUT_CAP bytes of stdout;
    non-terminating programs (timeout / unbounded output) get status 'nonterm'."""
    stdin = open(stdin_path, "rb") if stdin_path else subprocess.DEVNULL
    try:
        p = subprocess.Popen(argv, cwd=cwd, env=base_env(), stdin=stdin, stdout=subprocess.PIPE,
                             stderr=subprocess.DEVNULL, preexec_fn=_limits, start_new_session=True)
    except OSError as e:
        return {"status": "oserror", "stdout": str(e)}
    finally:
        if stdin_path:
            stdin.close()
    data = bytearray()
    deadline = time.monotonic() + timeout
    nonterm = False
    fd = p.stdout.fileno()
    while True:
        left = deadline - time.monotonic()
        if left <= 0 or len(data) >= STDOUT_CAP:
            nonterm = True
            break
        r, _, _ = select.select([fd], [], [], left)
        if not r:
            continue
        chunk = os.read(fd, 65536)
        if not chunk:
            break
        data += chunk
    if nonterm:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except OSError:
            pass
    try:
        rc = p.wait(timeout=max(1.0, deadline - time.monotonic() + 1.0))
    except subprocess.TimeoutExpired:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except OSError:
            pass
        p.wait()
        nonterm = True
    p.stdout.close()
    out = bytes(data)
    if nonterm:
        return {"status": "nonterm", "stdout": out[:4 * PREFIX_CMP].decode("latin-1")}
    return {"status": status_of(rc), "stdout": out.decode("latin-1")}


# ------------------------------------------------------------------------------------------------
# compilers
# ------------------------------------------------------------------------------------------------
def compiler_argv(side, target):
    if side == "ref":
        return [CFG.ref_win] if target == "win" else [CFG.ref]
    if target == "win":
        return [CFG.new_win] if CFG.new_win else [CFG.new, "-ftarget=windows"]
    return [CFG.new]


def faketime_env(env, when):
    if CFG.faketime:
        env["LD_PRELOAD"] = CFG.faketime
        env["FAKETIME"] = when
        env["FAKETIME_DONT_FAKE_MONOTONIC"] = "1"
    return env


RANDOM_DEBUG = [
    (re.compile(r"(Chose random line for jump marker: )\d+"), r"\1<N>"),
    (re.compile(r"(Generated random line )\d+( in file )\d+"), r"\1<N>\2<N>"),
]
RANDOM_DEBUG_DROP = re.compile(r"^.*(Generated random line <N>|Line was already generated|Line was added to the list).*\n?",
                               re.M)


def norm_text(b, out=None, argv0=None):
    t = b.decode("latin-1") if isinstance(b, bytes) else b
    if out:
        t = t.replace(out, "<OUT>")
    for a in filter(None, set([argv0, os.path.realpath(argv0) if argv0 else None, CFG.ref, CFG.ref_win,
                               CFG.new, CFG.new_win])):
        t = t.replace(a, "<MEMEASM>")
    t = re.sub(r"/opt/memeasm-ref/[\w.-]+", "<MEMEASM>", t)
    t = re.sub(r"/tmp/cc\w+\.(o|s)", "/tmp/<TMP>", t)
    for rx, repl in RANDOM_DEBUG:
        t = rx.sub(repl, t)
    t = RANDOM_DEBUG_DROP.sub("", t)
    return t


def classify(c):
    if c["rc"] == 0:
        return "ok"
    if c["rc"] in ("timeout", "oserror"):
        return c["rc"]
    if isinstance(c["rc"], int) and c["rc"] < 0:
        return "crash"  # the compiler itself died from a signal
    if "Compilation failed with" in c["stderr"]:
        return "diag"
    return "toolchain"


def compile_(side, target, flags, entry, out, mode=None, when=REF_TIME, files=None, extra_env=None, timeout=None):
    """Compile entry (or explicit files) with flags. mode: None (executable), '-S' or '-O'."""
    argv = compiler_argv(side, target) + list(flags) + ([mode] if mode else []) + ["-o", out]
    argv += files if files is not None else entry.get("argfiles") or entry["files"]
    cwd = str(ROOT / entry["cwd"]) if entry and entry.get("cwd") and files is None else str(ROOT)
    env = base_env()
    if side == "ref":
        faketime_env(env, when)
    if extra_env:
        env.update(extra_env)
    r = run_proc(argv, cwd=cwd, env=env, timeout=timeout or (entry or {}).get("compile_timeout") or COMPILE_TIMEOUT)
    c = {"rc": r["rc"], "stdout": norm_text(r["stdout"], out, argv[0]), "stderr": norm_text(r["stderr"], out, argv[0])}
    c["class"] = classify(c)
    c["argv"] = sh_quote(["memeasm"] + argv[len(compiler_argv(side, target)):]).replace(out, "<OUT>")
    return c


def cmp_compile(r, n):
    """Returns None if equal, else a description."""
    if r["class"] == "toolchain" or n["class"] == "toolchain":
        # assembler/linker errors: the reference prints gas messages, the new compiler its own
        if r["class"] != n["class"] or r["rc"] != n["rc"] or r["stdout"] != n["stdout"]:
            return "compile result differs (toolchain-error class)"
        return None
    diffs = [k for k in ("rc", "stdout", "stderr") if r[k] != n[k]]
    return f"compile {'/'.join(diffs)} differ" if diffs else None


def fmt_compile(label, c):
    s = f"--- {label} compile: rc={c['rc']} class={c['class']}  ({c.get('argv', '')})\n"
    if c["stdout"]:
        s += f"[stdout]\n{c['stdout']}" + ("" if c["stdout"].endswith("\n") else "\n")
    if c["stderr"]:
        s += f"[stderr]\n{c['stderr']}" + ("" if c["stderr"].endswith("\n") else "\n")
    return s


def text_diff(a, b, la="reference", lb="new", n=80):
    d = list(difflib.unified_diff(a.splitlines(), b.splitlines(), la, lb, lineterm="", n=2))
    if len(d) > n:
        d = d[:n] + [f"... ({len(d) - n} more diff lines)"]
    return "\n".join(d) + "\n"


# ------------------------------------------------------------------------------------------------
# normalisation of compiler artefacts
# ------------------------------------------------------------------------------------------------
def norm_asm(t, strip_labels=False):
    t = re.sub(r"^# Generated by the MemeAssembly compiler (v[\d.]+) on .*$", r"# Generated by the MemeAssembly compiler \1 on <DATE>",
               t, flags=re.M)
    used = set(re.findall(r"jmp \.LConfusedStonks_(\d+)", t))
    def lab(m):
        return m.group(0) if (m.group(1) in used and not strip_labels) else ""
    t = re.sub(r"^[ \t]*\.LConfusedStonks_(\d+):[ \t]*\n", lab, t, flags=re.M)
    return t


def objdump_info(objdump, path, sections=(".text",)):
    d = Path(path).parent
    name = Path(path).name
    dis = run_proc([objdump, "-d", "-r", "-w"] + sum([["-j", s] for s in sections], []) + [name], cwd=d)
    data = run_proc([objdump, "-s", "-j", ".data", name], cwd=d)
    syms = run_proc([objdump, "-t", name], cwd=d)
    dtext = "\n".join(l for l in dis["stdout"].decode("latin-1").splitlines()
                      if name not in l and l.strip())
    datatext = "\n".join(l for l in data["stdout"].decode("latin-1").splitlines()
                         if name not in l and l.strip() and "Contents of section" not in l)
    glob = []
    for l in syms["stdout"].decode("latin-1").splitlines():
        m = re.match(r"^[0-9a-f]+ (.{7}) (\S+)\s+(?:[0-9a-f]+\s+)?(\S+)$", l)
        if not m:
            continue
        flags, sect, sym = m.groups()
        if "g" in flags or sect == "*UND*":
            glob.append(f"{sym} {'UND' if sect == '*UND*' else 'DEF'}")
    return {"dis": dtext, "data": datatext, "globals": sorted(set(glob))}


INSN_RX = re.compile(r"^\s*[0-9a-f]+:\t(?:[0-9a-f]{2} ?)+\s*\t(.*)$")


def norm_dis(t):
    """Instruction text only: no addresses/encodings/symbol annotations, no padding nops."""
    out = []
    for l in t.splitlines():
        m = INSN_RX.match(l)
        if m:
            ins = m.group(1)
            ins = re.sub(r"\s+#.*$", "", ins)
            ins = re.sub(r"\b[0-9a-f]+ <[^>]*>", "<T>", ins)
            ins = re.sub(r"<[^>]*>", "", ins)
            ins = re.sub(r"\s+", " ", ins).strip()
            if re.match(r"^(nop|xchg ax,ax|data16|cs nop|int3$)", ins) or not ins:
                continue
            out.append(ins)
        elif re.match(r"^\s*[0-9a-f]+: R_", l):
            r = l.split(":", 1)[1].split()
            out.append(r[0] + " " + re.sub(r"[-+]0x[0-9a-f]+$", "", r[1] if len(r) > 1 else ""))
    return "\n".join(out)


# ------------------------------------------------------------------------------------------------
# reference outcome store (live or expected/)
# ------------------------------------------------------------------------------------------------
def exp_path(cid):
    return EXPECTED / (re.sub(r"[^\w./-]", "_", cid.replace(":", "/")) + ".json")


def get_ref(cid, producer):
    if CFG.live:
        v = producer()
        if CFG.update:
            p = exp_path(cid)
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(json.dumps(v, indent=1, sort_keys=True) + "\n")
        return v
    p = exp_path(cid)
    if not p.exists():
        return None
    return json.loads(p.read_text())


class Result:
    def __init__(self, status, summary="", details=""):
        self.status, self.summary, self.details = status, summary, details


# ------------------------------------------------------------------------------------------------
# work directories
# ------------------------------------------------------------------------------------------------
_wd_lock = threading.Lock()
_wd_n = [0]


def workdir(cid, side):
    with _wd_lock:
        _wd_n[0] += 1
        n = _wd_n[0]
    d = Path(CFG.work) / f"{n:05d}_{re.sub(r'[^\w]+', '_', cid)[-60:]}" / side
    d.mkdir(parents=True, exist_ok=True)
    return d


def runs_of(entry):
    if "runs" in entry:
        return entry["runs"]
    r = {}
    if entry.get("stdin"):
        r["stdin"] = entry["stdin"]
    return [r]


def heavy(entry, flags):
    return entry.get("heavy") or any(f in ("-O-2", "-O-s") for f in flags)


def build_and_run(side, entry, flags, cid, target="lin", files=None, when=REF_TIME, keep=False):
    """Compile an executable and run it for each run spec. Returns {"compile":..., "runs":[...]}."""
    d = workdir(cid, side)
    exe = str(d / ("prog.exe" if target == "win" else "prog"))
    lock = HEAVY if heavy(entry, flags) else None
    if lock:
        lock.acquire()
    try:
        c = compile_(side, target, flags, entry, exe, when=when, files=files, timeout=600 if lock else None)
        res = {"compile": c, "runs": None}
        if c["rc"] == 0 and os.path.exists(exe):
            res["runs"] = [execute(exe, entry, r, d, target) for r in runs_of(entry)]
        elif c["rc"] == 0:
            res["runs"] = "no output file"
        if not keep:
            shutil.rmtree(d, ignore_errors=True)
        else:
            res["_exe"] = exe
        return res
    finally:
        if lock:
            lock.release()


def execute(exe, entry, r, d, target="lin"):
    stdin = str(ROOT / r["stdin"]) if r.get("stdin") else None
    if target == "win":
        argv = [CFG.wine_run, exe] + r.get("args", [])
        res = run_exe(argv, d, stdin, timeout=max(30.0, entry.get("timeout", 5) * 6))
        noise = re.compile(r"^.*(\[\x1b\[31mBOX64\x1b\[0m\]|\[BOX64\]|winedbg|wine: ).*\n?", re.M)
        if noise.search(res["stdout"]):
            res["crash"] = True
            res["stdout"] = noise.sub("", res["stdout"])
        return res
    return run_exe([exe] + r.get("args", []), d, stdin, timeout=entry.get("timeout", 5))


def cmp_runs(rr, nr, exit_unspec=False):
    """Compare two run lists; returns list of problems."""
    probs = []
    if isinstance(rr, str) or isinstance(nr, str) or rr is None or nr is None:
        if rr != nr:
            probs.append(f"runs: reference={rr!r:.60} new={nr!r:.60}")
        return probs
    for i, (a, b) in enumerate(zip(rr, nr)):
        if a["status"] == "nonterm" or b["status"] == "nonterm":
            if a["status"] != b["status"]:
                probs.append(f"run {i}: status reference={a['status']} new={b['status']}")
            k = min(len(a["stdout"]), len(b["stdout"]), PREFIX_CMP)
            if a["stdout"][:k] != b["stdout"][:k]:
                probs.append(f"run {i}: stdout prefix differs")
            continue
        sa, sb = a["status"], b["status"]
        if exit_unspec and sa.startswith("exit:") and sb.startswith("exit:"):
            sa = sb
        if sa != sb:
            probs.append(f"run {i}: status reference={a['status']} new={b['status']}")
        if a["stdout"] != b["stdout"]:
            probs.append(f"run {i}: stdout differs")
    return probs


def fmt_runs(label, runs, entry):
    if not isinstance(runs, list):
        return f"--- {label} runs: {runs!r}\n"
    s = ""
    for i, (spec, r) in enumerate(zip(runs_of(entry), runs)):
        s += f"--- {label} run {i} {json.dumps(spec)}: {r['status']}\n{r['stdout'][:3000]!r}\n"
    return s


def outcome_matches(r, n, entry):
    if r["compile"]["class"] == "toolchain" or n["compile"]["class"] == "toolchain":
        return cmp_compile(r["compile"], n["compile"]) is None
    return cmp_compile(r["compile"], n["compile"]) is None and not cmp_runs(r["runs"], n["runs"], entry.get("exit_code_unspecified"))


# ------------------------------------------------------------------------------------------------
# check kinds.  Each returns (ref_producer, new_producer, comparer) via a function run_kind_*.
# comparer(ref, new) -> Result ;  ref may contain extra data for bug evaluation (oracle)
# ------------------------------------------------------------------------------------------------
def bug_spec(entry):
    return entry.get("ref_bug")


def spec_needs_oracle(spec):
    if spec["kind"] == "runs":
        return True
    return any(spec_needs_oracle(s) for s in spec.get("of", []))


def find_oracle_spec(spec):
    if spec["kind"] == "runs":
        return spec
    for s in spec.get("of", []):
        o = find_oracle_spec(s)
        if o:
            return o
    return None


def ref_oracle(entry, flags, cid):
    rb = bug_spec(entry)
    if not rb or not spec_needs_oracle(rb["correct"]):
        return None
    sp = find_oracle_spec(rb["correct"])
    files = [sp["oracle"]] if sp.get("oracle") else None
    oflags = sp["oracle_flags"] if "oracle_flags" in sp else [f for f in flags]
    return build_and_run("ref", entry, oflags, cid + ":oracle", files=files)


def eval_spec(spec, kind, new, ref, entry):
    """True if the new outcome satisfies the 'correct behaviour' spec, None if not evaluable."""
    k = spec["kind"]
    comp = new.get("compile") if isinstance(new, dict) else None
    if k == "any_of":
        vals = [eval_spec(s, kind, new, ref, entry) for s in spec["of"]]
        if any(v is True for v in vals):
            return True
        return False if any(v is False for v in vals) else None
    if k == "unspecified":
        return None
    if k == "no_toolchain_error":
        return comp is not None and comp["class"] in ("ok", "diag")
    if k == "diagnostic":
        return comp is not None and comp["class"] == "diag"
    if k == "terminates":  # the compiler neither crashes nor hangs (output may legitimately vary/fail to link)
        return comp is not None and comp["class"] in ("ok", "diag", "toolchain")
    if k == "compile_ok":
        return comp is not None and comp["rc"] == 0
    if k == "runs":
        if kind != "lin-exe" or not ref.get("oracle"):
            return None
        o = ref["oracle"]
        return comp is not None and comp["rc"] == 0 and not cmp_runs(o["runs"], new["runs"], entry.get("exit_code_unspecified"))
    if k == "linux_runs":
        if kind != "win-run" or not isinstance((ref.get("linux") or {}).get("runs"), list):
            return None
        lin = [dict(x, status=f"exit:{int(x['status'][5:]) & 0xff}") if x["status"].startswith("exit:") else x
               for x in ref["linux"]["runs"]]
        return comp is not None and comp["rc"] == 0 and not cmp_runs(lin, new["runs"])
    if k == "asm":
        if kind not in ("lin-asm", "win-asm"):
            return None
        if comp is None or comp["rc"] != 0:
            return False
        a = new.get("asm", "")
        return all(re.search(p, a) for p in spec.get("present", [])) and not any(re.search(p, a) for p in spec.get("absent", []))
    if k == "martyrdom":
        if kind != "martyrdom":
            return None
        if spec["expect"] == "no-martyrdom":
            return new.get("sh") == "exit:0" and "after 130" in new.get("stdout", "")
    return None


def apply_bug(entry, kind, ref, new, res):
    """Reclassify a normal comparison result for entries that trigger a reference bug."""
    rb = bug_spec(entry)
    if not rb or res.status in ("SKIP", "ERROR"):
        return res
    tag = f"[reference bug: {rb['desc']}]"
    in_bug = kind in rb["bug_kinds"]
    if res.status in ("PASS", "WARN"):
        if in_bug and rb["correct"]["kind"] == "unspecified":
            return Result(res.status, "same as reference (reference behaviour is undefined here)", tag + "\n" + res.details)
        if in_bug:
            st = "PASS" if CFG.allow_old_bugs else "OLDBUG"
            return Result(st, "new compiler reproduces the reference bug" + (" (allowed)" if CFG.allow_old_bugs else ""),
                          tag + "\n" + res.details)
        return res
    if not in_bug:
        return Result("CHANGED", "differs from buggy reference on a non-bug aspect: " + res.summary, tag + "\n" + res.details)
    v = eval_spec(rb["correct"], kind, new, ref, entry)
    if v is True:
        return Result("FIXED", "reference bug fixed: behaves as specified", tag + "\n" + res.details)
    if v is None:
        return Result("CHANGED", "differs from buggy reference (correct behaviour unspecified): " + res.summary,
                      tag + "\n" + res.details)
    return Result("FAIL", "differs from reference but not the specified correct behaviour: " + res.summary,
                  tag + f"\ncorrect = {json.dumps(rb['correct'])}\n" + res.details)


def generic(cid, entry, kind, ref_fn, new_fn, cmp_fn):
    ref = get_ref(cid, ref_fn)
    if ref is None:
        return Result("SKIP", "no stored reference outcome (run with the reference installed / --update-expected)")
    new = new_fn()
    res = cmp_fn(ref, new)
    return apply_bug(entry, kind, ref, new, res)


def k_exe(cid, entry, flags, target="lin"):
    kind = "lin-exe" if target == "lin" else "win-run"

    def ref_fn():
        r = build_and_run("ref", entry, flags, cid, target)
        if target == "lin":
            o = ref_oracle(entry, flags, cid)
            if o:
                r["oracle"] = o
        else:
            # Linux behaviour of the same program: the oracle for reference-Windows bugs
            r["linux"] = build_and_run("ref", entry, flags, cid + ":linux", "lin")
        return r

    def cmp(ref, new):
        det = fmt_compile("reference", ref["compile"]) + fmt_compile("new", new["compile"])
        p = cmp_compile(ref["compile"], new["compile"])
        probs = [p] if p else []
        if not p and ref["compile"]["class"] == "ok":
            probs += cmp_runs(ref["runs"], new["runs"], entry.get("exit_code_unspecified"))
        det += fmt_runs("reference", ref["runs"], entry) + fmt_runs("new", new["runs"], entry)
        if ref.get("oracle"):
            det += fmt_runs("oracle (correct behaviour)", ref["oracle"]["runs"], entry)
        if target == "win" and isinstance(ref.get("runs"), list):
            lin = ref.get("linux") or {}
            lruns = lin.get("runs")
            win_bug = isinstance(lruns, list) and any(w.get("crash") and l["status"] in ("exit:0", "nonterm")
                                                      for w, l in zip(ref["runs"], lruns))
            if win_bug and entry.get("win_program_bug"):
                # the program itself is not Windows-compatible (e.g. reads argv from the Linux register): any crash
                # or failure is acceptable for both compilers, as long as the new build does not succeed differently
                det = f"[program bug on Windows: {entry['win_program_bug']}]\n" + det + fmt_runs("reference (Linux)", lruns, entry)
                nruns = new.get("runs") if isinstance(new.get("runs"), list) else []
                if not probs or (nruns and all(r.get("crash") or r["status"] != "exit:0" for r in nruns)):
                    return Result("PASS", "program is not Windows-compatible; crashes/fails with both compilers", det)
                return Result("CHANGED", "program is not Windows-compatible, but the new build behaves differently: "
                              + "; ".join(probs), det)
            if win_bug:
                det = WIN_BUG + "\n" + det + fmt_runs("reference (Linux)", lruns, entry)
                if not probs:
                    return Result("PASS" if CFG.allow_old_bugs else "OLDBUG",
                                  "reproduces the reference Windows crash" + (" (allowed)" if CFG.allow_old_bugs else ""), det)
                lin_n = [dict(x, status=f"exit:{int(x['status'][5:]) & 0xff}") if x["status"].startswith("exit:") else x
                         for x in lruns]
                if isinstance(new.get("runs"), list) and not cmp_runs(lin_n, new["runs"], entry.get("exit_code_unspecified")):
                    return Result("FIXED", "reference Windows build crashes; new behaves like the Linux build", det)
                return Result("FAIL", "reference Windows build crashes and new differs from the Linux behaviour: "
                              + "; ".join(probs), det)
        if probs:
            return Result("FAIL", "; ".join(probs), det)
        return Result("PASS", "", det)
    return generic(cid, entry, kind, ref_fn, lambda: build_and_run("new", entry, flags, cid, target), cmp)


WIN_BUG = ("[reference bug (Windows): writechar/readchar store WriteFile/ReadFile's 5th argument with "
           "'mov QWORD PTR [rsp + 32], 0' after 'sub rsp, 32', which overwrites the saved r9 (r9 becomes 0 after every "
           "print/read), and r10/r11 (volatile in the Win64 ABI) are not preserved]")


def asm_build(side, entry, flags, cid, target, when=REF_TIME, files=None, strip=False):
    d = workdir(cid, side)
    out = str(d / "out.S")
    c = compile_(side, target, flags, entry, out, mode="-S", when=when, files=files)
    r = {"compile": c}
    if c["rc"] == 0:
        try:
            r["asm"] = norm_asm(Path(out).read_text(encoding="latin-1"), strip)
        except OSError:
            r["asm"] = None
    shutil.rmtree(d, ignore_errors=True)
    return r


def k_asm(cid, entry, flags, target="lin"):
    kind = "lin-asm" if target == "lin" else "win-asm"
    debug = "-g" in flags

    def ref_fn():
        r = asm_build("ref", entry, flags, cid, target)
        if CFG.faketime and not debug:
            r2 = asm_build("ref", entry, flags, cid, target, when=REF_TIME2)
            r["nondet"] = r2 != r
        return r

    def cmp(ref, new):
        det = fmt_compile("reference", ref["compile"]) + fmt_compile("new", new["compile"])
        if ref.get("nondet") and not entry.get("random"):
            return Result("FAIL", "reference output is non-deterministic but the manifest does not mark the program random", det)
        p = cmp_compile(ref["compile"], new["compile"])
        if p:
            return Result("FAIL", p, det)
        if ref["compile"]["rc"] != 0:
            return Result("PASS", "", det)
        if debug:
            if not CFG.new_is_ref and target == "lin" and new.get("asm") and not re.search(r"^\s*\.(loc|file)\b", new["asm"], re.M):
                return Result("WARN", "-g: no DWARF .file/.loc directives in -S output", det)
            return Result("PASS", "-g: assembly text not compared (STABS vs DWARF)", det)
        if ref.get("asm") != new.get("asm"):
            return Result("FAIL", "assembly differs", det + text_diff(ref.get("asm") or "", new.get("asm") or ""))
        return Result("PASS", "", det)
    return generic(cid, entry, kind, ref_fn, lambda: asm_build("new", entry, flags, cid, target), cmp)


def obj_build(side, entry, flags, cid, target, link=None):
    d = workdir(cid, side)
    out = str(d / "out.o")
    c = compile_(side, target, flags, entry, out, mode="-O")
    r = {"compile": c}
    if c["rc"] == 0:
        if not os.path.exists(out):
            r["obj"] = "no output file"
        else:
            r["obj"] = objdump_info(OBJDUMP_WIN if target == "win" else OBJDUMP_LIN, out)
            if target == "win":
                # link the COFF object with mingw gcc (with the driver if there is one)
                srcs = [str(ROOT / entry["driver"])] if entry.get("driver") else []
                l = run_proc([CC_WIN, "-o", str(d / "linked.exe")] + srcs + [out], cwd=d)
                r["link_rc"] = l["rc"]
                r["link_err"] = l["stderr"].decode("latin-1")[-2000:]
    shutil.rmtree(d, ignore_errors=True)
    return r


def k_obj(cid, entry, flags, target="lin"):
    kind = "lin-obj" if target == "lin" else "win-obj"

    def cmp(ref, new):
        det = fmt_compile("reference", ref["compile"]) + fmt_compile("new", new["compile"])
        p = cmp_compile(ref["compile"], new["compile"])
        if p:
            return Result("FAIL", p, det)
        if ref["compile"]["rc"] != 0:
            return Result("PASS", "", det)
        ro, no = ref.get("obj"), new.get("obj")
        if not isinstance(no, dict) or not isinstance(ro, dict):
            return Result("FAIL", f"object: {no!r:.80}", det)
        probs = []
        if ro["globals"] != no["globals"]:
            probs.append("global/undefined symbols differ")
            det += text_diff("\n".join(ro["globals"]), "\n".join(no["globals"]))
        if ro["data"] != no["data"]:
            probs.append(".data differs")
            det += text_diff(ro["data"], no["data"])
        if "link_rc" in ref and (ref["link_rc"] == 0) != (new.get("link_rc") == 0):
            probs.append(f"mingw link rc reference={ref['link_rc']} new={new.get('link_rc')}")
            det += "--- new link stderr:\n" + new.get("link_err", "") + "\n"
        if ro["dis"] == no["dis"] and not probs:
            return Result("PASS", "", det)
        if norm_dis(ro["dis"]) == norm_dis(no["dis"]):
            if probs:
                return Result("FAIL", "; ".join(probs), det)
            return Result("WARN", "disassembly equal modulo encodings/addresses/symbol names", det + text_diff(ro["dis"], no["dis"]))
        probs.append("instructions differ")
        return Result("FAIL", "; ".join(probs), det + text_diff(norm_dis(ro["dis"]), norm_dis(no["dis"]))
                      + "\n(raw)\n" + text_diff(ro["dis"], no["dis"], n=40))
    return generic(cid, entry, kind, lambda: obj_build("ref", entry, flags, cid, target),
                   lambda: obj_build("new", entry, flags, cid, target), cmp)


def link_build(side, entry, flags, cid):
    """-O object + C driver, linked as in the example Makefiles (gcc -o X driver.c obj.o -no-pie)."""
    d = workdir(cid, side)
    out = str(d / "out.o")
    c = compile_(side, "lin", flags, entry, out, mode="-O")
    r = {"compile": c, "runs": None}
    if c["rc"] == 0:
        l = run_proc([CC_LIN, "-o", str(d / "prog"), str(ROOT / entry["driver"]), out, "-no-pie"], cwd=d)
        r["link_rc"] = l["rc"]
        r["link_err"] = l["stderr"].decode("latin-1")[-2000:]
        if l["rc"] == 0:
            r["runs"] = [execute(str(d / "prog"), entry, {}, d)]
    shutil.rmtree(d, ignore_errors=True)
    return r


def k_link(cid, entry, flags):
    def cmp(ref, new):
        det = fmt_compile("reference", ref["compile"]) + fmt_compile("new", new["compile"])
        p = cmp_compile(ref["compile"], new["compile"])
        if p:
            return Result("FAIL", p, det)
        if ref.get("link_rc") != new.get("link_rc"):
            return Result("FAIL", f"link rc reference={ref.get('link_rc')} new={new.get('link_rc')}",
                          det + "--- new link stderr\n" + new.get("link_err", ""))
        det += fmt_runs("reference", ref["runs"], {}) + fmt_runs("new", new["runs"], {})
        probs = cmp_runs(ref["runs"], new["runs"])
        return Result("FAIL" if probs else "PASS", "; ".join(probs), det)
    return generic(cid, entry, "lin-link", lambda: link_build("ref", entry, flags, cid),
                   lambda: link_build("new", entry, flags, cid), cmp)


# ------------------------------------------------------------------------------------------------
# Windows executables: PE structure and imports
# ------------------------------------------------------------------------------------------------
def pe_info(path):
    p = run_proc([OBJDUMP_WIN, "-p", "-h", path])
    t = p["stdout"].decode("latin-1")
    info = {"objdump_rc": p["rc"]}
    m = re.search(r"^Magic\s+([0-9a-f]+)", t, re.M)
    info["magic"] = m.group(1) if m else None
    m = re.search(r"^Subsystem\s+([0-9a-f]+)", t, re.M)
    info["subsystem"] = int(m.group(1), 16) if m else None
    m = re.search(r"^AddressOfEntryPoint\s+([0-9a-f]+)", t, re.M)
    ep = int(m.group(1), 16) if m else None
    m = re.search(r"^ImageBase\s+([0-9a-f]+)", t, re.M)
    base = int(m.group(1), 16) if m else 0
    m = re.search(r"file format (\S+)", t)
    info["format"] = m.group(1) if m else None
    imports, dll = {}, None
    for l in t.splitlines():
        m = re.match(r"^\s*DLL Name: (\S+)", l)
        if m:
            dll = m.group(1).lower()
            imports.setdefault(dll, [])
            continue
        m = re.match(r"^\s+[0-9a-f]+\s+(?:<none>|[0-9a-f]+)\s+[0-9a-f]+\s+(\w+)\s*$", l) if dll else None
        if m:
            imports[dll].append(m.group(1))
        elif dll and not l.strip():
            pass
    info["imports"] = {k: sorted(set(v)) for k, v in imports.items()}
    # entry point must be inside a CODE section
    info["entry_in_code"] = False
    for m in re.finditer(r"^\s*\d+\s+(\S+)\s+([0-9a-f]+)\s+([0-9a-f]+)\s+[0-9a-f]+\s+[0-9a-f]+\s+\S+\n\s+(.*)$", t, re.M):
        name, size, vma, flags = m.group(1), int(m.group(2), 16), int(m.group(3), 16), m.group(4)
        if ep is not None and vma <= base + ep < vma + size and "CODE" in flags:
            info["entry_in_code"] = True
    return info


def pe_build(side, entry, flags, cid):
    d = workdir(cid, side)
    exe = str(d / "prog.exe")
    lock = HEAVY if heavy(entry, flags) else None
    if lock:
        lock.acquire()
    try:
        c = compile_(side, "win", flags, entry, exe)
        r = {"compile": c}
        if c["rc"] == 0 and os.path.exists(exe):
            with open(exe, "rb") as f:
                hdr = f.read(4096)
            r["mz"] = hdr[:2] == b"MZ"
            off = int.from_bytes(hdr[0x3c:0x40], "little") if len(hdr) > 0x40 else 0
            r["pe_sig"] = hdr[off:off + 4] == b"PE\0\0" if off < 4092 else False
            r["machine"] = int.from_bytes(hdr[off + 4:off + 6], "little") if off < 4090 else None
            r["pe"] = pe_info(exe)
        if side == "ref" and c["rc"] == 0:
            a = asm_build("ref", entry, flags, cid + ":asm", "win")
            r["externs"] = sorted(set(re.findall(r"^\.extern (\w+)", a.get("asm") or "", re.M)))
        return r
    finally:
        shutil.rmtree(d, ignore_errors=True)
        if lock:
            lock.release()


def k_win_exe(cid, entry, flags):
    def cmp(ref, new):
        det = fmt_compile("reference", ref["compile"]) + fmt_compile("new", new["compile"])
        p = cmp_compile(ref["compile"], new["compile"])
        if p:
            return Result("FAIL", p, det)
        if ref["compile"]["rc"] != 0:
            return Result("PASS", "", det)
        probs = []
        pe = new.get("pe") or {}
        det += f"--- reference PE: {json.dumps(ref.get('pe'))}\n--- new PE: {json.dumps(pe)}\n"
        if not new.get("mz") or not new.get("pe_sig"):
            probs.append("not a PE file (MZ/PE signature)")
        if new.get("machine") != 0x8664:
            probs.append(f"machine {new.get('machine')!r} != 0x8664")
        if pe.get("magic") != "020b":
            probs.append(f"optional header magic {pe.get('magic')} != 020b (PE32+)")
        if pe.get("subsystem") != 3:
            probs.append(f"subsystem {pe.get('subsystem')} != 3 (console)")
        if not pe.get("entry_in_code"):
            probs.append("entry point not inside a code section")
        k32 = set(pe.get("imports", {}).get("kernel32.dll", []))
        need = set(ref.get("externs", [])) & {"GetStdHandle", "WriteFile", "ReadFile"}
        if not need:
            need = {"GetStdHandle", "WriteFile", "ReadFile"}
        miss = need - k32
        if miss:
            probs.append(f"missing kernel32 imports: {sorted(miss)}")
        return Result("FAIL" if probs else "PASS", "; ".join(probs), det)
    return generic(cid, entry, "win-exe", lambda: pe_build("ref", entry, flags, cid),
                   lambda: pe_build("new", entry, flags, cid), cmp)


# ------------------------------------------------------------------------------------------------
# random programs (confused stonks / perfectly balanced): the reference seeds rand() with
# time(NULL); libfaketime freezes the clock so the reference's choices can be sampled.
# ------------------------------------------------------------------------------------------------
def sample_time(i):
    t = datetime.datetime(2024, 1, 1) + datetime.timedelta(seconds=i * 3607 + i * i)
    return t.strftime("%Y-%m-%d %H:%M:%S")


def has_pb(entry):
    return any("perfectly balanced as all things should be" in (ROOT / f).read_text(encoding="latin-1") for f in entry["files"])


def compile_key(c):
    return json.dumps({k: c[k] for k in ("rc", "stdout", "stderr", "class")}, sort_keys=True)


def is_subsequence(small, big):
    it = iter(big)
    return all(any(x == y for y in it) for x in small)


def k_random_asm(cid, entry, flags, target):
    kind = "lin-asm" if target == "lin" else "win-asm"
    pb = has_pb(entry)

    def ref_fn():
        comps, variants = {}, set()
        for i in range(RANDOM_SAMPLES_ASM):
            r = asm_build("ref", entry, flags, cid, target, when=sample_time(i), strip=True)
            comps[compile_key(r["compile"])] = r["compile"]
            variants.add(r.get("asm"))
        full = None
        if pb:
            d = workdir(cid, "full")
            files = []
            for j, f in enumerate(entry["files"]):
                src = (ROOT / f).read_text(encoding="latin-1")
                src = "\n".join("" if "perfectly balanced as all things should be" in l else l for l in src.split("\n"))
                p = d / str(j) / Path(f).name
                p.parent.mkdir(parents=True)
                p.write_text(src, encoding="latin-1")
                files.append(str(p))
            full = asm_build("ref", entry, flags, cid, target, files=files, strip=True).get("asm")
        return {"compiles": list(comps.values()), "variants": sorted(v for v in variants if v is not None), "full": full,
                "samples": RANDOM_SAMPLES_ASM}

    def cmp(ref, new):
        det = "".join(fmt_compile(f"reference#{i}", c) for i, c in enumerate(ref["compiles"])) + fmt_compile("new", new["compile"])
        if not any(cmp_compile(c, new["compile"]) is None for c in ref["compiles"]):
            return Result("FAIL", "compile result not among the reference's sampled results", det)
        if new["compile"]["rc"] != 0:
            return Result("PASS", "", det)
        a = new.get("asm") or ""
        if a in ref["variants"]:
            return Result("PASS", "random: matches a sampled reference variant (confused-stonks label ignored)", det)
        # blank lines are ignored: the deleted/"perfectly balanced" lines themselves leave empty '\t' lines behind
        nonblank = lambda t: [l for l in t.splitlines() if l.strip()]
        if ref.get("full") and is_subsequence(nonblank(a), nonblank(ref["full"])):
            return Result("PASS", "random: perfectly balanced output is a line-subsequence of the full program", det)
        best = min(ref["variants"], key=lambda v: len(text_diff(v, a, n=10 ** 6))) if ref["variants"] else ""
        return Result("FAIL", "random: assembly matches no sampled reference variant", det + text_diff(best, a))
    return generic(cid, entry, kind, ref_fn, lambda: asm_build("new", entry, flags, cid, target, strip=True), cmp)


def strip_outcome(o):
    o = dict(o)
    o["compile"] = {k: o["compile"][k] for k in ("rc", "stdout", "stderr", "class")}
    o.pop("_exe", None)
    return o


def k_random_exe(cid, entry, flags):
    pb = has_pb(entry)

    def ref_fn():
        outcomes, seen_asm, last_new = [], set(), 0
        for i in range(RANDOM_SAMPLES_EXE):
            when = sample_time(i)
            a = asm_build("ref", entry, flags, cid, "lin", when=when)
            key = hashlib.sha1(json.dumps(a, sort_keys=True).encode()).hexdigest()
            if key in seen_asm:
                continue
            seen_asm.add(key)
            last_new = i
            o = strip_outcome(build_and_run("ref", entry, flags, cid, when=when))
            if not any(json.dumps(o, sort_keys=True) == json.dumps(x, sort_keys=True) for x in outcomes):
                outcomes.append(o)
        return {"outcomes": outcomes, "variants": len(seen_asm), "samples": RANDOM_SAMPLES_EXE,
                "complete": RANDOM_SAMPLES_EXE - last_new > RANDOM_SAMPLES_EXE // 2}

    def new_fn():
        return {"outcomes": [strip_outcome(build_and_run("new", entry, flags, cid)) for _ in range(NEW_RANDOM_RUNS)]}

    def cmp(ref, new):
        det = f"reference: {len(ref['outcomes'])} distinct outcomes from {ref['variants']} distinct variants " \
              f"({ref['samples']} samples, saturated={ref['complete']})\n"
        bad = []
        for i, o in enumerate(new["outcomes"]):
            if not any(outcome_matches(r, o, entry) for r in ref["outcomes"]):
                bad.append(i)
                det += fmt_compile(f"new#{i}", o["compile"]) + fmt_runs(f"new#{i}", o["runs"], entry)
        if not bad:
            return Result("PASS", f"random: {len(new['outcomes'])} new outcomes all among the reference outcome set", det)
        for i, r in enumerate(ref["outcomes"][:12]):
            det += fmt_compile(f"reference#{i}", r["compile"]) + fmt_runs(f"reference#{i}", r["runs"], entry)
        if ref["complete"] and not pb:
            return Result("FAIL", f"random: {len(bad)} new outcome(s) not in the (saturated) reference outcome set", det)
        return Result("WARN", f"random: {len(bad)} new outcome(s) not observed in the sampled reference outcomes "
                              "(sample set not exhaustive)", det)
    return generic(cid, entry, "lin-exe", ref_fn, new_fn, cmp)


# ------------------------------------------------------------------------------------------------
# martyrdom: SIGINT to the program must SIGKILL its parent (sh) and exit 0
# ------------------------------------------------------------------------------------------------
def martyr_helper(exe, timeout):
    """Runs in a separate python process (child subreaper)."""
    libc = ctypes.CDLL(None, use_errno=True)
    libc.prctl(36, 1, 0, 0, 0)  # PR_SET_CHILD_SUBREAPER
    p = subprocess.Popen(["/bin/sh", "-c", '"$0"; echo "after $?"', exe], stdout=subprocess.PIPE,
                         stdin=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=base_env(), preexec_fn=_limits)
    fd = p.stdout.fileno()
    out = b""
    deadline = time.monotonic() + timeout
    while b"R\n" not in out and time.monotonic() < deadline:
        r, _, _ = select.select([fd], [], [], 0.1)
        if r:
            c = os.read(fd, 4096)
            if not c:
                break
            out += c
    res = {"saw_R": b"R\n" in out}
    child = None
    if res["saw_R"]:
        for _ in range(50):
            try:
                kids = open(f"/proc/{p.pid}/task/{p.pid}/children").read().split()
            except OSError:
                kids = []
            if kids:
                child = int(kids[0])
                break
            time.sleep(0.05)
        if child:
            time.sleep(0.2)
            os.kill(child, signal.SIGINT)
    try:
        rc = p.wait(timeout=5)
        res["sh"] = status_of(rc)
    except subprocess.TimeoutExpired:
        res["sh"] = "nonterm"
        p.kill()
        p.wait()
    end = time.monotonic() + 5
    while time.monotonic() < end:
        r, _, _ = select.select([fd], [], [], 0.1)
        if r:
            c = os.read(fd, 4096)
            if not c:
                break
            out += c
    res["stdout"] = out.decode("latin-1")
    res["child"] = None
    if child:
        end = time.monotonic() + 5
        while time.monotonic() < end:
            try:
                pid, st = os.waitpid(child, os.WNOHANG)
            except ChildProcessError:
                res["child"] = "reaped-by-sh"
                break
            if pid:
                res["child"] = status_of(os.waitstatus_to_exitcode(st))
                break
            time.sleep(0.05)
        else:
            os.kill(child, signal.SIGKILL)
            res["child"] = "nonterm"
    print(json.dumps(res))


def martyr_build(side, entry, flags, cid):
    d = workdir(cid, side)
    exe = str(d / "prog")
    c = compile_(side, "lin", flags, entry, exe)
    r = {"compile": c}
    if c["rc"] == 0:
        p = run_proc([sys.executable, "-I", str(Path(__file__).resolve()), "--martyr-helper", exe,
                      str(entry.get("timeout", 5))], timeout=60)
        try:
            r.update(json.loads(p["stdout"].decode()))
        except ValueError:
            r["helper_error"] = p["stderr"].decode("latin-1")[-1000:]
    shutil.rmtree(d, ignore_errors=True)
    return r


def k_martyrdom(cid, entry, flags):
    def cmp(ref, new):
        det = fmt_compile("reference", ref["compile"]) + fmt_compile("new", new["compile"])
        p = cmp_compile(ref["compile"], new["compile"])
        if p:
            return Result("FAIL", p, det)
        keys = ("saw_R", "sh", "stdout", "child")
        det += f"--- reference: {json.dumps({k: ref.get(k) for k in keys})}\n--- new: {json.dumps({k: new.get(k) for k in keys})}\n"
        # the child's own status is only observable if it was reparented to the helper (racy) -> compare when known
        known = lambda v: isinstance(v, str) and v.startswith(("exit:", "signal:"))
        diff = [k for k in keys if ref.get(k) != new.get(k) and (k != "child" or (known(ref.get(k)) and known(new.get(k))))]
        return Result("FAIL" if diff else "PASS", f"martyrdom behaviour differs ({', '.join(diff)})" if diff else "", det)
    return generic(cid, entry, "martyrdom", lambda: martyr_build("ref", entry, flags, cid),
                   lambda: martyr_build("new", entry, flags, cid), cmp)


# ------------------------------------------------------------------------------------------------
# command line handling (loose: clap generates usage/errors in the new compiler)
# ------------------------------------------------------------------------------------------------
P_OK = "tests/e2e/programs/cmd/nop.memeasm"
CLI_CASES = [
    # id, args ({OUT} = output file), expect ("zero"/"nonzero"), regex the combined output must match
    ("help", ["-h"], "zero", r"(?i)usage"),
    ("help_long", ["--help"], "zero", r"(?i)usage"),
    ("version", ["-v"], "zero", r"\d+\.\d+"),
    ("o_without_argument", ["-o"], "nonzero", r"(?i)-o\b|output"),
    ("no_input_file", ["-o", "{OUT}"], "nonzero", r"(?i)input|file"),
    ("no_output_file", [P_OK], "nonzero", r"(?i)-o\b|output"),
    ("unknown_option", ["-x", "-o", "{OUT}", P_OK], "nonzero", r"-x"),
    ("unknown_f_option", ["-fbogus", "-o", "{OUT}", P_OK], "nonzero", r"bogus"),
    ("bad_opt_level_5", ["-O5", "-o", "{OUT}", P_OK], "nonzero", r"5"),
    ("bad_opt_level_m4", ["-O-4", "-o", "{OUT}", P_OK], "nonzero", r"-4"),
    ("bad_opt_level_foo", ["-Ofoo", "-o", "{OUT}", P_OK], "nonzero", r"foo"),
    ("bad_compile_mode", ["-fcompile-mode", "foo", "-o", "{OUT}", P_OK], "nonzero", r"(?i)foo|compile.mode"),
    ("compile_mode_missing_value", ["-o", "{OUT}", P_OK, "-fcompile-mode"], "nonzero", r"compile-mode"),
    ("nonexistent_input", ["-o", "{OUT}", "tests/e2e/programs/does_not_exist.memeasm"], "nonzero", r"(?i)does_not_exist|no such file"),
    ("directory_input", ["-o", "{OUT}", "tests/e2e/programs"], "nonzero", r"(?i)programs|directory|regular file"),
    ("options_after_input", ["-o", "{OUT}", P_OK, "-fno-martyrdom", "-O-1"], "zero", r""),
    ("all_modes_together", ["-S", "-d", "-fno-martyrdom", "-fcompile-mode", "obfuscated", "-O-3", "-o", "{OUT}", P_OK], "zero", r""),
    # -i: advertised by the C usage text but removed upstream (e99f0b3); stays rejected (rust-rewrite-changes.md #10)
    ("option_i_rejected", ["-i", "-o", "{OUT}", P_OK], "nonzero", r"-i\b"),
    ("ftarget_linux", ["-ftarget=linux", "-o", "{OUT}", P_OK], "zero", r""),
    ("ftarget_invalid", ["-ftarget=amiga", "-o", "{OUT}", P_OK], "nonzero", r"amiga"),
]


def k_cli(cid, case):
    name, args, expect, rx = case

    def build(side):
        d = workdir(cid, side)
        out = str(d / "out")
        argv = compiler_argv(side, "lin") + [out if a == "{OUT}" else a for a in args]
        env = base_env()
        if side == "ref":
            faketime_env(env, REF_TIME)
        r = run_proc(argv, cwd=str(ROOT), env=env, timeout=COMPILE_TIMEOUT)
        res = {"rc": r["rc"], "out": norm_text(r["stdout"] + r["stderr"], out, argv[0])}
        if name == "ftarget_linux" and r["rc"] == 0:
            res["run"] = run_exe([out], d)
        shutil.rmtree(d, ignore_errors=True)
        return res

    def ok(r):
        good = (r["rc"] == 0) if expect == "zero" else (r["rc"] not in (0, "timeout", "oserror"))
        return good and re.search(rx, r["out"]) is not None

    if name.startswith("ftarget") and CFG.new_is_ref:
        return Result("SKIP", "-ftarget is new-compiler-only (sanity mode)")
    ref = get_ref("cli:" + name, lambda: build("ref")) if not name.startswith("ftarget") else None
    new = build("new")
    det = f"args: {sh_quote(args)}\nexpect rc {expect}, output matching /{rx}/\n"
    if ref:
        det += f"--- reference rc={ref['rc']}\n{ref['out'][:2000]}\n"
    det += f"--- new rc={new['rc']}\n{new['out'][:2000]}\n"
    if not ok(new):
        return Result("FAIL", f"rc={new['rc']} / message check failed", det)
    if name == "ftarget_linux":
        if new.get("run", {}).get("status") != "exit:0":
            return Result("FAIL", "-ftarget=linux executable does not run", det + json.dumps(new.get("run")))
    if ref and not ok(ref):
        return Result("WARN", "reference does not satisfy the loose expectation itself", det)
    return Result("PASS", "", det)


# ------------------------------------------------------------------------------------------------
# gdb: breakpoint on file.memeasm:LINE in a -g executable (new compiler only, optional)
# ------------------------------------------------------------------------------------------------
_port = [24000 + os.getpid() % 2000]


def k_gdb(cid, entry, line, expect_text):
    if CFG.new_is_ref:
        return Result("SKIP", "gdb check targets the new compiler's DWARF output (reference emits STABS)")
    if not shutil.which("gdb-multiarch") or not shutil.which("qemu-x86_64"):
        return Result("SKIP", "gdb-multiarch / qemu-x86_64 not installed")
    d = workdir(cid, "new")
    exe = str(d / "prog")
    c = compile_("new", "lin", ["-g"], entry, exe)
    if c["rc"] != 0:
        return Result("FAIL", "compilation with -g failed", fmt_compile("new", c))
    with _wd_lock:
        _port[0] += 1
        port = _port[0]
    q = subprocess.Popen(["qemu-x86_64", "-L", QEMU_PREFIX, "-g", str(port), exe], cwd=d, stdout=subprocess.PIPE,
                         stderr=subprocess.DEVNULL, stdin=subprocess.DEVNULL, env=base_env())
    time.sleep(0.5)
    src = Path(entry["files"][0]).name
    g = run_proc(["gdb-multiarch", "-batch", "-nx", "-ex", f"set sysroot {QEMU_PREFIX}", "-ex", f"file {exe}",
                  "-ex", f"target remote :{port}", "-ex", f"break {src}:{line}", "-ex", "continue",
                  "-ex", "info line *$pc", "-ex", "kill"], cwd=d, timeout=60)
    try:
        q.kill()
        q.wait(5)
    except Exception:
        pass
    out = g["stdout"].decode("latin-1") + g["stderr"].decode("latin-1")
    shutil.rmtree(d, ignore_errors=True)
    det = f"break {src}:{line}\n{out}"
    if re.search(rf"Breakpoint 1, .*{re.escape(src)}:{line}\b", out, re.S) or re.search(rf"Line {line} of \"[^\"]*{re.escape(src)}\"", out):
        return Result("PASS", "", det)
    return Result("FAIL", f"breakpoint at {src}:{line} not hit", det)


# ------------------------------------------------------------------------------------------------
# planning
# ------------------------------------------------------------------------------------------------
MODE_PROGS = ["calc/factorials", "calc/upper_stdin", "misc/many_functions", "cmd/monke", "crash/die", "cmd/print",
              "multi/three_files", "bully/mixed"]
MODES = [
    ("obfuscated", ["-fcompile-mode", "obfuscated"], ["lin-exe", "lin-asm", "win-asm"]),
    ("O-1", ["-O-1"], ["lin-exe", "lin-asm", "lin-obj", "win-asm", "win-obj"]),
    ("O-3", ["-O-3"], ["lin-exe", "lin-asm", "lin-obj", "win-asm", "win-obj"]),
    ("O69420", ["-O69420"], ["lin-exe", "lin-asm", "lin-obj", "win-asm", "win-obj"]),
    ("no-martyrdom", ["-fno-martyrdom"], ["lin-exe", "lin-asm", "lin-obj"]),
    ("d", ["-d"], ["lin-exe", "lin-asm"]),
    ("g", ["-g"], ["lin-exe", "lin-asm", "lin-obj"]),
    ("d-g-O-1", ["-d", "-g", "-O-1"], ["lin-exe"]),
]
# -O-2 / -O-s executables are ~1.6 GB (reference enum bug, see bugs/opt_minus_*): only built there
GDB_CASES = [("calc/factorials", "upvote rcx"), ("misc/many_functions", None)]


def synth_entries(manifest):
    by = {e["name"]: e for e in manifest}
    out = []
    for name in MODE_PROGS:
        base = by[name]
        for tag, mflags, kinds in MODES:
            e = dict(base)
            e.pop("obfuscated", None)
            e["name"] = f"modes/{name}/{tag}"
            e["category"] = "modes"
            e["flags"] = base.get("flags", []) + mflags
            if "-g" in mflags and "bully" in e["flags"]:
                continue
            e["kinds"] = [k for k in kinds if k in base["kinds"] or k in ("lin-asm", "win-asm")]
            out.append(e)
    for tag, mflags in [("O69420", ["-O69420"]), ("O-1", ["-O-1"]), ("obfuscated", ["-fcompile-mode", "obfuscated"])]:
        e = dict(by["martyrdom/loop"])
        e.update(name=f"modes/martyrdom/loop/{tag}", category="modes", flags=mflags, kinds=["lin-exe"])
        out.append(e)
    return out


def plan(manifest):
    checks = []

    def add(cid, cat, fn, *a):
        checks.append((cid, cat, fn, a))

    for e in manifest + synth_entries(manifest):
        name, cat, flags = e["name"], e["category"], e.get("flags", [])
        variants = [("", flags)]
        if e.get("obfuscated"):
            variants.append(("obfuscated", flags + ["-fcompile-mode", "obfuscated"]))
        for suf, fl in variants:
            sfx = f":{suf}" if suf else ""
            for kind in e["kinds"]:
                cid = f"{kind}:{name}{sfx}"
                if e.get("random"):
                    if kind == "lin-exe":
                        add(cid, cat, k_random_exe, e, fl)
                    elif kind in ("lin-asm", "win-asm"):
                        add(cid, cat, k_random_asm, e, fl, kind[:3])
                    else:
                        add(cid, cat, lambda *a: Result("SKIP", "random program: covered by the asm/exe checks"))
                    continue
                if kind == "lin-exe":
                    add(cid, cat, k_exe, e, fl)
                elif kind in ("lin-asm", "win-asm"):
                    add(cid, cat, k_asm, e, fl, kind[:3])
                elif kind in ("lin-obj", "win-obj"):
                    add(cid, cat, k_obj, e, fl, kind[:3])
                elif kind == "win-exe":
                    add(cid, cat, k_win_exe, e, fl)
            if e.get("driver"):
                add(f"lin-link:{name}{sfx}", cat, k_link, e, fl)
            if "win-exe" in e["kinds"] and not e.get("random"):
                if CFG.wine_run:
                    add(f"win-run:{name}{sfx}", "windows-run", k_exe, e, fl, "win")
            if e.get("martyrdom"):
                for v in e.get("martyrdom_variants", [[], ["-fno-martyrdom"]]):
                    vn = "default" if not v else "".join(v).lstrip("-")
                    add(f"martyrdom:{name}{sfx}:{vn}", cat if cat != "martyrdom" else "martyrdom", k_martyrdom, e, fl + v)
    for c in CLI_CASES:
        add(f"cli:{c[0]}", "cli", k_cli, c)
    by = {e["name"]: e for e in manifest}
    for name, pat in GDB_CASES:
        e = by[name]
        lines = (ROOT / e["files"][0]).read_text().split("\n")
        if pat is None:
            idx = next(i for i, l in enumerate(lines) if l.strip() and not l.lstrip().startswith(("I like to have fun", "What the hell")))
        else:
            idx = next(i for i, l in enumerate(lines) if l.strip() == pat)
        add(f"gdb:{name}:{idx + 1}", "gdb", k_gdb, e, idx + 1, pat)
    return checks


# ------------------------------------------------------------------------------------------------
# main
# ------------------------------------------------------------------------------------------------
def resolve(p):
    if not p:
        return p
    q = Path(p)
    if not q.is_absolute():
        q = (Path.cwd() / q) if (Path.cwd() / q).exists() else ROOT / q
    return str(q)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("filters", nargs="*", help="only run checks whose id matches one of these regexes")
    ap.add_argument("-j", "--jobs", type=int, default=min(8, os.cpu_count() or 2))
    ap.add_argument("--update-expected", action="store_true", help="(live mode) write reference outcomes to expected/")
    ap.add_argument("--expected", action="store_true", help="use expected/ even if the reference is installed")
    ap.add_argument("--allow-old-bugs", action="store_true", help="do not fail when the new compiler reproduces reference bugs")
    ap.add_argument("--keep-work", action="store_true")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--martyr-helper", nargs=2, metavar=("EXE", "TIMEOUT"), help=argparse.SUPPRESS)
    a = ap.parse_args()
    if a.martyr_helper:
        martyr_helper(a.martyr_helper[0], float(a.martyr_helper[1]))
        return 0

    CFG.ref = os.environ.get("REF", "/opt/memeasm-ref/memeasm-ref")
    CFG.ref_win = os.environ.get("REF_WIN", "/opt/memeasm-ref/memeasm-ref-win")
    CFG.new = resolve(os.environ.get("NEW", "target/release/memeasm"))
    real_refs = {os.path.realpath(x) for x in (CFG.ref, "/opt/memeasm-ref/memeasm-linux", CFG.ref_win)}
    CFG.new_is_ref = os.path.realpath(CFG.new) in real_refs
    CFG.new_win = resolve(os.environ.get("NEW_WIN")) or (CFG.ref_win if CFG.new_is_ref else None)
    CFG.live = not a.expected and os.path.exists(CFG.ref) and os.path.exists(CFG.ref_win)
    CFG.update = a.update_expected
    if CFG.update and not CFG.live:
        print("--update-expected needs the reference compilers", file=sys.stderr)
        return 2
    CFG.faketime = next((f for f in FAKETIME_LIBS if os.path.exists(f)), None)
    CFG.allow_old_bugs = a.allow_old_bugs or CFG.new_is_ref
    CFG.wine_run = os.environ.get("WINE_RUN") or None
    if not os.path.exists(CFG.new) and not a.list:
        print(f"new compiler not found: {CFG.new} (set NEW=...)", file=sys.stderr)
        return 2
    if CFG.live and not CFG.faketime:
        print("warning: libfaketime not found; random programs cannot be sampled reproducibly", file=sys.stderr)

    manifest = json.loads(MANIFEST.read_text())
    checks = plan(manifest)
    if a.filters:
        checks = [c for c in checks if any(re.search(f, c[0]) for f in a.filters)]
    if a.list:
        for c in checks:
            print(c[1].ljust(12), c[0])
        print(f"{len(checks)} checks")
        return 0

    OUTDIR.mkdir(exist_ok=True)
    CFG.work = tempfile.mkdtemp(prefix="memeasm-e2e-")
    print(f"new: {CFG.new}" + (f"   (windows: {CFG.new_win})" if CFG.new_win else "   (windows: -ftarget=windows)")
          + ("   [SANITY MODE: new == reference]" if CFG.new_is_ref else ""))
    print(f"reference: {'live ' + CFG.ref + ' / ' + CFG.ref_win if CFG.live else 'stored outcomes in ' + str(EXPECTED)}"
          f"   faketime: {'yes' if CFG.faketime else 'no'}   wine: {CFG.wine_run or 'no (set WINE_RUN)'}")
    print(f"{len(checks)} checks, {a.jobs} jobs")
    results = {}
    t0 = time.time()
    done = [0]

    def run_one(c):
        cid, cat, fn, args = c
        st = time.time()
        try:
            r = fn(cid, *args)
        except Exception:
            r = Result("ERROR", "runner exception", traceback.format_exc())
        r.elapsed = time.time() - st
        return cid, cat, r

    # heavy checks first so they do not serialise at the end
    checks.sort(key=lambda c: (0 if ("O-2" in c[0] or "O-s" in c[0] or "opt_minus" in c[0]) else
                               1 if c[0].startswith(("lin-exe:rand", "lin-asm:rand", "lin-exe:workflows/example",
                                                     "lin-asm:workflows", "win-asm:rand", "win-asm:workflows")) else 2))
    with ThreadPoolExecutor(a.jobs) as ex:
        futs = [ex.submit(run_one, c) for c in checks]
        for f in as_completed(futs):
            cid, cat, r = f.result()
            results[cid] = (cat, r)
            done[0] += 1
            if sys.stdout.isatty():
                print(f"\r[{done[0]}/{len(checks)}] {cid[:70]:<70}", end="", flush=True)
            elif r.status in FAILING:
                print(f"  {r.status:<7} {cid}  {r.summary[:100]}", flush=True)
    if sys.stdout.isatty():
        print("\r" + " " * 90 + "\r", end="")
    if not a.keep_work:
        shutil.rmtree(CFG.work, ignore_errors=True)
    else:
        print(f"work dir kept: {CFG.work}")

    # ---- report ----
    cats = {}
    for cid, (cat, r) in results.items():
        cats.setdefault(cat, {}).setdefault(r.status, 0)
        cats[cat][r.status] += 1
    cols = [s for s in STATUSES if any(s in v for v in cats.values())]
    print()
    print(f"{'category':<14}" + "".join(f"{s:>9}" for s in cols) + f"{'total':>9}")
    for cat in sorted(cats):
        print(f"{cat:<14}" + "".join(f"{cats[cat].get(s, 0) or '':>9}" for s in cols) + f"{sum(cats[cat].values()):>9}")
    tot = {s: sum(v.get(s, 0) for v in cats.values()) for s in cols}
    print(f"{'TOTAL':<14}" + "".join(f"{tot[s]:>9}" for s in cols) + f"{len(results):>9}")
    print()
    order = {s: i for i, s in enumerate(reversed(STATUSES))}
    notable = sorted(((cid, cat, r) for cid, (cat, r) in results.items() if r.status not in ("PASS",)),
                     key=lambda x: (order[x[2].status], x[0]))
    for cid, cat, r in notable:
        print(f"{r.status:<8} {cid:<58} {r.summary[:110]}")
    with open(OUTDIR / "details.txt", "w") as f:
        f.write(f"new={CFG.new} new_win={CFG.new_win} live={CFG.live} at {time.ctime()}\n\n")
        for cid, cat, r in notable + sorted(((cid, cat, r) for cid, (cat, r) in results.items() if r.status == "PASS"),
                                            key=lambda x: x[0]):
            f.write(f"{'=' * 100}\n{r.status} {cid}  [{cat}]  {r.summary}  ({getattr(r, 'elapsed', 0):.1f}s)\n{r.details}\n")
    with open(OUTDIR / "results.json", "w") as f:
        json.dump({cid: {"category": cat, "status": r.status, "summary": r.summary} for cid, (cat, r) in sorted(results.items())},
                  f, indent=1)
    nfail = sum(1 for _, (_, r) in results.items() if r.status in FAILING)
    print(f"\n{len(results)} checks in {time.time() - t0:.0f}s: {nfail} failing.  Details: {OUTDIR / 'details.txt'}")
    return 1 if nfail else 0


if __name__ == "__main__":
    sys.exit(main())
