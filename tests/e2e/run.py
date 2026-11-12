#!/usr/bin/env python3
"""End-to-end behavioural test suite: new MemeAssembly compiler vs the original C compiler.

Usage:  tests/e2e/run.sh [options] [FILTER ...]     (FILTER = substring/regex of check ids)

Environment:
  NEW       new compiler (default: target/release/memeasm)
  NEW_WIN   compiler to use for Windows targets instead of "$NEW --target=windows"
            (auto-set to REF_WIN when NEW is the reference -> sanity mode)
  REF       reference Linux compiler   (default /opt/memeasm-ref/memeasm-ref)
  REF_WIN   reference Windows compiler (default /opt/memeasm-ref/memeasm-ref-win)

Reference outcomes are computed live when the reference compilers exist (and can be stored with
--update-expected), otherwise they are read from tests/e2e/expected/.  See tests/e2e/README.md (status
meanings, how documented fixes are judged) and tests/e2e/gen_programs.py for the corpus, which is
regenerated into tests/e2e/programs/ at the start of every run.

Command line flags (manifest, check tables) are written in the reference's getopt spelling
(-fcompile-mode bully, -O-1, -fno-martyrdom); the new compiler gets the clap spelling of the same
options (new_spelling).

Every spawned process runs in its own session with a timeout and rlimits; its process group is
SIGKILLed when it finishes or times out, and all live groups are killed on exit/SIGTERM/SIGINT.
"""
import argparse
import atexit
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


# Every spawned process gets its own process group (start_new_session), which is SIGKILLed when the
# command finishes, times out, or the suite exits (atexit / SIGINT / SIGTERM).  This also reaps
# grandchildren (the reference pipes into gcc/as/ld, Wine starts helpers) and the reference compiler
# when it hangs (e.g. "perfectly balanced" in bully mode, docs/rust-rewrite-changes.md #8).
_live_groups = set()
_groups_lock = threading.Lock()


def _spawn(argv, **kw):
    p = subprocess.Popen(argv, start_new_session=True, preexec_fn=_limits, **kw)
    with _groups_lock:
        _live_groups.add(p.pid)
    return p


def _kill_group(p):
    try:
        os.killpg(p.pid, signal.SIGKILL)
    except OSError:
        pass
    with _groups_lock:
        _live_groups.discard(p.pid)


def kill_all_groups(*_):
    with _groups_lock:
        groups = list(_live_groups)
        _live_groups.clear()
    for g in groups:
        try:
            os.killpg(g, signal.SIGKILL)
        except OSError:
            pass


def _die(signum, _frame):
    kill_all_groups()
    os._exit(128 + signum)


def base_env():
    env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "LANG": "C", "LC_ALL": "C", "TZ": "UTC",
           "QEMU_LD_PREFIX": QEMU_PREFIX, "HOME": os.environ.get("HOME", "/tmp")}
    if "TMPDIR" in os.environ:
        env["TMPDIR"] = os.environ["TMPDIR"]
    return env


def run_proc(argv, cwd=None, env=None, timeout=60, stdin=None, input=None):
    """Run a (compile/tool) command, return dict rc/stdout/stderr (bytes) or rc='timeout'.
    `input` (bytes) is fed to stdin.  The whole process group is killed afterwards (and on timeout)."""
    if input is not None:
        stdin = subprocess.PIPE
    try:
        p = _spawn(argv, cwd=cwd, env=env or base_env(), stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                   stdin=subprocess.DEVNULL if stdin is None else stdin)
    except OSError as e:
        return {"rc": "oserror", "stdout": b"", "stderr": str(e).encode()}
    try:
        out, err = p.communicate(input=input, timeout=timeout)
        rc = p.returncode
    except subprocess.TimeoutExpired:
        _kill_group(p)
        try:
            out, err = p.communicate(timeout=10)
        except subprocess.TimeoutExpired:  # a grandchild escaped the group and holds the pipes
            out, err = b"", b""
            p.kill()
            p.wait()
        rc = "timeout"
    finally:
        _kill_group(p)
    return {"rc": rc, "stdout": out or b"", "stderr": err or b""}


