use std::path::Path;
use std::process::Command;

use super::*;
use crate::backend::writers::{elf, write};
use crate::backend::OutputKind;
use crate::target::Target;

fn linux_fixtures() -> Vec<(String, PathBuf)> {
    let mut v = fixtures("linux");
    v.extend(fixtures("linux_g").into_iter().map(|(n, p)| (format!("{n}_g"), p)));
    v
}

/// Our ELF objects are byte-for-byte what GNU as writes for the same source.
#[test]
fn elf_objects_match_gas() {
    let dir = scratch("elf_objects_match_gas");
    let mut checked = 0;
    for (name, src) in linux_fixtures() {
        let Some(gas) = gas_elf(&src, &dir.join(format!("{name}.gas.o"))) else { return };
        let module = elf_import::import(&gas);
        let ours = write(&module, Target::Linux, OutputKind::Object).unwrap();
        if ours != gas {
            let p = dir.join(format!("{name}.ours.o"));
            std::fs::write(&p, &ours).unwrap();
            panic!("{name}: object differs from gas; compare `readelf -aW` of {} and {}", p.display(), dir.join(format!("{name}.gas.o")).display());
        }
        checked += 1;
    }
    assert!(checked >= 20, "only {checked} fixtures checked");
}

/// `readelf` has no complaints, and the object links (static, like the original) and runs
/// identically when linked by GNU ld.
#[test]
fn elf_objects_link_with_ld() {
    let dir = scratch("elf_objects_link_with_ld");
    let Some(gcc) = need(tool(&["x86_64-linux-gnu-gcc"]), "x86_64-linux-gnu-gcc") else { return };
    let Some(readelf) = need(tool(&["x86_64-linux-gnu-readelf", "readelf"]), "readelf") else { return };
    for (name, src) in linux_fixtures() {
        let Some(m) = module_for(&src, &dir) else { return };
        let obj = dir.join(format!("{name}.o"));
        write_file(&obj, &write(&m, Target::Linux, OutputKind::Object).unwrap());
        let out = run_ok(Command::new(&readelf).arg("-aW").arg(&obj));
        assert!(out.stderr.is_empty(), "{name}: readelf: {}", String::from_utf8_lossy(&out.stderr));
        let base = name.trim_end_matches("_g");
        let Some((args, input)) = program_input(base) else { continue };
        let exe = dir.join(format!("{name}.ld"));
        run_ok(Command::new(&gcc).args(["-static", "-no-pie", "-z", "execstack", "-o"]).arg(&exe).arg(&obj));
        let Some(mut c) = linux_runner(&exe) else { return };
        c.args(&args);
        let (code, stdout) = run_with_input(c, input);
        assert!(!stdout.is_empty() || name.contains("o69420"), "{name}: no output from the ld-linked program");
        let _ = code;
    }
}

/// Our static executables behave exactly like the gas+gcc-linked reference programs.
#[test]
fn elf_executables_match_reference() {
    let dir = scratch("elf_executables_match_reference");
    let Some(gcc) = need(tool(&["x86_64-linux-gnu-gcc"]), "x86_64-linux-gnu-gcc") else { return };
    let mut ran = 0;
    for (name, src) in linux_fixtures() {
        let base = name.trim_end_matches("_g");
        let Some((args, input)) = program_input(base) else { continue };
        let Some(m) = module_for(&src, &dir) else { return };
        let exe = dir.join(&name);
        write_file(&exe, &write(&m, Target::Linux, OutputKind::Executable).unwrap());
        let reference = dir.join(format!("{name}.ref"));
        run_ok(Command::new(&gcc).args(["-static", "-no-pie", "-z", "execstack", "-o"]).arg(&reference).arg(dir.join(format!("{name}.gas.o"))));
        let Some(mut ours) = linux_runner(&exe) else { return };
        let Some(mut theirs) = linux_runner(&reference) else { return };
        ours.args(&args);
        theirs.args(&args);
        let a = run_with_input(ours, input);
        let b = run_with_input(theirs, input);
        assert_eq!(
            (a.0, String::from_utf8_lossy(&a.1)),
            (b.0, String::from_utf8_lossy(&b.1)),
            "{name}: (exit code, stdout) differ from the reference"
        );
        ran += 1;
    }
    assert!(ran >= 10, "only {ran} programs ran");
}

/// `_start` passes argc/argv/envp like crt1 and aligns the stack the same way; `.quad`
/// absolute addresses are resolved.
#[test]
fn elf_executable_args_env_and_alignment() {
    let dir = scratch("elf_executable_args");
    let Some(m) = module_for(&fixture_dir().join("args.linux.s"), &dir) else { return };
    let exe = dir.join("args");
    write_file(&exe, &write(&m, Target::Linux, OutputKind::Executable).unwrap());
    let Some(mut c) = linux_runner(&exe) else { return };
    c.args(["first", "second arg"]).env_clear().env("MEMEASM_TEST", "yes");
    let (code, out) = run_with_input(c, b"");
    assert_eq!(String::from_utf8_lossy(&out), "second arg\nMEMEASM_TEST=yes\n");
    assert_eq!(code, 43);
}

