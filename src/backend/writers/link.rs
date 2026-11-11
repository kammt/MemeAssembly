//! The part of a static linker shared by the executable writers: given final addresses for
//! the module's sections, resolve symbols and apply every relocation.
use super::super::module::{Binding, ObjModule, RelocKind, Symbol, SymbolId};
use super::{field_size, is_pcrel, put_u32, put_u64, section_name};

/// Final address of a symbol, or `None` if it is undefined and `resolve_undefined` doesn't
/// know it either.
pub fn symbol_address(
    m: &ObjModule,
    id: SymbolId,
    section_addrs: &[u64; 2],
    resolve_undefined: &dyn Fn(&str) -> Option<u64>,
) -> Option<u64> {
    let s = &m.symbols[id];
    match s.definition {
        Some((sec, off)) => Some(section_addrs[sec] + off),
        None => resolve_undefined(&s.name),
    }
}

/// Result of [`relocate`].
pub struct Linked {
    /// Section contents (`.text`, `.data`) with all relocations applied.
    pub sections: [Vec<u8>; 2],
    /// `(section, offset)` of every 64-bit absolute address that was written (needed for
    /// rebasing position independent images).
    pub abs64_sites: Vec<(usize, u64)>,
}

/// Applies all relocations of `m` assuming `.text`/`.data` are loaded at `section_addrs`.
/// Undefined symbols are looked up with `resolve_undefined`; failures are reported like ld
/// does (`(.text+0x1b): undefined reference to `foo'`), all at once.
pub fn relocate(
    m: &ObjModule,
    section_addrs: &[u64; 2],
    resolve_undefined: &dyn Fn(&str) -> Option<u64>,
) -> Result<Linked, String> {
    let mut sections = [m.sections[0].data.clone(), m.sections[1].data.clone()];
    let mut abs64_sites = Vec::new();
    let mut errors = Vec::new();
    for (si, sec) in m.sections.iter().enumerate() {
        for r in &sec.relocs {
            let sym = &m.symbols[r.symbol];
            let Some(s) = symbol_address(m, r.symbol, section_addrs, resolve_undefined) else {
                errors.push(format!(
                    "({}+{:#x}): undefined reference to `{}'",
                    section_name(si),
                    r.offset,
                    sym.name
                ));
                continue;
            };
            let p = section_addrs[si] + r.offset;
            let value = compute(r.kind, s, r.addend, p);
            let off = r.offset as usize;
            let buf = &mut sections[si];
            match write_field(buf, off, r.kind, value) {
                Ok(()) => {}
                Err(()) => errors.push(format!(
                    "({}+{:#x}): relocation truncated to fit: {} against `{}'",
                    section_name(si),
                    r.offset,
                    kind_name(r.kind),
                    sym.name
                )),
            }
            if r.kind == RelocKind::Abs64 {
                abs64_sites.push((si, r.offset));
            }
            debug_assert!(off as u64 + field_size(r.kind) <= buf.len() as u64);
        }
    }
    if errors.is_empty() {
        Ok(Linked { sections, abs64_sites })
    } else {
        Err(errors.join("\n"))
    }
}

/// ELF semantics: `S + A - P` for PC-relative kinds, `S + A` otherwise.
pub fn compute(kind: RelocKind, s: u64, addend: i64, p: u64) -> i128 {
    let v = s as i128 + addend as i128;
    if is_pcrel(kind) {
        v - p as i128
    } else {
        v
    }
}

/// Stores a resolved value, checking that it fits the field like ld does.
pub fn write_field(buf: &mut [u8], off: usize, kind: RelocKind, value: i128) -> Result<(), ()> {
    match kind {
        RelocKind::Pc32 | RelocKind::Branch32 => {
            let v = i32::try_from(value).map_err(|_| ())?;
            put_u32(buf, off, v as u32);
        }
        RelocKind::Abs32 => {
            // R_X86_64_32 is zero extended.
            let v = u32::try_from(value).map_err(|_| ())?;
            put_u32(buf, off, v);
        }
        RelocKind::Abs64 => {
            let v = if value < 0 { i64::try_from(value).map_err(|_| ())? as u64 } else { u64::try_from(value).map_err(|_| ())? };
            put_u64(buf, off, v);
        }
    }
    Ok(())
}

fn kind_name(kind: RelocKind) -> &'static str {
    match kind {
        RelocKind::Pc32 => "R_X86_64_PC32",
        RelocKind::Branch32 => "R_X86_64_PLT32",
        RelocKind::Abs32 => "R_X86_64_32",
        RelocKind::Abs64 => "R_X86_64_64",
    }
}

/// An undefined `main`, reported the way ld does for crt's reference to it.
pub const NO_MAIN: &str = "in function `_start': undefined reference to `main'";

/// Links a program: [`relocate`]s `m` and finds its entry point, the global symbol `entry`
/// defined in `m`. Returns the relocated sections and the entry's address. All errors are
/// reported at once, a missing entry point (as `missing`) after the relocation errors.
pub fn link_program(
    m: &ObjModule,
    entry: &str,
    missing: &str,
    section_addrs: &[u64; 2],
    resolve_undefined: &dyn Fn(&str) -> Option<u64>,
) -> Result<(Linked, u64), String> {
    let entry = m.symbol_by_name(entry).and_then(|id| match m.symbols[id] {
        Symbol { definition: Some((sec, off)), binding: Binding::Global, .. } => Some(section_addrs[sec] + off),
        _ => None,
    });
    match (relocate(m, section_addrs, resolve_undefined), entry) {
        (Ok(linked), Some(entry)) => Ok((linked, entry)),
        (Ok(_), None) => Err(missing.to_string()),
        (Err(e), Some(_)) => Err(e),
        (Err(e), None) => Err(format!("{e}\n{missing}")),
    }
}
