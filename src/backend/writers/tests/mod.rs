//! Writer tests.
//!
//! Most tests compare against the real toolchain: the fixtures in `fixtures/` are the `-S`
//! output of the original C compiler (per target), assembled with GNU as into an ELF object
//! that [`elf_import`] turns into an [`ObjModule`]. The writers' output is then compared with
//! what gas/mingw-as/llvm-mc produce, linked with the system linkers and executed.
//! Tests needing a tool that isn't installed print a note and pass, so `cargo test` works on
//! any host; `MEMEASM_REQUIRE_TOOLS=1` turns missing tools into failures.
//!
//! Tools used (Debian/Ubuntu package names): `binutils-x86-64-linux-gnu`,
//! `gcc-x86-64-linux-gnu` (linking/running references), `binutils-mingw-w64-x86-64`,
//! `llvm`/`lld` (`llvm-mc`, `llvm-objdump`, `ld64.lld`), `qemu-user` (to run x86-64 programs on
//! other hosts). Windows executables run only if `MEMEASM_WINE` names a command that runs a PE
//! file (e.g. a `wine` wrapper).
#![allow(dead_code)]

mod coff_tests;
mod elf_import;
mod elf_tests;
mod macho_tests;
mod pe_tests;
mod samples;

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::backend::module::ObjModule;

pub fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/backend/writers/tests/fixtures")
}

/// All fixtures for a target suffix (`linux`, `windows`, `macos`), sorted by name.
pub fn fixtures(suffix: &str) -> Vec<(String, PathBuf)> {
    let mut v: Vec<_> = std::fs::read_dir(fixture_dir())
        .unwrap()
        .filter_map(|e| {
            let p = e.unwrap().path();
            let name = p.file_name()?.to_str()?.to_string();
            let stem = name.strip_suffix(&format!(".{suffix}.s"))?.to_string();
            Some((stem, p))
        })
        .collect();
    v.sort();
    v
}