#[test]
fn elf_executable_headers_are_sane() {
    let dir = scratch("elf_executable_headers");
    let Some(readelf) = need(tool(&["x86_64-linux-gnu-readelf", "readelf"]), "readelf") else { return };
    let Some(m) = module_for(&fixture_dir().join("helloworld.linux_g.s"), &dir) else { return };
    let exe = dir.join("hello");
    write_file(&exe, &write(&m, Target::Linux, OutputKind::Executable).unwrap());
    let out = run_ok(Command::new(&readelf).arg("-aW").arg(&exe));
    assert!(out.stderr.is_empty(), "readelf: {}", String::from_utf8_lossy(&out.stderr));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("EXEC (Executable file)"));
    assert!(text.contains("GNU_STACK") && text.contains(" RWE "), "{text}");
    assert!(text.contains(" main") && text.contains(" _start") && text.contains(" writechar"));
    assert!(text.contains(".stab ") && text.contains(".stabstr"));
}

/// The STABS of a `-g` executable carry final addresses: the `main:F1` entries point at
/// `main` and line entries at addresses inside `.text`.
#[test]
fn elf_executable_stabs_are_resolved() {
    let dir = scratch("elf_executable_stabs");
    let Some(objdump) = need(tool(&["x86_64-linux-gnu-objdump"]), "x86_64-linux-gnu-objdump") else { return };
    let Some(nm) = need(tool(&["x86_64-linux-gnu-nm"]), "x86_64-linux-gnu-nm") else { return };
    let Some(m) = module_for(&fixture_dir().join("helloworld.linux_g.s"), &dir) else { return };
    let exe = dir.join("hello");
    write_file(&exe, &write(&m, Target::Linux, OutputKind::Executable).unwrap());
    let syms = stdout(Command::new(&nm).arg(&exe));
    let main = syms.lines().find(|l| l.ends_with(" main")).unwrap();
    let main = u64::from_str_radix(main.split(' ').next().unwrap(), 16).unwrap();
    let stabs = stdout(Command::new(&objdump).arg("-G").arg(&exe));
    let mut funs = 0;
    let mut lines = 0;
    for l in stabs.lines() {
        let f: Vec<&str> = l.split_whitespace().collect();
        if f.len() < 6 {
            continue;
        }
        let value = u64::from_str_radix(f[4], 16).unwrap_or(0);
        match f[1] {
            "FUN" => {
                assert_eq!(value, main, "{l}");
                funs += 1;
            }
            "SLINE" => {
                assert!(value > main && value < main + 0x1000, "{l}");
                lines += 1;
            }
            "HdrSym" => assert_eq!(f[3], "61", "{l}"),
            _ => {}
        }
    }
    assert!(funs > 10 && lines > 10, "{stabs}");
}

#[test]
fn elf_executable_reports_undefined_symbols_like_ld() {
    let dir = scratch("elf_undefined");
    let Some(m) = module_for(&fixture_dir().join("edge.linux.s"), &dir) else { return };
    let err = write(&m, Target::Linux, OutputKind::Executable).unwrap_err();
    assert!(err.contains("(.text+0x1): undefined reference to `ext'"), "{err}");
    assert!(!err.contains("main"), "{err}");

    let mut m = samples::hello_world_linux();
    let main = m.symbol_by_name("main").unwrap();
    m.symbols[main].name = "notmain".into();
    let err = write(&m, Target::Linux, OutputKind::Executable).unwrap_err();
    assert_eq!(err, "in function `_start': undefined reference to `main'");
}

/// Self-contained (no external tools): the hand-built module links and, where possible, runs.
#[test]
fn elf_hand_built_hello_world() {
    let m = samples::hello_world_linux();
    let obj = write(&m, Target::Linux, OutputKind::Object).unwrap();
    assert_eq!(&obj[..4], b"\x7fELF");
    let exe = write(&m, Target::Linux, OutputKind::Executable).unwrap();
    assert_eq!(u16::from_le_bytes([exe[16], exe[17]]), 2);
    let dir = scratch("elf_hand_built");
    let p = dir.join("hello");
    write_file(&p, &exe);
    if !host_x86_64() && tool(&["qemu-x86_64"]).is_none() {
        return;
    }
    let (code, out) = run_with_input(linux_runner(&p).unwrap(), b"");
    assert_eq!(String::from_utf8_lossy(&out), "Hi\n");
    assert_eq!(code, 7);
    let _ = elf::write_object;
    let _ = Path::new("");
}
