//! Mach-O: objects are checked against the LLVM assembler (`llvm-mc`, what Apple's `as` is)
//! and executables against `ld64.lld`. Nothing here can run a Mach-O program, so the code and
//! data of linked images are compared with GNU ld's output for the GNU as object at the same
//! addresses (byte for byte), and our executables with ld64.lld's (relocated fields compared by
//! where they point), plus entry point and rebased pointers.
use std::collections::BTreeSet;
use std::process::Command;

use super::*;
use crate::backend::module::RelocKind;
use crate::backend::writers::write;
use crate::backend::OutputKind;
use crate::target::Target;

const LC_MAIN: u32 = 0x8000_0028;

/// The parts of a Mach-O file the tests look at.
struct MachO {
    filetype: u32,
    flags: u32,
    cmds: Vec<u32>,
    sections: Vec<Sect>,
    entryoff: Option<u64>,
}

#[derive(Debug)]
struct Sect {
    name: String,
    addr: u64,
    size: u64,
    offset: u64,
}

fn u32_at(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(b[off..off + 4].try_into().unwrap())
}

fn u64_at(b: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(b[off..off + 8].try_into().unwrap())
}

fn cstr(b: &[u8]) -> String {
    String::from_utf8_lossy(b.split(|&c| c == 0).next().unwrap()).into_owned()
}

fn parse(b: &[u8]) -> MachO {
    assert_eq!(u32_at(b, 0), 0xfeed_facf, "not a 64-bit Mach-O file");
    assert_eq!(u32_at(b, 4), 0x0100_0007, "not x86-64");
    let mut m = MachO { filetype: u32_at(b, 12), flags: u32_at(b, 24), cmds: vec![], sections: vec![], entryoff: None };
    let mut off = 32;
    for _ in 0..u32_at(b, 16) {
        let (cmd, size) = (u32_at(b, off), u32_at(b, off + 4) as usize);
        m.cmds.push(cmd);
        if cmd == 0x19 {
            for i in 0..u32_at(b, off + 64) as usize {
                let s = off + 72 + 80 * i;
                m.sections.push(Sect {
                    name: cstr(&b[s..s + 16]),
                    addr: u64_at(b, s + 32),
                    size: u64_at(b, s + 40),
                    offset: u32_at(b, s + 48) as u64,
                });
            }
        } else if cmd == LC_MAIN {
            m.entryoff = Some(u64_at(b, off + 8));
        }
        off += size;
    }
    m
}

impl MachO {
    fn section(&self, name: &str) -> Option<&Sect> {
        self.sections.iter().find(|s| s.name == name)
    }

    fn contents<'a>(&self, b: &'a [u8], name: &str) -> &'a [u8] {
        self.section(name).map_or(&[], |s| &b[s.offset as usize..(s.offset + s.size) as usize])
    }

    /// `(section name, offset)` of an address (section ends included: labels at the end).
    fn locate(&self, addr: u64) -> Option<(String, u64)> {
        self.sections.iter().find(|s| addr >= s.addr && addr <= s.addr + s.size).map(|s| (s.name.clone(), addr - s.addr))
    }

    /// Offset of the entry point in `__text`.
    fn entry_in_text(&self) -> u64 {
        let text = self.section("__text").unwrap();
        self.entryoff.expect("LC_MAIN") - text.offset
    }
}

/// Assembles `src` with the LLVM assembler for macOS. With `l_prefix`, `.L` labels are renamed
/// to `L…`, which is what makes a label assembler-local on Darwin (`.L` is a normal symbol
/// there, but a temporary one for GNU as, which produced the module).
fn llvm_mc(src: &Path, out: &Path, l_prefix: bool) -> Option<Vec<u8>> {
    let mc = need(llvm("llvm-mc"), "llvm-mc")?;
    let mut text = std::fs::read_to_string(src).unwrap();
    if l_prefix {
        let mut renamed = String::new();
        let mut prev = ' ';
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '.' && !(prev.is_alphanumeric() || prev == '_' || prev == '.') && chars.peek() == Some(&'L') {
                prev = c;
                continue;
            }
            renamed.push(c);
            prev = c;
        }
        text = renamed;
    }
    let copy = out.with_extension("mc.s");
    std::fs::write(&copy, text).unwrap();
    run_ok(Command::new(mc).args(["-triple", "x86_64-apple-macos10.14", "-filetype=obj", "-o"]).arg(out).arg(&copy));
    Some(std::fs::read(out).unwrap())
}

