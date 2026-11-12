use std::process::Command;

use super::*;
use crate::backend::module::DebugInfo;
use crate::backend::writers::write;
use crate::backend::OutputKind;
use crate::target::Target;

/// Plain fixtures plus the `-g` ones (named `<name>_g`).
fn linux_fixtures() -> Vec<(String, PathBuf)> {
    let mut v = fixtures("linux");
    v.extend(debug_fixtures());
    v
}

fn debug_fixtures() -> Vec<(String, PathBuf)> {
    fixtures("linux_g").into_iter().map(|(n, p)| (format!("{n}_g"), p)).collect()
}

/// What `readelf` shows of a relocatable object, minus what only depends on the layout:
/// sections `(name, type, size, flags)` in any order, without the empty `.bss` and the
/// symbol/string tables; symbols `(name, type, binding, section, value)` without section
/// symbols; relocations `(section, offset, type, symbol + addend)`.
#[derive(Debug, PartialEq)]
struct ElfStructure {
    sections: Vec<String>,
    symbols: Vec<String>,
    relocs: Vec<String>,
}

fn elf_structure(readelf: &Path, obj: &Path) -> ElfStructure {
    let out = run_ok(Command::new(readelf).args(["-SW", "-sW", "-rW"]).arg(obj));
    assert!(out.stderr.is_empty(), "readelf {}: {}", obj.display(), String::from_utf8_lossy(&out.stderr));
    let text = String::from_utf8(out.stdout).unwrap();
    let mut index_names = vec![String::new()];
    let mut st = ElfStructure { sections: vec![], symbols: vec![], relocs: vec![] };
    let mut reloc_section = String::new();
    for l in text.lines() {
        let f: Vec<&str> = l.split_whitespace().collect();
        if l.trim_start().starts_with('[') && !l.contains("Nr]") {
            // [Nr] Name Type Address Off Size ES Flg Lk Inf Al (the name of section 0 is empty)
            let f: Vec<&str> = l.split(']').nth(1).unwrap().split_whitespace().collect();
            if f[0] == "NULL" {
                continue;
            }
            index_names.push(f[0].to_string());
            let flags = if f.len() == 10 { f[6] } else { "" };
            if ![".bss", ".symtab", ".strtab", ".shstrtab"].contains(&f[0]) {
                st.sections.push(format!("{} {} {} {flags}", f[0], f[1], f[4]));
            }
        } else if let Some(rest) = l.strip_prefix("Relocation section '") {
            reloc_section = rest.split('\'').next().unwrap().to_string();
        } else if f.len() >= 8 && f[0].ends_with(':') && f[0][..f[0].len() - 1].parse::<u32>().is_ok() {
            // Num: Value Size Type Bind Vis Ndx Name
            if f[3] == "SECTION" {
                continue;
            }
            let sec = f[6].parse::<usize>().map_or(f[6].to_string(), |i| index_names[i].clone());
            st.symbols.push(format!("{} {} {} {sec} {}", f[7], f[3], f[4], f[1]));
        } else if f.len() >= 5 && f[2].starts_with("R_X86_64") {
            st.relocs.push(format!("{reloc_section} {} {} {}", f[0], f[2], f[4..].join(" ")));
        }
    }
    st.sections.sort();
    st.symbols.sort();
    st
}

/// Our ELF objects have the sections, symbols, relocations and section contents GNU as writes
/// for the same source (the layout differs: no empty `.bss`, sections in index order, a section
/// symbol for every section). (The `-g` fixtures are excluded: gas writes their STABS, we write
/// DWARF.)
#[test]
fn elf_objects_match_gas() {
    let dir = scratch("elf_objects_match_gas");
    let Some(readelf) = need(tool(&["x86_64-linux-gnu-readelf", "readelf"]), "readelf") else { return };
    let mut checked = 0;
    for (name, src) in fixtures("linux") {
        let gas_path = gas_obj_path(&src, &dir);
        let Some(gas) = gas_elf(&src, &gas_path) else { return };
        let module = elf_import::import(&gas);
        assert!(module.debug.is_none());
        let ours = write(&module, Target::Linux, OutputKind::Object).unwrap();
        let ours_path = dir.join(format!("{name}.ours.o"));
        write_file(&ours_path, &ours);
        let (a, b) = (elf_structure(&readelf, &ours_path), elf_structure(&readelf, &gas_path));
        assert!(!b.sections.is_empty() && !b.symbols.is_empty());
        assert_eq!(a, b, "{name}: compare `readelf -aW` of {} and {}", ours_path.display(), gas_path.display());
        for sec in [".text", ".data"] {
            assert_eq!(elf_section(&ours, sec), elf_section(&gas, sec), "{name}: {sec} contents");
        }
        checked += 1;
    }
    assert!(checked >= 12, "only {checked} fixtures checked");
}