/// Looks up an executable in `PATH` (plus a few versioned LLVM names).
pub fn tool(names: &[&str]) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for name in names {
        for dir in std::env::split_paths(&path) {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

pub fn llvm(name: &str) -> Option<PathBuf> {
    let versioned: Vec<String> = (14..=30).rev().map(|v| format!("{name}-{v}")).collect();
    let mut names = vec![name];
    names.extend(versioned.iter().map(|s| s.as_str()));
    tool(&names)
}

/// Returns the tool or reports why the test is skipped.
pub fn need(found: Option<PathBuf>, what: &str) -> Option<PathBuf> {
    if found.is_none() {
        if std::env::var_os("MEMEASM_REQUIRE_TOOLS").is_some() {
            panic!("required tool missing: {what}");
        }
        eprintln!("note: {what} not found, skipping");
    }
    found
}

/// A fresh scratch directory for one test.
pub fn scratch(test: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("memeasm-writers-{}", std::process::id())).join(test);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

pub fn run(cmd: &mut Command) -> Output {
    let out = cmd.output().unwrap_or_else(|e| panic!("cannot run {cmd:?}: {e}"));
    out
}

pub fn run_ok(cmd: &mut Command) -> Output {
    let out = run(cmd);
    assert!(
        out.status.success(),
        "{cmd:?} failed: {}\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

pub fn stdout(cmd: &mut Command) -> String {
    String::from_utf8(run_ok(cmd).stdout).unwrap()
}

/// Assembles `src` with GNU as for x86-64 ELF.
pub fn gas_elf(src: &Path, out: &Path) -> Option<Vec<u8>> {
    let gas = need(tool(&["x86_64-linux-gnu-as"]).or_else(|| host_x86_64().then(|| tool(&["as"])).flatten()), "x86-64 GNU as")?;
    run_ok(Command::new(gas).arg("-o").arg(out).arg(src));
    Some(std::fs::read(out).unwrap())
}

/// The module the real assembler should produce for `src` (via gas + [`elf_import`]).
/// The gas object is kept at [`gas_obj_path`].
pub fn module_for(src: &Path, dir: &Path) -> Option<ObjModule> {
    let obj = gas_elf(src, &gas_obj_path(src, dir))?;
    Some(elf_import::import(&obj))
}

/// `memeasm -g -O` compiled in-process: the module the frontend generates for `files` (paths
/// relative to the repository root), with debug information. The original compiler's `-g`
/// output (STABS) was broken, so there are no `-g` fixtures to compare with; the DWARF tests
/// check this module's debug information with the tools instead.
pub fn compile_with_debug(files: &[&str]) -> ObjModule {
    use crate::frontend::commands::{CompileState, OutputMode};
    use crate::frontend::{analyser, encoder, parser};
    let mut state = CompileState::new(crate::target::Target::Linux);
    state.output_mode = OutputMode::ObjectFile;
    state.use_debug_info = true;
    for f in files {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(f);
        let data = std::fs::read(&path).unwrap();
        let file = parser::parse_file(path.to_str().unwrap().as_bytes(), &data, &mut state);
        state.files.push(file);
    }
    analyser::analyse_commands(&mut state);
    assert_eq!(state.compiler_errors, 0, "{files:?}");
    encoder::encode_module(&state).unwrap()
}

pub fn gas_obj_path(src: &Path, dir: &Path) -> PathBuf {
    dir.join(format!("{}.gas.o", src.file_stem().unwrap().to_str().unwrap()))
}

pub fn host_x86_64() -> bool {
    cfg!(all(target_arch = "x86_64", target_os = "linux"))
}

/// Command that runs a Linux x86-64 executable on this host.
pub fn linux_runner(exe: &Path) -> Option<Command> {
    if host_x86_64() {
        return Some(Command::new(exe));
    }
    let qemu = need(tool(&["qemu-x86_64", "qemu-x86_64-static"]), "qemu-x86_64")?;
    let mut c = Command::new(qemu);
    c.arg(exe);
    Some(c)
}

/// Runs a program with `stdin` as input, returning (exit code, stdout).
pub fn run_with_input(mut cmd: Command, stdin: &[u8]) -> (i32, Vec<u8>) {
    use std::io::Write;
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin).unwrap();
    let out = child.wait_with_output().unwrap();
    (out.status.code().unwrap_or(-1), out.stdout)
}

pub fn write_file(path: &Path, bytes: &[u8]) {
    std::fs::write(path, bytes).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// (arguments, stdin) fed to the example programs; `None` for programs that can't run as
/// executables (no `main`).
pub fn program_input(name: &str) -> Option<(Vec<&'static str>, &'static [u8])> {
    const BF_HELLO: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
    match name {
        "brainfuck" => Some((vec![BF_HELLO], b"")),
        "reverse_string" => Some((vec![], b"Hello, World\n")),
        "is_nice" => Some((vec![], b"69\n")),
        "helloworld" | "helloworld_o69420" | "helloworld_nomartyr" | "alphabet" | "alphabet_bully" | "the_link" => {
            Some((vec![], b""))
        }
        _ => None,
    }
}

pub fn le32(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(b[off..off + 4].try_into().unwrap())
}

pub fn le64(b: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(b[off..off + 8].try_into().unwrap())
}

/// The NUL-terminated string at the start of `b`.
pub fn cstr(b: &[u8]) -> String {
    String::from_utf8_lossy(b.split(|&c| c == 0).next().unwrap()).into_owned()
}

/// A section of an ELF file as `(address, contents)`.
pub fn elf_section(b: &[u8], name: &str) -> Option<(u64, Vec<u8>)> {
    let shoff = le64(b, 0x28) as usize;
    let (entsize, num, strndx) =
        (u16::from_le_bytes([b[0x3a], b[0x3b]]) as usize, u16::from_le_bytes([b[0x3c], b[0x3d]]) as usize, u16::from_le_bytes([b[0x3e], b[0x3f]]) as usize);
    let shdr = |i: usize| shoff + i * entsize;
    let strtab = le64(b, shdr(strndx) + 0x18) as usize;
    (0..num).find_map(|i| {
        let h = shdr(i);
        if cstr(&b[strtab + le32(b, h) as usize..]) != name {
            return None;
        }
        let (addr, off, size) = (le64(b, h + 0x10), le64(b, h + 0x18) as usize, le64(b, h + 0x20) as usize);
        Some((addr, b[off..off + size].to_vec()))
    })
}