/// Links objects into an executable with `ld64.lld` (no libSystem needed: the programs only
/// make system calls).
fn ld64(objs: &[&Path], out: &Path, extra: &[&str]) -> Option<Vec<u8>> {
    let ld = need(llvm("ld64.lld"), "ld64.lld")?;
    let o = run_ok(
        Command::new(ld)
            .args(["-arch", "x86_64", "-platform_version", "macos", "10.14", "10.14", "-o"])
            .arg(out)
            .args(extra)
            .args(objs),
    );
    assert!(o.stderr.is_empty(), "ld64.lld: {}", String::from_utf8_lossy(&o.stderr));
    Some(std::fs::read(out).unwrap())
}

/// `llvm-objdump --macho <flags>` output; stderr (warnings about malformed input) must be empty.
fn objdump(path: &Path, flags: &[&str]) -> Option<String> {
    let objdump = need(llvm("llvm-objdump"), "llvm-objdump")?;
    let o = run_ok(Command::new(objdump).arg("--macho").args(flags).arg(path));
    assert!(o.stderr.is_empty(), "llvm-objdump {flags:?}: {}", String::from_utf8_lossy(&o.stderr));
    Some(String::from_utf8(o.stdout).unwrap())
}

/// Rebased pointer locations as `(section, offset in section)`.
fn rebases(path: &Path, macho: &MachO) -> Option<BTreeSet<(String, u64)>> {
    let out = objdump(path, &["--rebase"])?;
    Some(
        out.lines()
            .filter_map(|l| {
                let cols: Vec<&str> = l.split_whitespace().collect();
                let addr = u64::from_str_radix(cols.get(2)?.strip_prefix("0x")?, 16).ok()?;
                macho.locate(addr)
            })
            .collect(),
    )
}

/// Compares `__text`/`__data` of two executables linked from `m`: equal bytes outside the
/// relocated fields, and every relocated field pointing at the same place relative to the
/// sections.
fn same_linked_contents(m: &ObjModule, a: (&[u8], &MachO), b: (&[u8], &MachO)) -> Result<(), String> {
    for (si, name) in ["__text", "__data"].into_iter().enumerate() {
        let sec = &m.sections[si];
        let (ca, cb) = (a.1.contents(a.0, name), b.1.contents(b.0, name));
        if ca.len() != sec.data.len() || cb.len() != sec.data.len() {
            return Err(format!("{name}: sizes {} and {}, module has {}", ca.len(), cb.len(), sec.data.len()));
        }
        let mut field = vec![false; sec.data.len()];
        for r in &sec.relocs {
            let n = if r.kind == RelocKind::Abs64 { 8 } else { 4 };
            field[r.offset as usize..r.offset as usize + n].iter_mut().for_each(|f| *f = true);
            let target = |c: &[u8], mo: &MachO| {
                let o = r.offset as usize;
                let p = mo.section(name).unwrap().addr + r.offset;
                let s = match r.kind {
                    RelocKind::Abs64 => u64_at(c, o).wrapping_sub(r.addend as u64),
                    _ => (i32::from_le_bytes(c[o..o + 4].try_into().unwrap()) as i64 - r.addend + p as i64) as u64,
                };
                mo.locate(s)
            };
            let (ta, tb) = (target(ca, a.1), target(cb, b.1));
            if ta.is_none() || ta != tb {
                return Err(format!(
                    "{name}+{:#x} ({:?} against `{}'): resolves to {ta:?} vs {tb:?}",
                    r.offset, r.kind, m.symbols[r.symbol].name
                ));
            }
        }
        if let Some(i) = (0..ca.len()).find(|&i| !field[i] && ca[i] != cb[i]) {
            return Err(format!("{name}+{i:#x}: {:#04x} vs {:#04x}", ca[i], cb[i]));
        }
    }
    Ok(())
}

