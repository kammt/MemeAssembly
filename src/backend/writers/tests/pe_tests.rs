use std::process::Command;

use super::*;
use crate::backend::module::RelocKind;
use crate::backend::writers::write;
use crate::backend::OutputKind;
use crate::target::Target;

/// Command running a PE executable: `$MEMEASM_WINE <exe>` (e.g. a wine wrapper script).
fn wine_runner(exe: &Path) -> Option<Command> {
    let wine = need(std::env::var_os("MEMEASM_WINE").map(PathBuf::from), "MEMEASM_WINE (PE runner)")?;
    let mut c = Command::new(wine);
    c.arg(exe);
    Some(c)
}

fn objdump_p(exe: &Path) -> Option<String> {
    let objdump = need(tool(&["x86_64-w64-mingw32-objdump"]), "x86_64-w64-mingw32-objdump")?;
    let out = run_ok(Command::new(objdump).arg("-p").arg("-h").arg("-d").arg(exe));
    assert!(out.stderr.is_empty(), "objdump: {}", String::from_utf8_lossy(&out.stderr));
    Some(String::from_utf8(out.stdout).unwrap())
}

/// Headers and import tables as mingw's objdump sees them; the entry stub calls `main` and
/// the module's calls reach kernel32 through thunks.
#[test]
fn pe_executable_structure() {
    let dir = scratch("pe_structure");
    let Some(m) = module_for(&fixture_dir().join("helloworld.windows.s"), &dir) else { return };
    let exe = dir.join("helloworld.exe");
    write_file(&exe, &write(&m, Target::Windows, OutputKind::Executable).unwrap());
    let Some(p) = objdump_p(&exe) else { return };
    for want in [
        "file format pei-x86-64",
        "Magic\t\t\t020b\t(PE32+)",
        "ImageBase\t\t0000000140000000",
        "Subsystem\t\t00000003\t(Windows CUI)",
        "DLL Name: KERNEL32.dll",
        "DLL Name: msvcrt.dll",
        "GetStdHandle",
        "WriteFile",
        "ReadFile",
        "ExitProcess",
        "__getmainargs",
        "Characteristics 0x22",
        "DllCharacteristics\t00000160",
        "HIGH_ENTROPY_VA",
        "DYNAMIC_BASE",
        "NX_COMPAT",
        "Base Relocation Directory [.reloc]",
    ] {
        assert!(p.contains(want), "missing `{want}`:\n{p}");
    }
    assert!(!p.contains("relocations stripped"), "{p}");
    // Three `jmp *(%rip)` thunks (GetStdHandle, ReadFile, WriteFile) and two indirect calls
    // in the stub.
    assert_eq!(p.matches("jmp    *0x").count(), 3, "{p}");
    assert_eq!(p.matches("call   *0x").count(), 2, "{p}");
}

/// Our executables behave like the original toolchain's (mingw gcc on the same assembly).
#[test]
fn pe_executables_match_reference() {
    let dir = scratch("pe_executables_match_reference");
    let Some(gcc) = need(tool(&["x86_64-w64-mingw32-gcc"]), "x86_64-w64-mingw32-gcc") else { return };
    let mut ran = 0;
    for (name, src) in fixtures("windows") {
        let Some((args, input)) = program_input(&name) else { continue };
        if name == "brainfuck" {
            // The example reads argv from rsi (System V) even on Windows: the reference build
            // crashes as well, so there is nothing meaningful to compare.
            continue;
        }
        let Some(m) = module_for(&src, &dir) else { return };
        let exe = dir.join(format!("{name}.exe"));
        write_file(&exe, &write(&m, Target::Windows, OutputKind::Executable).unwrap());
        let reference = dir.join(format!("{name}.ref.exe"));
        run_ok(Command::new(&gcc).arg("-o").arg(&reference).arg(&src));
        let Some(mut ours) = wine_runner(&exe) else { return };
        let Some(mut theirs) = wine_runner(&reference) else { return };
        ours.args(&args);
        theirs.args(&args);
        let a = run_with_input(ours, input);
        let b = run_with_input(theirs, input);
        assert!(!b.1.is_empty() || name.contains("o69420"), "{name}: reference printed nothing");
        assert_eq!(
            (a.0, String::from_utf8_lossy(&a.1)),
            (b.0, String::from_utf8_lossy(&b.1)),
            "{name}: (exit code, stdout) differ from the reference"
        );
        ran += 1;
    }
    assert!(ran >= 5, "only {ran} programs ran");
}