def status_of(rc):
    if rc is None:
        return "nonterm"
    return f"exit:{rc}" if rc >= 0 else f"signal:{-rc}"


def run_exe(argv, cwd, stdin_path=None, timeout=5.0):
    """Run a test program (x86-64 via qemu binfmt).  Captures at most STDOUT_CAP bytes of stdout;
    non-terminating programs (timeout / unbounded output) get status 'nonterm'."""
    stdin = open(stdin_path, "rb") if stdin_path else subprocess.DEVNULL
    try:
        p = _spawn(argv, cwd=cwd, env=base_env(), stdin=stdin, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
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
        _kill_group(p)
    try:
        rc = p.wait(timeout=max(1.0, deadline - time.monotonic() + 1.0))
    except subprocess.TimeoutExpired:
        _kill_group(p)
        p.wait()
        nonterm = True
    _kill_group(p)
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
        return [CFG.new_win] if CFG.new_win else [CFG.new, "--target=windows"]
    return [CFG.new]


def new_spelling(args):
    """The new compiler's (clap) spelling of reference (getopt) arguments."""
    out = []
    for a in args:
        if a == "-fcompile-mode":
            a = "--compile-mode"
        elif a.startswith("-fcompile-mode="):
            a = "--compile-mode=" + a.split("=", 1)[1]
        elif a == "-fno-martyrdom":
            a = "--no-martyrdom"
        elif a.startswith("-ftarget="):
            a = "--target=" + a.split("=", 1)[1]
        elif a.startswith("-O") and len(a) > 2:
            a = "--opt-level=" + a[2:]
        out.append(a)
    return out


def side_args(side, args):
    """args as the compiler of side expects them (sanity mode: NEW is the reference). Debug
    messages are not compared: the C compiler runs without -d, and norm_text removes the "debug: "
    lines of the new compiler (docs/rust-rewrite-changes.md, "Debug output")."""
    if side == "ref" or CFG.new_is_ref:
        return [a for a in args if a != "-d"]
    return new_spelling(args)


def faketime_env(env, when):
    if CFG.faketime:
        env["LD_PRELOAD"] = CFG.faketime
        env["FAKETIME"] = when
        env["FAKETIME_DONT_FAKE_MONOTONIC"] = "1"
    return env


# A debug message of the new compiler; one that follows output without a line end (such as the
# colour reset after the Snap banner) is cut out of its line.
DEBUG_LINE = re.compile(r"^((?:\x1b\[0m)?)debug: [^\n]*\n", re.M)


def norm_text(b, out=None, argv0=None):
    t = b.decode("latin-1") if isinstance(b, bytes) else b
    if out:
        t = t.replace(out, "<OUT>")
    for a in filter(None, set([argv0, os.path.realpath(argv0) if argv0 else None, CFG.ref, CFG.ref_win,
                               CFG.new, CFG.new_win])):
        t = t.replace(a, "<MEMEASM>")
    t = re.sub(r"/opt/memeasm-ref/[\w.-]+", "<MEMEASM>", t)
    t = re.sub(r"/tmp/cc\w+\.(o|s)", "/tmp/<TMP>", t)
    return DEBUG_LINE.sub(r"\1", t)


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
    argv = compiler_argv(side, target) + side_args(side, flags) + ([mode] if mode else []) + ["-o", out]
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
    # the reference adds " on <date>" to the header line
    t = re.sub(r"^# Generated by the MemeAssembly compiler (v[\d.]+)(?: on .*)?$", r"# Generated by the MemeAssembly compiler \1",
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


def _has_compile_timeout(v):
    return '"class": "timeout"' in json.dumps(v, sort_keys=True)


def get_ref(cid, producer, timeout_expected=False):
    if CFG.live:
        v = producer()
        if CFG.update and not timeout_expected and _has_compile_timeout(v):
            # a reference compile that times out is almost always host overload: retry once, and never
            # store a spurious timeout as the expected outcome
            v = producer()
            if _has_compile_timeout(v):
                raise RuntimeError("reference compile timed out (host overloaded?); expectation not updated")
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


def sub_specs(spec):
    if spec["kind"] == "per_kind":
        return [v for k, v in spec.items() if k != "kind"]
    return spec.get("of", [])


def spec_needs_oracle(spec):
    if spec["kind"] == "runs":
        return True
    return any(spec_needs_oracle(s) for s in sub_specs(spec))


def find_oracle_spec(spec):
    if spec["kind"] == "runs":
        return spec
    for s in sub_specs(spec):
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


def low_byte_status(run):
    """A Linux run as the Windows wrapper reports it (exit code & 0xff)."""
    if run["status"].startswith("exit:"):
        return dict(run, status=f"exit:{int(run['status'][5:]) & 0xff}")
    return run


def eval_spec(spec, kind, new, ref, entry):
    """True if the new outcome satisfies the 'correct behaviour' spec, None if not evaluable."""
    k = spec["kind"]
    comp = new.get("compile") if isinstance(new, dict) else None
    if k == "any_of":
        vals = [eval_spec(s, kind, new, ref, entry) for s in spec["of"]]
        if any(v is True for v in vals):
            return True
        return False if any(v is False for v in vals) else None
    if k == "per_kind":  # {"kind": "per_kind", "<check kind>": spec, ..., "default": spec}
        sub = spec.get(kind, spec.get("default"))
        return eval_spec(sub, kind, new, ref, entry) if sub else None
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
    if k == "runs":  # the reference-built Linux oracle program shows the correct behaviour
        if kind not in ("lin-exe", "win-run") or not ref.get("oracle") or not isinstance(ref["oracle"].get("runs"), list):
            return None
        o = ref["oracle"]["runs"]
        if kind == "win-run":
            o = [low_byte_status(x) for x in o]
        return comp is not None and comp["rc"] == 0 and not cmp_runs(o, new["runs"], entry.get("exit_code_unspecified"))
    if k == "linux_runs":
        if kind != "win-run" or not isinstance((ref.get("linux") or {}).get("runs"), list):
            return None
        lin = [low_byte_status(x) for x in ref["linux"]["runs"]]
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
    if res.status == "FIXED":
        # the comparer already accounted for the difference with a documented fix (whitelist / Linux behaviour);
        # a 'correct' spec for this kind must still hold
        if in_bug and eval_spec(rb["correct"], kind, new, ref, entry) is False:
            return Result("FAIL", "matches the whitelisted reference but not the specified correct behaviour",
                          tag + f"\ncorrect = {json.dumps(rb['correct'])}\n" + res.details)
        return Result("FIXED", res.summary, tag + "\n" + res.details)
    if res.status == "OLDBUG":
        return res
    if res.status in ("PASS", "WARN"):
        if in_bug and rb["correct"]["kind"] == "unspecified":
            return Result(res.status, "same as reference (reference behaviour is undefined here)", tag + "\n" + res.details)
        if in_bug:
            st = "PASS" if CFG.allow_old_bugs else "OLDBUG"
            return Result(st, "new compiler reproduces the reference bug" + (" (allowed)" if CFG.allow_old_bugs else ""),
                          tag + "\n" + res.details)
        return res
    if not in_bug:
        # the bug does not concern this kind: a difference is judged as for any other program (the documented
        # -S/-O changes are already accounted for by the whitelist)
        return Result(res.status, res.summary, tag + "\n" + res.details)
    v = eval_spec(rb["correct"], kind, new, ref, entry)
    if v is True:
        return Result("FIXED", "reference bug fixed: behaves as specified", tag + "\n" + res.details)
    if v is None:
        return Result("CHANGED", "differs from buggy reference (correct behaviour unspecified): " + res.summary,
                      tag + "\n" + res.details)
    return Result("FAIL", "differs from reference but not the specified correct behaviour: " + res.summary,
                  tag + f"\ncorrect = {json.dumps(rb['correct'])}\n" + res.details)


def generic(cid, entry, kind, ref_fn, new_fn, cmp_fn):
    ref = get_ref(cid, ref_fn, timeout_expected=bool(entry.get("compile_timeout")))
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
            if "win-run" in (bug_spec(entry) or {}).get("bug_kinds", []):
                o = ref_oracle(entry, flags, cid)
                if o:
                    r["oracle"] = o
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
                lin_n = [low_byte_status(x) for x in lruns]
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


# ------------------------------------------------------------------------------------------------
# intentional differences: the reference -S text is rewritten by tests/frontend_parity_whitelist.pl
# (one rule per fixed bug of docs/rust-rewrite-changes.md), exactly as tests/frontend_parity.sh does,
# before it is compared with the new compiler's -S text (lin-asm/win-asm) or assembled with the
# cross gas and compared with the new compiler's -O object (lin-obj/win-obj).
# ------------------------------------------------------------------------------------------------
WHITELIST = ROOT / "tests" / "frontend_parity_whitelist.pl"
GAS = {"lin": ["x86_64-linux-gnu-as", "-W", "--64"],   # what the reference's `gcc -w -c -x assembler -` runs
       "win": ["x86_64-w64-mingw32-as", "-W"]}
_BULLY_PTR = re.compile(r"(BYTE|WORD|DWORD|QWORD) PTR \[[a-z0-9]+\]")


def flag_context(flags):
    """(compile mode, optimisation flag) of a command line, read like frontend_parity.sh's check()."""
    mode, opt, prev = "noob", "", None
    for a in flags:
        if prev in ("-fcompile-mode", "-fc"):
            mode = a
        if a.startswith("-fcompile-mode="):
            mode = a.split("=", 1)[1]
        if re.fullmatch(r"-O-[0-9s]|-O69420", a):
            opt = a
        prev = a
    return mode, opt


def bully_size(new_asm):
    """BULLY_SIZE as frontend_parity.sh's expected_asm() derives it: the size of the first sized
    pointer in the new -S output (power's own `QWORD PTR [rsp]` operands are not bully pointers)."""
    for m in _BULLY_PTR.finditer(new_asm or ""):
        if m.group(0) != "QWORD PTR [rsp]":
            return m.group(1)
    return ""


def whitelist(asm, target, flags, new_asm=""):
    """The reference -S text with the documented fixes applied.  Raises if the script fails."""
    mode, opt = flag_context(flags)
    env = base_env()
    env["BULLY_SIZE"] = bully_size(new_asm) if mode == "bully" else ""
    r = run_proc(["perl", str(WHITELIST), {"lin": "linux", "win": "windows"}[target], mode, opt], env=env,
                 input=asm.encode("latin-1"), timeout=60)
    if r["rc"] != 0:
        raise RuntimeError(f"whitelist script failed (rc={r['rc']}): {r['stderr'].decode('latin-1')[-500:]}")
    return r["stdout"].decode("latin-1")


def judge(ra, wa, na, what="assembly", fmt=None):
    """ra = reference, wa = whitelisted reference, na = new (texts, or objdump infos with fmt).
    Returns (status, summary, details)."""
    fmt = fmt or (lambda x: x)
    if na == wa:
        if wa == ra:
            return "PASS", "", ""
        return "FIXED", f"{what} equals the reference with the documented fixes applied (frontend_parity_whitelist.pl)", \
            "--- whitelist rewrite of the reference:\n" + text_diff(fmt(ra), fmt(wa), "reference", "whitelisted", n=60)
    if na == ra and wa != ra:
        st = "PASS" if CFG.allow_old_bugs else "OLDBUG"
        return st, f"{what} equals the unfixed reference: a documented fix is missing" + \
            (" (allowed)" if CFG.allow_old_bugs else ""), text_diff(fmt(ra), fmt(wa), "reference", "whitelisted")
    return "FAIL", f"{what} differs from the whitelisted reference", \
        text_diff(fmt(wa), fmt(na), "whitelisted reference", "new")


def gas_objinfo(asm, target, d):
    """Assembles -S text with the cross gas (as the reference's gcc pipe does).
    Returns (objdump_info or None, gas stderr)."""
    d.mkdir(parents=True, exist_ok=True)
    out = d / f"gas{len(list(d.iterdir()))}.o"
    r = run_proc(GAS[target] + ["-o", str(out), "-"], cwd=d, input=asm.encode("latin-1"))
    if r["rc"] != 0 or not out.exists():
        return None, r["stderr"].decode("latin-1")[-1500:]
    return objdump_info(OBJDUMP_WIN if target == "win" else OBJDUMP_LIN, str(out)), ""


def k_asm(cid, entry, flags, target="lin"):
    kind = "lin-asm" if target == "lin" else "win-asm"

    def ref_fn():
        r = asm_build("ref", entry, flags, cid, target)
        if CFG.faketime:
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
        ra, na = ref.get("asm") or "", new.get("asm") or ""
        st, summary, d2 = judge(ra, whitelist(ra, target, flags, na), na)
        return Result(st, summary, det + d2)
    return generic(cid, entry, kind, ref_fn, lambda: asm_build("new", entry, flags, cid, target), cmp)


def obj_build(side, entry, flags, cid, target):
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
    # the -S text of the same compilation: the reference's is whitelisted and assembled (k_obj); the new
    # compiler's only provides the bully operand size for the whitelist (as in frontend_parity.sh)
    if side == "ref" or flag_context(flags)[0] == "bully":
        s = compile_(side, target, flags, entry, str(d / "out.S"), mode="-S")
        try:
            r["asm"] = Path(d / "out.S").read_text(encoding="latin-1") if s["rc"] == 0 else None
        except OSError:
            r["asm"] = None
    shutil.rmtree(d, ignore_errors=True)
    return r


def obj_problems(want, got, det):
    """Compares two objdump infos -> (problems, exact .text equal, normalised .text equal, details)."""
    probs = []
    if want["globals"] != got["globals"]:
        probs.append("global/undefined symbols differ")
        det += text_diff("\n".join(want["globals"]), "\n".join(got["globals"]))
    if want["data"] != got["data"]:
        probs.append(".data differs")
        det += text_diff(want["data"], got["data"])
    return probs, want["dis"] == got["dis"], norm_dis(want["dis"]) == norm_dis(got["dis"]), det


def k_obj(cid, entry, flags, target="lin"):
    """The new -O object is compared with the reference -S text, rewritten by the whitelist (documented fixes) and
    assembled with the cross gas.  A self-check makes sure that assembling the unmodified reference -S text gives
    the reference's own -O object (same objdump info), and the mingw link of the object is compared as before."""
    kind = "lin-obj" if target == "lin" else "win-obj"

    def cmp(ref, new):
        det = fmt_compile("reference", ref["compile"]) + fmt_compile("new", new["compile"])
        if "asm" not in ref:
            return Result("ERROR", "stored reference outcome has no -S text: regenerate expected/ (--update-expected)", det)
        no = new.get("obj")
        ra = ref.get("asm")
        gas_dir = workdir(cid, "gas")
        try:
            wa = whitelist(ra, target, flags, new.get("asm") or "") if ra is not None else None
            wo, werr = gas_objinfo(wa, target, gas_dir) if wa is not None else (None, "no reference -S text")
            if wa == ra:
                ro_s, rerr = wo, werr
            else:
                ro_s, rerr = gas_objinfo(ra, target, gas_dir) if ra is not None else (None, "")
        finally:
            shutil.rmtree(gas_dir.parent, ignore_errors=True)
        p = cmp_compile(ref["compile"], new["compile"])
        if p:
            if ref["compile"]["class"] == "toolchain" and new["compile"]["rc"] == 0 and isinstance(no, dict) and wo:
                # the reference's gas run failed, but the whitelisted reference -S text assembles: judge against it
                probs, same, same_n, det = obj_problems(wo, no, det)
                if not probs and same:
                    return Result("FIXED", "reference -O fails in gas; the whitelisted reference -S assembles to the new "
                                  "object", det + "--- reference gas errors:\n" + rerr + "\n" +
                                  text_diff(ra, wa, "reference", "whitelisted", n=60))
                return Result("FAIL", "reference -O fails in gas; the new object differs from the whitelisted reference",
                              det + text_diff(norm_dis(wo["dis"]), norm_dis(no["dis"]), "whitelisted reference", "new"))
            return Result("FAIL", p, det)
        if ref["compile"]["rc"] != 0:
            return Result("PASS", "", det)
        ro = ref.get("obj")
        if not isinstance(no, dict) or not isinstance(ro, dict):
            return Result("FAIL", f"object: {no!r:.80}", det)
        # self-check of the assembling pipeline: gas(reference -S) must reproduce the reference -O object
        if ro_s is None:
            return Result("ERROR", "suite: the reference -S text does not assemble with " + GAS[target][0], det + rerr)
        sp, ssame, _, _ = obj_problems(ro, ro_s, "")
        if sp or not ssame:
            return Result("ERROR", "suite: assembling the reference -S text does not reproduce the reference -O object",
                          det + text_diff(ro["dis"], ro_s["dis"], "reference -O", "gas(reference -S)"))
        if wo is None:
            return Result("FAIL", "the whitelisted reference -S text does not assemble", det + werr)
        probs, same, same_n, det = obj_problems(wo, no, det)
        if "link_rc" in ref and (ref["link_rc"] == 0) != (new.get("link_rc") == 0):
            probs.append(f"mingw link rc reference={ref['link_rc']} new={new.get('link_rc')}")
            det += "--- new link stderr:\n" + new.get("link_err", "") + "\n"
        fixed = wa != ra
        if same and not probs:
            if not fixed or wo == ro_s:
                return Result("PASS", "", det)
            return Result("FIXED", "object equals the whitelisted reference -S text assembled with gas (documented "
                          "fixes, frontend_parity_whitelist.pl)", det + text_diff(ra, wa, "reference", "whitelisted", n=60))
        if fixed and not obj_problems(ro_s, no, "")[0] and ro_s["dis"] == no["dis"]:
            return Result("PASS" if CFG.allow_old_bugs else "OLDBUG", "object equals the unfixed reference: a documented "
                          "fix is missing", det + text_diff(ra, wa, "reference", "whitelisted"))
        if same_n:
            if probs:
                return Result("FAIL", "; ".join(probs), det)
            return Result("WARN", "disassembly equal modulo encodings/addresses/symbol names",
                          det + text_diff(wo["dis"], no["dis"], "whitelisted reference", "new"))
        probs.append("instructions differ")
        return Result("FAIL", "; ".join(probs) + (" (vs the whitelisted reference)" if fixed else ""),
                      det + text_diff(norm_dis(wo["dis"]), norm_dis(no["dis"]), "whitelisted reference", "new")
                      + "\n(raw)\n" + text_diff(wo["dis"], no["dis"], "whitelisted reference", "new", n=40))
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
# time(NULL); libfaketime freezes the clock so the reference's choices can be sampled. The new
# compiler draws from an OS-seeded generator, so its results are compared with the set of sampled
# reference results (or, for perfectly balanced, with the program without those lines).
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
        wl = lambda t: whitelist(t, target, flags, a)
        wvariants = {v: wl(v) for v in ref["variants"]}
        for v, w in wvariants.items():
            if a == w:
                if w == v:
                    return Result("PASS", "random: matches a sampled reference variant (confused-stonks label ignored)", det)
                return Result("FIXED", "random: matches a sampled reference variant with the documented fixes applied "
                              "(frontend_parity_whitelist.pl)", det + text_diff(v, w, "reference", "whitelisted", n=60))
        if a in wvariants:
            return Result("PASS" if CFG.allow_old_bugs else "OLDBUG", "random: equals an unfixed reference variant: a "
                          "documented fix is missing", det + text_diff(a, wvariants[a], "reference", "whitelisted"))
        # blank lines are ignored: the deleted/"perfectly balanced" lines themselves leave empty '\t' lines behind
        nonblank = lambda t: [l for l in t.splitlines() if l.strip()]
        if ref.get("full"):
            if is_subsequence(nonblank(a), nonblank(wl(ref["full"]))):
                return Result("PASS", "random: perfectly balanced output is a line-subsequence of the full program "
                              "(whitelisted)", det)
        best = min(wvariants.values(), key=lambda v: len(text_diff(v, a, n=10 ** 6))) if wvariants else ""
        return Result("FAIL", "random: assembly matches no sampled (whitelisted) reference variant",
                      det + text_diff(best, a, "whitelisted reference", "new"))
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
    # id, args ({OUT} = output file; reference spelling, see new_spelling), expect ("zero"/"nonzero"),
    # regex the combined output must match
    ("help", ["-h"], "zero", r"(?i)usage"),
    ("help_long", ["--help"], "zero", r"(?i)usage"),
    ("version", ["-v"], "zero", r"\d+\.\d+"),
    ("o_without_argument", ["-o"], "nonzero", r"(?i)-o\b|output"),
    ("no_input_file", ["-o", "{OUT}"], "nonzero", r"(?i)input|file"),
    ("no_output_file", [P_OK], "nonzero", r"(?i)-o\b|output"),
    ("unknown_option", ["-x", "-o", "{OUT}", P_OK], "nonzero", r"-x"),
    ("unknown_f_option", ["-fbogus", "-o", "{OUT}", P_OK], "nonzero", r"bogus|'-f'"),
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
    ("ftarget_invalid", ["-ftarget=amiga", "-o", "{OUT}", P_OK], "nonzero", r"(?i)amiga|invalid target"),
    # new compiler only: -S and -O together are rejected (the reference let the last one win)
    ("assembly_and_object", ["-S", "-O", "-o", "{OUT}", P_OK], "nonzero", r"(?i)cannot be used with"),
]


def k_cli(cid, case):
    name, args, expect, rx = case

    def build(side):
        d = workdir(cid, side)
        out = str(d / "out")
        argv = compiler_argv(side, "lin") + side_args(side, [out if a == "{OUT}" else a for a in args])
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

    new_only = name.startswith("ftarget") or name == "assembly_and_object"
    if new_only and CFG.new_is_ref:
        return Result("SKIP", "new-compiler-only case (sanity mode)")
    ref = get_ref("cli:" + name, lambda: build("ref")) if not new_only else None
    new = build("new")
    det = f"args: {sh_quote(args)}\nexpect rc {expect}, output matching /{rx}/\n"
    if ref:
        det += f"--- reference rc={ref['rc']}\n{ref['out'][:2000]}\n"
    det += f"--- new rc={new['rc']}\n{new['out'][:2000]}\n"
    if not ok(new):
        return Result("FAIL", f"rc={new['rc']} / message check failed", det)
    if name == "ftarget_linux":
        if new.get("run", {}).get("status") != "exit:0":
            return Result("FAIL", "--target=linux executable does not run", det + json.dumps(new.get("run")))
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
    q = _spawn(["qemu-x86_64", "-L", QEMU_PREFIX, "-g", str(port), exe], cwd=d, stdout=subprocess.DEVNULL,
               stderr=subprocess.DEVNULL, stdin=subprocess.DEVNULL, env=base_env())
    time.sleep(0.5)
    src = Path(entry["files"][0]).name
    g = run_proc(["gdb-multiarch", "-batch", "-nx", "-ex", f"set sysroot {QEMU_PREFIX}", "-ex", f"file {exe}",
                  "-ex", f"target remote :{port}", "-ex", f"break {src}:{line}", "-ex", "continue",
                  "-ex", "info line *$pc", "-ex", "kill"], cwd=d, timeout=60)
    _kill_group(q)
    try:
        q.wait(5)
    except subprocess.TimeoutExpired:
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
    ("d-O-1", ["-d", "-O-1"], ["lin-exe"]),
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
                if CFG.wine_run and e.get("win_run_unsupported"):
                    add(f"win-run:{name}{sfx}", "windows-run",
                        lambda *a, why=e["win_run_unsupported"]: Result("SKIP", "cannot run under box64+Wine: " + why))
                elif CFG.wine_run:
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
    atexit.register(kill_all_groups)
    signal.signal(signal.SIGTERM, _die)
    signal.signal(signal.SIGINT, _die)
    signal.signal(signal.SIGHUP, _die)
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

    # the corpus is generated, not committed
    subprocess.run([sys.executable, "-I", str(HERE / "gen_programs.py")], check=True, stdout=subprocess.DEVNULL)
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
    print(f"new: {CFG.new}" + (f"   (windows: {CFG.new_win})" if CFG.new_win else "   (windows: --target=windows)")
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