/// `.text`/`.data` of an ELF file as `(address, contents)`.
fn elf_section(b: &[u8], name: &str) -> Option<(u64, Vec<u8>)> {
    let shoff = u64_at(b, 0x28) as usize;
    let (entsize, num, strndx) =
        (u16::from_le_bytes([b[0x3a], b[0x3b]]) as usize, u16::from_le_bytes([b[0x3c], b[0x3d]]) as usize, u16::from_le_bytes([b[0x3e], b[0x3f]]) as usize);
    let shdr = |i: usize| shoff + i * entsize;
    let strtab = u64_at(b, shdr(strndx) + 0x18) as usize;
    (0..num).find_map(|i| {
        let h = shdr(i);
        if cstr(&b[strtab + u32_at(b, h) as usize..]) != name {
            return None;
        }
        let (addr, off, size) = (u64_at(b, h + 0x10), u64_at(b, h + 0x18) as usize, u64_at(b, h + 0x20) as usize);
        Some((addr, b[off..off + size].to_vec()))
    })
}

/// Ground truth independent of everything Mach-O: GNU ld links the GNU as object of the same
/// source with `.text`/`.data` at the addresses the Mach-O image uses. With the same section
/// addresses, the code and data must be byte for byte the same.
fn gnu_ld_reference(gas_obj: &Path, macho: &MachO, out: &Path) -> Option<[Vec<u8>; 2]> {
    let ld = need(tool(&["x86_64-linux-gnu-ld"]).or_else(|| host_x86_64().then(|| tool(&["ld"])).flatten()), "x86-64 GNU ld")?;
    let mut cmd = Command::new(ld);
    cmd.args(["-e", "0", "-z", "noexecstack", "-o"]).arg(out);
    for (sect, elf) in [("__text", ".text"), ("__data", ".data")] {
        if let Some(s) = macho.section(sect) {
            cmd.arg(format!("--section-start={elf}={:#x}", s.addr));
        }
    }
    run_ok(cmd.arg(gas_obj));
    let b = std::fs::read(out).unwrap();
    Some([".text", ".data"].map(|n| elf_section(&b, n).map(|s| s.1).unwrap_or_default()))
}

/// Asserts that the image's `__text`/`__data` equal GNU ld's output at the same addresses.
fn assert_matches_gnu_ld(name: &str, image: &[u8], image_path: &Path, gas_obj: &Path) -> Option<()> {
    let p = parse(image);
    let elf = image_path.with_extension("gnu-ld");
    let reference = gnu_ld_reference(gas_obj, &p, &elf)?;
    for (i, sect) in ["__text", "__data"].into_iter().enumerate() {
        assert!(
            p.contents(image, sect) == reference[i].as_slice(),
            "{name}: {sect} differs from GNU ld's; compare {} and {}",
            image_path.display(),
            elf.display()
        );
    }
    Some(())
}

/// Our objects, linked by ld64.lld, give exactly the code and data GNU ld makes of GNU as's
/// ELF object at the same addresses, with the entry point at `_main` and every absolute
/// pointer rebased.
#[test]
fn macho_objects_link_like_gnu_toolchain() {
    let dir = scratch("macho_objects_link_like_gnu_toolchain");
    let mut checked = 0;
    for (name, src) in fixtures("macos") {
        if name == "edge" {
            continue; // undefined symbols; see macho_edge_relocations_match_llvm_mc
        }
        let Some(m) = module_for(&src, &dir) else { return };
        if m.symbol_by_name("_main").is_none() {
            continue; // not a program
        }
        let obj = dir.join(format!("{name}.o"));
        write_file(&obj, &write(&m, Target::MacOS, OutputKind::Object).unwrap());
        let exe = dir.join(format!("{name}.lld"));
        let Some(image) = ld64(&[&obj], &exe, &[]) else { return };
        let Some(()) = assert_matches_gnu_ld(&name, &image, &exe, &gas_obj_path(&src, &dir)) else { return };
        let p = parse(&image);
        let main = m.symbols[m.symbol_by_name("_main").unwrap()].definition.unwrap().1;
        assert_eq!(p.entry_in_text(), main, "{name}: entry point");
        let expected: BTreeSet<(String, u64)> =
            m.sections[1].relocs.iter().filter(|r| r.kind == RelocKind::Abs64).map(|r| ("__data".to_string(), r.offset)).collect();
        let Some(rebased) = rebases(&exe, &p) else { return };
        assert_eq!(rebased, expected, "{name}: rebases");
        checked += 1;
    }
    assert!(checked >= 7, "only {checked} fixtures checked");
}