/// `main(argc, argv, envp)` gets the real command line (via msvcrt's `__getmainargs`) and a
/// 16-byte aligned call.
#[test]
fn pe_executable_args_and_alignment() {
    let dir = scratch("pe_args");
    let Some(m) = module_for(&fixture_dir().join("args.windows.s"), &dir) else { return };
    let exe = dir.join("args.exe");
    write_file(&exe, &write(&m, Target::Windows, OutputKind::Executable).unwrap());
    let Some(mut c) = wine_runner(&exe) else { return };
    c.args(["first", "second arg"]);
    let (code, out) = run_with_input(c, b"");
    assert_eq!(String::from_utf8_lossy(&out), "second arg");
    assert_eq!(code, 51, "argc + 40 + (rsp & 15) at main");
}

#[test]
fn pe_hand_built_hello_world() {
    let m = samples::hello_world_windows();
    let exe = write(&m, Target::Windows, OutputKind::Executable).unwrap();
    assert_eq!(&exe[..2], b"MZ");
    let dir = scratch("pe_hand_built");
    let p = dir.join("hello.exe");
    write_file(&p, &exe);
    let Some(c) = wine_runner(&p) else { return };
    let (code, out) = run_with_input(c, b"");
    assert_eq!(String::from_utf8_lossy(&out), "Hi\n");
    assert_eq!(code, 7);
}

#[test]
fn pe_reports_unresolved_symbols() {
    let mut m = samples::hello_world_windows();
    let id = m.symbol_by_name("WriteFile").unwrap();
    m.symbols[id].name = "printf".into();
    let err = write(&m, Target::Windows, OutputKind::Executable).unwrap_err();
    assert!(err.contains("undefined reference to `printf'"), "{err}");

    let mut m = samples::hello_world_windows();
    let id = m.symbol_by_name("main").unwrap();
    m.symbols[id].name = "notmain".into();
    let err = write(&m, Target::Windows, OutputKind::Executable).unwrap_err();
    assert!(err.contains("undefined reference to `main'"), "{err}");
}

/// Every absolute address gets a DIR64 base relocation (one block per page, padded like ld),
/// and the program finds its pointers intact under Wine.
#[test]
fn pe_base_relocations() {
    let dir = scratch("pe_base_relocations");
    let Some(m) = module_for(&fixture_dir().join("pointers.windows.s"), &dir) else { return };
    let exe = dir.join("pointers.exe");
    write_file(&exe, &write(&m, Target::Windows, OutputKind::Executable).unwrap());
    let Some(p) = objdump_p(&exe) else { return };
    // .text at 0x1000: the movabs immediate; .data at 0x2000/0x3000: four pointers on the
    // first page, one on the next.
    let movabs = m.sections[0].relocs.iter().find(|r| r.kind == RelocKind::Abs64).unwrap().offset;
    let text_page = format!("reloc    0 offset {:>4x} [{:x}] DIR64", movabs, 0x1000 + movabs);
    for want in [
        "Virtual Address: 00001000 Chunk size 12 (0xc) Number of fixups 2",
        text_page.as_str(),
        "Virtual Address: 00002000 Chunk size 16 (0x10) Number of fixups 4",
        "reloc    0 offset    0 [2000] DIR64",
        "reloc    1 offset    8 [2008] DIR64",
        "reloc    2 offset   10 [2010] DIR64",
        "reloc    3 offset   20 [2020] DIR64",
        "Virtual Address: 00003000 Chunk size 12 (0xc) Number of fixups 2",
        "reloc    0 offset    0 [3000] DIR64",
        "reloc    1 offset    0 [3000] ABSOLUTE",
    ] {
        assert!(p.contains(want), "missing `{want}`:\n{p}");
    }
    assert_eq!(p.matches("DIR64").count(), 6, "{p}");
    let Some(c) = wine_runner(&exe) else { return };
    let (code, _) = run_with_input(c, b"");
    assert_eq!(code, 42);
}

/// Without absolute addresses, `.reloc` still exists (one empty block), as with ld.
#[test]
fn pe_without_absolute_addresses_has_empty_reloc_block() {
    let m = samples::hello_world_windows();
    let dir = scratch("pe_empty_reloc");
    let exe = dir.join("hello.exe");
    write_file(&exe, &write(&m, Target::Windows, OutputKind::Executable).unwrap());
    let Some(p) = objdump_p(&exe) else { return };
    assert!(p.contains("Chunk size 8 (0x8) Number of fixups 0"), "{p}");
}