/// `readelf` has no complaints, and the object links (static, like the original) and runs
/// when linked by GNU ld.
#[test]
fn elf_objects_link_with_ld() {
    let dir = scratch("elf_objects_link_with_ld");
    let Some(gcc) = need(tool(&["x86_64-linux-gnu-gcc"]), "x86_64-linux-gnu-gcc") else { return };
    let Some(readelf) = need(tool(&["x86_64-linux-gnu-readelf", "readelf"]), "readelf") else { return };
    for (name, src) in linux_fixtures() {
        let Some(m) = module_for(&src, &dir) else { return };
        let obj = dir.join(format!("{name}.o"));
        write_file(&obj, &write(&m, Target::Linux, OutputKind::Object).unwrap());
        let out = run_ok(Command::new(&readelf).arg("-aW").arg("--debug-dump").arg(&obj));
        assert!(out.stderr.is_empty(), "{name}: readelf: {}", String::from_utf8_lossy(&out.stderr));
        let base = name.trim_end_matches("_g");
        let Some((args, input)) = program_input(base) else { continue };
        let exe = dir.join(format!("{name}.ld"));
        run_ok(Command::new(&gcc).args(["-static", "-no-pie", "-z", "execstack", "-o"]).arg(&exe).arg(&obj));
        let Some(mut c) = linux_runner(&exe) else { return };
        c.args(&args);
        let (_, stdout) = run_with_input(c, input);
        assert!(!stdout.is_empty() || name.contains("o69420"), "{name}: no output from the ld-linked program");
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
        run_ok(Command::new(&gcc).args(["-static", "-no-pie", "-z", "execstack", "-o"]).arg(&reference).arg(gas_obj_path(&src, &dir)));
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
    for s in [".debug_info", ".debug_abbrev", ".debug_line "] {
        assert!(text.contains(s), "missing {s}: {text}");
    }
    assert!(!text.contains(".rela"), "{text}");
    // Separate page-aligned segments like `ld -z separate-code`: headers R, code R+X, data RW.
    let loads: Vec<(u64, u64, String)> = text
        .lines()
        .filter(|l| l.trim_start().starts_with("LOAD"))
        .map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let hex = |s: &str| u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap();
            (hex(f[1]), hex(f[2]), f[6..f.len() - 1].join(" "))
        })
        .collect();
    let flags: Vec<&str> = loads.iter().map(|l| l.2.as_str()).collect();
    assert_eq!(flags, ["R", "R E", "RW"], "{text}");
    assert_eq!((loads[0].0, loads[0].1), (0, 0x400000), "{text}");
    for (off, vaddr, _) in &loads {
        assert!(off % 0x1000 == 0 && vaddr % 0x1000 == 0, "{text}");
    }
}

/// `(line, address)` rows of the decoded line table (`objdump --dwarf=decodedline`), without
/// the end-of-sequence row.
fn decoded_lines(objdump: &Path, file: &Path) -> Vec<(u32, u64)> {
    let out = run_ok(Command::new(objdump).arg("--dwarf=decodedline").arg(file));
    assert!(out.stderr.is_empty(), "objdump: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let line = f.get(1)?.parse().ok()?;
            let addr = u64::from_str_radix(f.get(2)?.trim_start_matches("0x"), 16).ok()?;
            Some((line, addr))
        })
        .collect()
}