/// Same header, load commands, section names and symbols as llvm-mc writes once labels are
/// made assembler-local the Darwin way (`L` prefix). Sizes may differ: llvm-mc picks other
/// instruction encodings than GNU as in places. llvm-readobj accepts everything.
#[test]
fn macho_objects_are_shaped_like_llvm_mc() {
    let dir = scratch("macho_objects_are_shaped_like_llvm_mc");
    let mut checked = 0;
    for (name, src) in fixtures("macos") {
        if name == "brainfuck" {
            // Labels like `.L'+'Wins_0` are fine for llvm-mc only while they start with a dot.
            continue;
        }
        let Some(m) = module_for(&src, &dir) else { return };
        let ours_path = dir.join(format!("{name}.ours.o"));
        let ours = write(&m, Target::MacOS, OutputKind::Object).unwrap();
        write_file(&ours_path, &ours);
        let theirs_path = dir.join(format!("{name}.mc.o"));
        let Some(theirs) = llvm_mc(&src, &theirs_path, true) else { return };
        // Header: magic, CPU, file type, number of commands; flags.
        assert_eq!(ours[..20], theirs[..20], "{name}: mach header");
        assert_eq!(ours[24..32], theirs[24..32], "{name}: mach header flags");
        let (a, b) = (parse(&ours), parse(&theirs));
        assert_eq!(a.cmds, b.cmds, "{name}: load commands");
        let names = |p: &MachO| p.sections.iter().map(|s| s.name.clone()).collect::<Vec<_>>();
        assert_eq!(names(&a), names(&b), "{name}: sections");
        let syms = |p: &Path| -> Option<Vec<String>> {
            let out = objdump(p, &["--syms"])?;
            // Drop the value column, keep binding/type/section/name.
            Some(out.lines().skip(2).map(|l| l.split_whitespace().skip(1).collect::<Vec<_>>().join(" ")).collect())
        };
        let (Some(sa), Some(sb)) = (syms(&ours_path), syms(&theirs_path)) else { return };
        assert_eq!(sa, sb, "{name}: symbols");
        let Some(dis) = objdump(&ours_path, &["-d", "-r", "--private-headers"]) else { return };
        assert!(dis.contains("MH_MAGIC_64  X86_64        ALL  0x00      OBJECT"), "{name}:\n{dis}");
        let Some(readobj) = need(llvm("llvm-readobj"), "llvm-readobj") else { return };
        let o = run_ok(Command::new(readobj).args(["--macho-dysymtab", "--macho-version-min", "-r", "-s", "-S"]).arg(&ours_path));
        assert!(o.stderr.is_empty(), "{name}: llvm-readobj: {}", String::from_utf8_lossy(&o.stderr));
        checked += 1;
    }
    assert!(checked >= 10, "only {checked} fixtures checked");
}
/// Relocation records for the edge cases, as llvm-mc writes them for the `L`-prefixed source:
/// those of llvm-mc that refer to symbols in another section or undefined symbols (which the
/// module keeps too) must appear identically in ours.
#[test]
fn macho_edge_relocations_match_llvm_mc() {
    let dir = scratch("macho_edge_relocations");
    let src = fixture_dir().join("edge.macos.s");
    let Some(mut m) = module_for(&src, &dir) else { return };
    // GNU as uses R_X86_64_PC32 for calls to local labels; the compiler's emitter marks every
    // call/jmp operand as a branch, which is what makes it a BRANCH relocation.
    let text = &mut m.sections[0];
    for r in &mut text.relocs {
        if r.kind == RelocKind::Pc32 && matches!(text.data[r.offset as usize - 1], 0xe8 | 0xe9) {
            r.kind = RelocKind::Branch32;
        }
    }
    let ours_path = dir.join("edge.ours.o");
    write_file(&ours_path, &write(&m, Target::MacOS, OutputKind::Object).unwrap());
    let theirs_path = dir.join("edge.mc.o");
    let Some(_) = llvm_mc(&src, &theirs_path, true) else { return };
    let Some(readobj) = need(llvm("llvm-readobj"), "llvm-readobj") else { return };
    let relocs = |p: &Path| -> Vec<String> {
        let out = String::from_utf8(run_ok(Command::new(&readobj).args(["-r", "--expand-relocs"]).arg(p)).stdout).unwrap();
        out.split("Relocation {").skip(1).map(|r| r.split_whitespace().collect::<Vec<_>>().join(" ")).collect()
    };
    let (ours, theirs) = (relocs(&ours_path), relocs(&theirs_path));
    // llvm-mc keeps relocations against `_main`, `local_fn` within __text: drop those.
    let kept: Vec<&String> = theirs.iter().filter(|r| !(r.contains("Symbol: local_fn") || r.contains("Symbol: _main") && r.contains("PCRel: 1"))).collect();
    let ours_refs: Vec<&String> = ours.iter().collect();
    assert_eq!(ours_refs, kept, "relocations differ");
    // And the bytes those relocations apply to agree.
    let (a, b) = (std::fs::read(&ours_path).unwrap(), std::fs::read(&theirs_path).unwrap());
    let (pa, pb) = (parse(&a), parse(&b));
    assert_eq!(pa.contents(&a, "__data"), pb.contents(&b, "__data"));
}

