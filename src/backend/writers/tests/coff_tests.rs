//! COFF: objects are compared with what `x86_64-w64-mingw32-as` writes for the same fixture
//! (via `llvm-readobj` and the raw section contents) and linked with mingw-w64 gcc.
use std::process::Command;

use super::*;
use crate::backend::writers::write;
use crate::backend::OutputKind;
use crate::target::Target;

/// Assembles `src` with mingw-w64 GNU as.
fn mingw_as(src: &Path, out: &Path) -> Option<Vec<u8>> {
    let gas = need(tool(&["x86_64-w64-mingw32-as"]), "x86_64-w64-mingw32-as")?;
    run_ok(Command::new(gas).arg("-o").arg(out).arg(src));
    Some(std::fs::read(out).unwrap())
}

/// Windows fixtures plus the relocation edge cases (target independent assembly).
fn coff_fixtures() -> Vec<(String, PathBuf)> {
    let mut v = fixtures("windows");
    v.push(("edge".into(), fixture_dir().join("edge.linux.s")));
    v
}

/// What `llvm-readobj` shows of a COFF object, minus what only depends on gas' layout
/// choices: sections `(name, characteristics without the alignment)` (sizes are compared
/// separately: gas pads to the alignment, at least 16), symbols `(name, value, section,
/// class)` without `.file` and the empty `.bss` (a local code label is `Static` to us and
/// `Label` to gas), and relocations `(section, offset, type, symbol)`.
#[derive(Debug, PartialEq)]
struct CoffStructure {
    sections: Vec<String>,
    symbols: Vec<String>,
    relocs: Vec<String>,
}

/// Parses `llvm-readobj --sections --symbols --relocations` of `obj` into a [`CoffStructure`].
fn coff_structure(readobj: &Path, obj: &Path) -> CoffStructure {
    let out = run_ok(Command::new(readobj).args(["--sections", "--symbols", "--relocations"]).arg(obj));
    assert!(out.stderr.is_empty(), "llvm-readobj {}: {}", obj.display(), String::from_utf8_lossy(&out.stderr));
    let text = String::from_utf8(out.stdout).unwrap();
    let mut st = CoffStructure { sections: vec![], symbols: vec![], relocs: vec![] };
    let (mut name, mut value, mut section) = (String::new(), String::new(), String::new());
    let mut reloc_section = None;
    for l in text.lines().map(str::trim) {
        let field = |key: &str| l.strip_prefix(key).map(|v| v.split(" (").next().unwrap().to_string());
        if let Some(v) = field("Name: ") {
            name = v;
        } else if let Some(v) = field("Value: ") {
            value = v;
        } else if let Some(v) = field("Section: ") {
            section = v;
        } else if let Some(v) = l.strip_prefix("Characteristics [ (") {
            let c = u32::from_str_radix(v.trim_end_matches(')').trim_start_matches("0x"), 16).unwrap();
            if name != ".bss" {
                st.sections.push(format!("{name} {:#x}", c & !0x00f0_0000));
            }
        } else if let Some(v) = field("StorageClass: ") {
            let class = if v == "Label" { "Static".to_string() } else { v };
            if name != ".file" && name != ".bss" {
                st.symbols.push(format!("{name} {value} {section} {class}"));
            }
        } else if let Some(v) = l.strip_prefix("Section (") {
            reloc_section = Some(v.split(' ').nth(1).unwrap().to_string());
        } else if l == "}" {
            reloc_section = None;
        } else if let Some(sec) = &reloc_section {
            // offset type symbol (index)
            st.relocs.push(format!("{sec} {}", l.rsplit_once(" (").unwrap().0));
        }
    }
    st.symbols.sort();
    st
}

/// Raw data of the COFF section `name`.
fn coff_section(b: &[u8], name: &str) -> Vec<u8> {
    let n = u16::from_le_bytes([b[2], b[3]]) as usize;
    (0..n)
        .map(|i| 20 + 40 * i)
        .find(|&h| cstr(&b[h..h + 8]) == name)
        .map(|h| {
            let (size, ptr) = (le32(b, h + 16) as usize, le32(b, h + 20) as usize);
            b[ptr..ptr + size].to_vec()
        })
        .unwrap_or_default()
}

/// Our COFF objects have the sections, symbols, relocations and section contents
/// `x86_64-w64-mingw32-as` writes for the same source (the layout differs: no `.file` symbol,
/// no empty `.bss`, no padding to the section alignment).
#[test]
fn coff_objects_match_mingw_as() {
    let dir = scratch("coff_objects_match_mingw_as");
    let Some(readobj) = need(llvm("llvm-readobj"), "llvm-readobj") else { return };
    let mut checked = 0;
    for (name, src) in coff_fixtures() {
        let Some(m) = module_for(&src, &dir) else { return };
        let theirs_path = dir.join(format!("{name}.mingw.o"));
        let Some(theirs) = mingw_as(&src, &theirs_path) else { return };
        let ours = write(&m, Target::Windows, OutputKind::Object).unwrap();
        let ours_path = dir.join(format!("{name}.ours.o"));
        write_file(&ours_path, &ours);
        let (a, b) = (coff_structure(&readobj, &ours_path), coff_structure(&readobj, &theirs_path));
        assert!(!b.sections.is_empty() && !b.symbols.is_empty());
        assert_eq!(a, b, "{name}: compare `llvm-readobj -S -s -r` of {} and {}", ours_path.display(), theirs_path.display());
        for (i, (sec, pad)) in [(".text", 0x90), (".data", 0)].into_iter().enumerate() {
            let (a, b) = (coff_section(&ours, sec), coff_section(&theirs, sec));
            let padded = a.len().next_multiple_of(m.sections[i].align.max(16) as usize);
            assert!(
                b.starts_with(&a) && b[a.len()..].iter().all(|&c| c == pad) && b.len() == padded,
                "{name}: {sec} contents differ"
            );
        }
        checked += 1;
    }
    assert!(checked >= 10, "only {checked} fixtures checked");
}

/// Objects link with the mingw-w64 toolchain (against its crt), proving the relocations and
/// symbols are usable by a real PE linker.
#[test]
fn coff_objects_link_with_mingw() {
    let dir = scratch("coff_objects_link");
    let Some(gcc) = need(tool(&["x86_64-w64-mingw32-gcc"]), "x86_64-w64-mingw32-gcc") else { return };
    let mut linked = 0;
    for (name, src) in fixtures("windows") {
        if program_input(&name).is_none() {
            continue;
        }
        let Some(m) = module_for(&src, &dir) else { return };
        let obj = dir.join(format!("{name}.obj"));
        write_file(&obj, &write(&m, Target::Windows, OutputKind::Object).unwrap());
        run_ok(Command::new(&gcc).arg("-o").arg(dir.join(format!("{name}.exe"))).arg(&obj));
        linked += 1;
    }
    assert!(linked >= 5);
}

/// Without any tools: the hand-built module produces a well-formed header.
#[test]
fn coff_hand_built_hello_world() {
    let m = samples::hello_world_windows();
    let obj = write(&m, Target::Windows, OutputKind::Object).unwrap();
    assert_eq!(&obj[..2], &[0x64, 0x86]);
    assert_eq!(u16::from_le_bytes([obj[2], obj[3]]), 2);
}