fn expected_lines(d: &DebugInfo, base: u64) -> Vec<(u32, u64)> {
    d.lines.iter().map(|r| (r.line, base + r.offset)).collect()
}

/// `llvm-dwarfdump --verify` accepts our DWARF and the decoded line table is the module's,
/// both in objects (relocations applied by the tools) and in objects linked by GNU ld.
#[test]
fn elf_dwarf_objects_verify_and_decode() {
    let dir = scratch("elf_dwarf_objects");
    let Some(dwarfdump) = need(llvm("llvm-dwarfdump"), "llvm-dwarfdump") else { return };
    let Some(objdump) = need(tool(&["x86_64-linux-gnu-objdump"]), "x86_64-linux-gnu-objdump") else { return };
    let Some(gcc) = need(tool(&["x86_64-linux-gnu-gcc"]), "x86_64-linux-gnu-gcc") else { return };
    let Some(nm) = need(tool(&["x86_64-linux-gnu-nm"]), "x86_64-linux-gnu-nm") else { return };
    let mut checked = 0;
    for (name, src) in debug_fixtures() {
        let Some(m) = module_for(&src, &dir) else { return };
        let d = m.debug.as_ref().expect("-g fixture has debug info");
        assert!(d.lines.len() > 3 && !d.functions.is_empty(), "{name}: {d:?}");
        let obj = dir.join(format!("{name}.o"));
        write_file(&obj, &write(&m, Target::Linux, OutputKind::Object).unwrap());
        run_ok(Command::new(&dwarfdump).arg("--verify").arg(&obj));
        assert_eq!(decoded_lines(&objdump, &obj), expected_lines(d, 0), "{name}: object line table");

        // Linked by ld: .text starts after crt code, so look up the module's base via main.
        let exe = dir.join(format!("{name}.ld"));
        run_ok(Command::new(&gcc).args(["-static", "-no-pie", "-z", "execstack", "-nostartfiles", "-nostdlib", "-e", "0", "-o"]).arg(&exe).arg(&obj));
        run_ok(Command::new(&dwarfdump).arg("--verify").arg(&exe));
        let f = &d.functions[0];
        let syms = stdout(Command::new(&nm).arg(&exe));
        let addr = syms.lines().find(|l| l.ends_with(&format!(" {}", f.name))).map(|l| u64::from_str_radix(&l[..16], 16).unwrap());
        if let Some(addr) = addr {
            assert_eq!(decoded_lines(&objdump, &exe), expected_lines(d, addr - f.low), "{name}: ld-linked line table");
        }
        checked += 1;
    }
    assert!(checked >= 9, "only {checked} fixtures checked");
}

/// Our executables carry DWARF with final addresses: the line table and every subprogram's
/// `DW_AT_low_pc` match the symbol table.
#[test]
fn elf_dwarf_executables_are_resolved() {
    let dir = scratch("elf_dwarf_executables");
    let Some(dwarfdump) = need(llvm("llvm-dwarfdump"), "llvm-dwarfdump") else { return };
    let Some(objdump) = need(tool(&["x86_64-linux-gnu-objdump"]), "x86_64-linux-gnu-objdump") else { return };
    let Some(nm) = need(tool(&["x86_64-linux-gnu-nm"]), "x86_64-linux-gnu-nm") else { return };
    for (name, src) in debug_fixtures() {
        let Some(m) = module_for(&src, &dir) else { return };
        let d = m.debug.as_ref().unwrap();
        let exe = dir.join(&name);
        let bytes = match write(&m, Target::Linux, OutputKind::Executable) {
            Ok(b) => b,
            Err(e) if e.contains("undefined reference to `main'") => continue, // library-style
            Err(e) => panic!("{name}: {e}"),
        };
        write_file(&exe, &bytes);
        run_ok(Command::new(&dwarfdump).arg("--verify").arg(&exe));
        let syms = stdout(Command::new(&nm).arg(&exe));
        let addr_of = |sym: &str| {
            syms.lines()
                .find(|l| l.ends_with(&format!(" {sym}")))
                .map(|l| u64::from_str_radix(&l[..16], 16).unwrap())
                .unwrap_or_else(|| panic!("{name}: no symbol {sym}"))
        };
        let f0 = &d.functions[0];
        let text_addr = addr_of(&f0.name) - f0.low;
        assert_eq!(decoded_lines(&objdump, &exe), expected_lines(d, text_addr), "{name}");
        let info = stdout(Command::new(&dwarfdump).arg("--debug-info").arg(&exe));
        for f in &d.functions {
            let want = format!("DW_AT_low_pc\t({:#018x})", text_addr + f.low);
            let entry = info.split("DW_TAG_subprogram").find(|e| e.contains(&format!("(\"{}\")", f.name))).unwrap_or_else(|| panic!("{name}: no DIE for {}", f.name));
            assert!(entry.contains(&want), "{name}: {} should have {want}: {entry}", f.name);
        }
    }
}