/// Executables are what ld64.lld makes of our object: same code and data (relocated fields
/// compared by where they point), same entry point, same rebased pointers. At our own
/// addresses, code and data are exactly what GNU ld makes of the GNU as object.
#[test]
fn macho_executables_match_ld64_lld() {
    let dir = scratch("macho_executables_match_ld64_lld");
    let mut checked = 0;
    for (name, src) in fixtures("macos") {
        if name == "edge" {
            continue; // undefined symbols
        }
        let Some(m) = module_for(&src, &dir) else { return };
        if m.symbol_by_name("_main").is_none() {
            continue; // not a program
        }
        let obj = dir.join(format!("{name}.o"));
        write_file(&obj, &write(&m, Target::MacOS, OutputKind::Object).unwrap());
        let ours_path = dir.join(format!("{name}.ours"));
        let ours = write(&m, Target::MacOS, OutputKind::Executable).unwrap();
        write_file(&ours_path, &ours);
        let lld_path = dir.join(format!("{name}.lld"));
        let Some(lld) = ld64(&[&obj], &lld_path, &[]) else { return };
        let (a, b) = (parse(&ours), parse(&lld));
        assert_eq!(a.filetype, 2);
        if let Err(e) = same_linked_contents(&m, (&ours, &a), (&lld, &b)) {
            panic!("{name}: {e}; compare {} and {}", ours_path.display(), lld_path.display());
        }
        let Some(()) = assert_matches_gnu_ld(&name, &ours, &ours_path, &gas_obj_path(&src, &dir)) else { return };
        assert_eq!(a.entry_in_text(), b.entry_in_text(), "{name}: entry point");
        let (Some(ra), Some(rb)) = (rebases(&ours_path, &a), rebases(&lld_path, &b)) else { return };
        assert_eq!(ra, rb, "{name}: rebases");
        if name == "pointers" {
            assert_eq!(ra.len(), 4, "{ra:?}");
        }
        let starts = |p: &Path, mo: &MachO| -> Option<Vec<u64>> {
            let text = mo.section("__text").unwrap().addr;
            let out = objdump(p, &["--function-starts"])?;
            Some(out.lines().filter_map(|l| u64::from_str_radix(l.split_whitespace().next()?, 16).ok()).map(|a| a - text).collect())
        };
        let (Some(fa), Some(fb)) = (starts(&ours_path, &a), starts(&lld_path, &b)) else { return };
        assert!(!fa.is_empty());
        assert_eq!(fa, fb, "{name}: function starts");
        let Some(h) = objdump(&ours_path, &["--all-headers", "--exports-trie", "-d"]) else { return };
        for want in [
            "MH_MAGIC_64  X86_64        ALL LIB64     EXECUTE",
            "NOUNDEFS DYLDLINK TWOLEVEL PIE",
            "segname __PAGEZERO",
            "cmd LC_DYLD_INFO_ONLY",
            "cmd LC_MAIN",
            "name /usr/lib/dyld (offset 12)",
            "name /usr/lib/libSystem.B.dylib (offset 24)",
            "cmd LC_UUID",
            "platform macos",
            "minos 10.14",
            "sdk 15.0",
            "cmd LC_FUNCTION_STARTS",
            "cmd LC_DATA_IN_CODE",
            "0x100000000  __mh_execute_header",
            "_main:",
        ] {
            assert!(h.contains(want), "{name}: missing `{want}`:\n{h}");
        }
        checked += 1;
    }
    assert!(checked >= 7, "only {checked} fixtures checked");
}

