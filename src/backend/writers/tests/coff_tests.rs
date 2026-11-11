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

/// Our COFF objects are byte-for-byte what `x86_64-w64-mingw32-as` writes for the same source.
#[test]
fn coff_objects_match_mingw_as() {
    let dir = scratch("coff_objects_match_mingw_as");
    let mut checked = 0;
    for (name, src) in coff_fixtures() {
        let Some(m) = module_for(&src, &dir) else { return };
        let theirs_path = dir.join(format!("{name}.mingw.o"));
        let Some(theirs) = mingw_as(&src, &theirs_path) else { return };
        let ours = write(&m, Target::Windows, OutputKind::Object).unwrap();
        if ours != theirs {
            let p = dir.join(format!("{name}.ours.o"));
            std::fs::write(&p, &ours).unwrap();
            panic!(
                "{name}: object differs from mingw-as; compare `x86_64-w64-mingw32-objdump -x -s` of {} and {}",
                p.display(),
                theirs_path.display()
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
    assert_eq!(u16::from_le_bytes([obj[2], obj[3]]), 3);
}