/// Runs `exe` under qemu's gdbstub and gdb with a breakpoint at `location`; returns gdb's
/// output after `continue` + `bt`.
fn gdb_session(dir: &Path, exe: &Path, location: &str) -> Option<String> {
    let gdb = need(tool(&["gdb-multiarch"]).or_else(|| host_x86_64().then(|| tool(&["gdb"])).flatten()), "gdb-multiarch")?;
    let qemu = need(tool(&["qemu-x86_64"]), "qemu-x86_64")?;
    let _ = dir;
    let port = 20000 + (std::process::id() + location.len() as u32 * 7) % 20000;
    let mut q = Command::new(qemu).arg("-g").arg(port.to_string()).arg(exe).stdin(Stdio::null()).stdout(Stdio::null()).spawn().unwrap();
    let mut cmd = Command::new(gdb);
    cmd.args(["-nx", "-batch"]).arg(exe);
    for c in [format!("target remote :{port}"), format!("break {location}"), "continue".into(), "bt".into(), "kill".into()] {
        cmd.arg("-ex").arg(c);
    }
    let out = run(&mut cmd);
    let _ = q.kill();
    let _ = q.wait();
    Some(format!("{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

/// gdb (against qemu's gdbstub) can set a breakpoint by `file:line` and shows the MemeAssembly
/// function names and source lines in the backtrace.
#[test]
fn elf_dwarf_gdb_breakpoint() {
    let dir = scratch("elf_dwarf_gdb");
    // A real program: break on its third source line.
    let src = fixture_dir().join("helloworld.linux_g.s");
    let Some(m) = module_for(&src, &dir) else { return };
    let d = m.debug.as_ref().unwrap();
    let row = &d.lines[2];
    let exe = dir.join("helloworld");
    write_file(&exe, &write(&m, Target::Linux, OutputKind::Executable).unwrap());
    let Some(text) = gdb_session(&dir, &exe, &format!("HelloWorld.memeasm:{}", row.line)) else { return };
    let want = format!("Breakpoint 1, main () at {}:{}", String::from_utf8_lossy(&d.files[0]), row.line);
    assert!(text.contains(&want), "expected `{want}`:\n{text}");
    assert!(text.contains("#0  main () at "), "{text}");

    // Two functions: the backtrace from `writechar` shows `main` as the caller.
    let m = samples::hello_world_linux_debug();
    let exe = dir.join("hand_built");
    write_file(&exe, &write(&m, Target::Linux, OutputKind::Executable).unwrap());
    let Some(text) = gdb_session(&dir, &exe, "hello.memeasm:7") else { return };
    assert!(text.contains("Breakpoint 1, writechar () at /src/hello.memeasm:7"), "{text}");
    assert!(text.contains("#0  writechar () at /src/hello.memeasm:7"), "{text}");
    assert!(text.contains(" in main () at /src/hello.memeasm:"), "{text}");
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
}

/// `_start` calls `main` through rax, which holds `main`'s address on entry (like crt1).
#[test]
fn elf_start_passes_main_in_rax() {
    use crate::backend::emitter::x86::{branch, BranchKind};
    use crate::backend::emitter::{Cond, Op, Operand};
    use crate::backend::writers::{reg, rip};
    let mut b = samples::Builder::new();
    b.global("main");
    b.label(0, "main");
    b.ins(Op::Lea, &[reg("rcx"), rip(None, "main")]);
    b.ins(Op::Cmp, &[reg("rax"), reg("rcx")]);
    b.ins(Op::Mov, &[reg("eax"), Operand::Imm(0)]); // (5 bytes, keeps the flags)
    b.code(&branch(BranchKind::Jcc(Cond::E), false, 5)); // je over the next mov
    b.ins(Op::Mov, &[reg("eax"), Operand::Imm(1)]);
    b.ins(Op::Ret, &[]);
    let p = scratch("elf_start_rax").join("rax");
    write_file(&p, &write(&b.finish(), Target::Linux, OutputKind::Executable).unwrap());
    if !host_x86_64() && tool(&["qemu-x86_64"]).is_none() {
        return;
    }
    assert_eq!(run_with_input(linux_runner(&p).unwrap(), b"").0, 0);
}

/// Calls to a global function defined in the same section stay relocations in objects
/// (`R_X86_64_PLT32` against the symbol, like gas) and are resolved in executables.
#[test]
fn elf_calls_to_defined_globals() {
    let m = samples::hello_world_linux_global_calls();
    let dir = scratch("elf_global_calls");
    let (po, pe) = (dir.join("hello.o"), dir.join("hello"));
    write_file(&po, &write(&m, Target::Linux, OutputKind::Object).unwrap());
    write_file(&pe, &write(&m, Target::Linux, OutputKind::Executable).unwrap());
    if let Some(readelf) = need(tool(&["x86_64-linux-gnu-readelf", "readelf"]), "readelf") {
        let relocs = stdout(Command::new(&readelf).arg("-rW").arg(&po));
        let plt = relocs.lines().filter(|l| l.contains("R_X86_64_PLT32") && l.contains(" writechar - 4")).count();
        assert_eq!(plt, 3, "{relocs}");
        if let Some(gcc) = need(tool(&["x86_64-linux-gnu-gcc"]), "x86_64-linux-gnu-gcc") {
            let linked = dir.join("hello.gcc");
            run_ok(Command::new(gcc).args(["-static", "-no-pie", "-z", "execstack", "-o"]).arg(&linked).arg(&po));
            if host_x86_64() || tool(&["qemu-x86_64"]).is_some() {
                let (code, out) = run_with_input(linux_runner(&linked).unwrap(), b"");
                assert_eq!((code, String::from_utf8_lossy(&out).as_ref()), (7, "Hi\n"));
            }
        }
    }
    if !host_x86_64() && tool(&["qemu-x86_64"]).is_none() {
        return;
    }
    let (code, out) = run_with_input(linux_runner(&pe).unwrap(), b"");
    assert_eq!(String::from_utf8_lossy(&out), "Hi\n");
    assert_eq!(code, 7);
}

/// Hand-built debug info: the writer accepts it, and with the tools present it verifies.
#[test]
fn elf_hand_built_debug_info() {
    let m = samples::hello_world_linux_debug();
    let obj = write(&m, Target::Linux, OutputKind::Object).unwrap();
    let exe = write(&m, Target::Linux, OutputKind::Executable).unwrap();
    let dir = scratch("elf_hand_built_debug");
    let (po, pe) = (dir.join("hello.o"), dir.join("hello"));
    write_file(&po, &obj);
    write_file(&pe, &exe);
    let Some(dwarfdump) = need(llvm("llvm-dwarfdump"), "llvm-dwarfdump") else { return };
    for p in [&po, &pe] {
        run_ok(Command::new(&dwarfdump).arg("--verify").arg(p));
        let info = stdout(Command::new(&dwarfdump).arg("--debug-info").arg(p));
        for want in ["DW_LANG_Mips_Assembler", "(\"main\")", "(\"writechar\")", "DW_AT_decl_line\t(7)", "(\"/src/hello.memeasm\")", "(\"/src\")", "(\"hand-built\")"] {
            assert!(info.contains(want), "{}: missing {want}: {info}", p.display());
        }
    }
}