/// No tools needed: the hand-built module gives sane objects and executables.
#[test]
fn macho_hand_built_hello_world() {
    let m = samples::hello_world_macos();
    let obj = write(&m, Target::MacOS, OutputKind::Object).unwrap();
    let o = parse(&obj);
    assert_eq!((o.filetype, o.flags), (1, 0));
    assert_eq!(o.cmds, [0x19, 0x32, 0x2, 0xb]);
    assert_eq!(o.section("__text").unwrap().size, m.sections[0].data.len() as u64);

    let exe = write(&m, Target::MacOS, OutputKind::Executable).unwrap();
    let e = parse(&exe);
    assert_eq!(e.filetype, 2);
    assert_eq!(e.flags, 0x1 | 0x4 | 0x80 | 0x20_0000);
    let text = e.section("__text").unwrap();
    assert_eq!(text.addr, 0x1_0000_0000 + text.offset);
    let main = m.symbols[m.symbol_by_name("_main").unwrap()].definition.unwrap().1;
    assert_eq!(e.entry_in_text(), main);
    assert_eq!(e.contents(&exe, "__text").len(), m.sections[0].data.len());

    let dir = scratch("macho_hand_built");
    let p = dir.join("hello");
    write_file(&p, &exe);
    let Some(h) = objdump(&p, &["--all-headers", "-d"]) else { return };
    assert!(h.contains("cmd LC_MAIN"), "{h}");
    let lp = dir.join("hello.o");
    write_file(&lp, &obj);
    let Some(lld) = ld64(&[&lp], &dir.join("hello.lld"), &[]) else { return };
    same_linked_contents(&m, (&exe, &e), (&lld, &parse(&lld))).unwrap();
}

/// Calls to a global function defined in the same section become extern `BRANCH` relocations
/// (like llvm-mc), and the executable resolves them like ld64.lld.
#[test]
fn macho_calls_to_defined_globals() {
    let m = samples::hello_world_macos_global_calls();
    let obj = write(&m, Target::MacOS, OutputKind::Object).unwrap();
    let exe = write(&m, Target::MacOS, OutputKind::Executable).unwrap();
    let dir = scratch("macho_global_calls");
    let (po, pe) = (dir.join("hello.o"), dir.join("hello"));
    write_file(&po, &obj);
    write_file(&pe, &exe);
    let Some(r) = objdump(&po, &["-r"]) else { return };
    let branches = r.lines().filter(|l| l.split_whitespace().skip(1).eq(["True", "long", "True", "BRANCH", "False", "writechar"])).count();
    assert_eq!(branches, 3, "{r}");
    let Some(lld) = ld64(&[&po], &dir.join("hello.lld"), &[]) else { return };
    same_linked_contents(&m, (&exe, &parse(&exe)), (&lld, &parse(&lld))).unwrap();
}

/// Link errors: undefined symbols (no libraries are linked), a missing `_main`, and 32-bit
/// absolute references, which x86-64 Mach-O cannot express.
#[test]
fn macho_link_errors() {
    let mut m = samples::hello_world_macos();
    let id = m.symbol_by_name("_main").unwrap();
    m.symbols[id].name = "main".into();
    let err = write(&m, Target::MacOS, OutputKind::Executable).unwrap_err();
    assert!(err.contains("undefined reference to `_main'"), "{err}");

    let dir = scratch("macho_link_errors");
    let Some(m) = module_for(&fixture_dir().join("edge.macos.s"), &dir) else { return };
    let err = write(&m, Target::MacOS, OutputKind::Executable).unwrap_err();
    assert!(err.contains("(.text+0x1): undefined reference to `ext'"), "{err}");
    assert!(err.contains("(.data+0x20): undefined reference to `ext'"), "{err}");

    let Some(m) = module_for(&fixture_dir().join("edge.linux.s"), &dir) else { return };
    for kind in [OutputKind::Object, OutputKind::Executable] {
        let err = write(&m, Target::MacOS, kind).unwrap_err();
        assert!(err.contains("32-bit absolute addressing is not supported"), "{err}");
    }
}
