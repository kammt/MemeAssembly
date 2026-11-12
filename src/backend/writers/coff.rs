//! COFF x86-64 relocatable objects (`pe-x86-64`), with the sections, symbols and relocations
//! `x86_64-w64-mingw32-as` writes: section headers for `.text`/`.data`, then raw data,
//! relocations, symbol table and string table.
use super::super::module::{ObjModule, Reloc, RelocKind};
use super::{is_global, put_u32, symbol_groups, Le, StrTab};

pub const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const IMAGE_FILE_LINE_NUMS_STRIPPED: u16 = 0x0004;

pub const IMAGE_SCN_CNT_CODE: u32 = 0x20;
pub const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x40;
pub const IMAGE_SCN_MEM_EXECUTE: u32 = 0x2000_0000;
pub const IMAGE_SCN_MEM_READ: u32 = 0x4000_0000;
pub const IMAGE_SCN_MEM_WRITE: u32 = 0x8000_0000;

pub const IMAGE_REL_AMD64_ADDR64: u16 = 1;
pub const IMAGE_REL_AMD64_ADDR32: u16 = 2;
pub const IMAGE_REL_AMD64_REL32: u16 = 4;

const IMAGE_SYM_CLASS_EXTERNAL: u8 = 2;
const IMAGE_SYM_CLASS_STATIC: u8 = 3;

/// Largest section alignment COFF can express (`IMAGE_SCN_ALIGN_8192BYTES`).
const MAX_ALIGN: u64 = 8192;

const SECTION_NAMES: [&str; 2] = [".text", ".data"];

fn align_flag(align: u64) -> u32 {
    (align.max(1).trailing_zeros() + 1) << 20
}

/// One symbol table record (18 bytes) plus its auxiliary records.
struct CoffSym {
    name: String,
    value: u32,
    section: i16,
    class: u8,
    aux: Vec<[u8; 18]>,
}

/// Symbol names: up to 8 bytes inline (NUL padded, no terminator needed), longer ones in the
/// string table.
fn name_field(name: &str, strtab: &mut StrTab) -> [u8; 8] {
    let mut f = [0u8; 8];
    if name.len() <= 8 {
        f[..name.len()].copy_from_slice(name.as_bytes());
    } else {
        f[4..].copy_from_slice(&strtab.add(name).to_le_bytes());
    }
    f
}

pub fn write_object(m: &ObjModule) -> Result<Vec<u8>, String> {
    for (i, s) in m.sections.iter().enumerate() {
        if s.align > MAX_ALIGN {
            return Err(format!("section {}: alignment 2**{} not representable", SECTION_NAMES[i], s.align.trailing_zeros()));
        }
    }

    // Like mingw-as, PC-relative references to a symbol in the same section (global or not)
    // are resolved here; everything else becomes a relocation.
    let local_pcrel = |si: usize, r: &Reloc| {
        matches!(r.kind, RelocKind::Pc32 | RelocKind::Branch32) && m.symbols[r.symbol].definition.is_some_and(|(t, _)| t == si)
    };
    let nreloc: Vec<u16> = m.sections.iter().enumerate().map(|(si, s)| s.relocs.iter().filter(|r| !local_pcrel(si, r)).count() as u16).collect();

    // Symbol table: section symbols (section i at index 2i, one aux record each), local
    // labels, globals, undefined symbols.
    let mut syms: Vec<CoffSym> = Vec::new();
    for (i, s) in m.sections.iter().enumerate() {
        let mut aux = [0u8; 18];
        aux[..4].copy_from_slice(&(s.data.len() as u32).to_le_bytes());
        aux[4..6].copy_from_slice(&nreloc[i].to_le_bytes());
        syms.push(CoffSym { name: SECTION_NAMES[i].into(), value: 0, section: i as i16 + 1, class: IMAGE_SYM_CLASS_STATIC, aux: vec![aux] });
    }
    let groups = symbol_groups(m);
    let mut sym_index = vec![u32::MAX; m.symbols.len()];
    let mut next = 2 * syms.len() as u32;
    for (&id, class) in groups
        .locals
        .iter()
        .map(|id| (id, IMAGE_SYM_CLASS_STATIC))
        .chain(groups.globals.iter().chain(&groups.undefined).map(|id| (id, IMAGE_SYM_CLASS_EXTERNAL)))
    {
        let s = &m.symbols[id];
        let (section, value) = s.definition.map_or((0, 0), |(sec, off)| (sec as i16 + 1, off as u32));
        sym_index[id] = next;
        next += 1;
        syms.push(CoffSym { name: s.name.clone(), value, section, class, aux: Vec::new() });
    }

    // Relocations: defined symbols (local or global) become section-relative, the symbol
    // offset going into the field; undefined symbols are referenced by name.
    let mut fields: Vec<Vec<u8>> = m.sections.iter().map(|s| s.data.clone()).collect();
    let mut relocs: [Vec<u8>; 2] = [Vec::new(), Vec::new()];
    for (si, sec) in m.sections.iter().enumerate() {
        for r in &sec.relocs {
            let s = &m.symbols[r.symbol];
            if local_pcrel(si, r) {
                let (_, off) = s.definition.expect("checked");
                let v = off as i64 + r.addend - r.offset as i64;
                let v = i32::try_from(v).map_err(|_| format!("relocation out of range for `{}'", s.name))?;
                put_u32(&mut fields[si], r.offset as usize, v as u32);
                continue;
            }
            let (symi, base) = match s.definition {
                Some((tsec, off)) => (2 * tsec as u32, off as i64),
                None => {
                    debug_assert!(is_global(s));
                    (sym_index[r.symbol], 0)
                }
            };
            let o = r.offset as usize;
            let f = &mut fields[si];
            let ty = match r.kind {
                // The CPU adds the end of the field (P + 4); the ELF addend already accounts for
                // it relative to the field start.
                RelocKind::Pc32 | RelocKind::Branch32 => {
                    let v = base + r.addend + 4;
                    let v = i32::try_from(v).map_err(|_| format!("relocation addend out of range for `{}'", s.name))?;
                    put_u32(f, o, v as u32);
                    IMAGE_REL_AMD64_REL32
                }
                RelocKind::Abs32 => {
                    let v = base + r.addend;
                    let v = i32::try_from(v).map_err(|_| format!("relocation addend out of range for `{}'", s.name))?;
                    put_u32(f, o, v as u32);
                    IMAGE_REL_AMD64_ADDR32
                }
                RelocKind::Abs64 => {
                    f[o..o + 8].copy_from_slice(&(base + r.addend).to_le_bytes());
                    IMAGE_REL_AMD64_ADDR64
                }
            };
            relocs[si].u32(r.offset as u32);
            relocs[si].u32(symi);
            relocs[si].u16(ty);
        }
    }

    // Layout: header, section headers, raw data, relocations, symbols, strings.
    let mut pos = 20 + 40 * 2u32;
    let mut raw_ptr = [0u32; 2];
    for i in 0..2 {
        if !fields[i].is_empty() {
            raw_ptr[i] = pos;
            pos += fields[i].len() as u32;
        }
    }
    let mut reloc_ptr = [0u32; 2];
    for i in 0..2 {
        if !relocs[i].is_empty() {
            reloc_ptr[i] = pos;
            pos += relocs[i].len() as u32;
        }
    }

    let mut out = Vec::new();
    out.u16(IMAGE_FILE_MACHINE_AMD64);
    out.u16(2);
    out.u32(0); // TimeDateStamp
    out.u32(pos); // PointerToSymbolTable
    out.u32(next); // NumberOfSymbols
    out.u16(0); // SizeOfOptionalHeader
    out.u16(IMAGE_FILE_LINE_NUMS_STRIPPED);
    let chars = [
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE,
    ];
    for i in 0..2 {
        out.name::<8>(SECTION_NAMES[i]);
        out.u32(0); // VirtualSize
        out.u32(0); // VirtualAddress
        out.u32(fields[i].len() as u32);
        out.u32(raw_ptr[i]);
        out.u32(reloc_ptr[i]);
        out.u32(0); // PointerToLinenumbers
        out.u16(nreloc[i]);
        out.u16(0);
        out.u32(chars[i] | align_flag(m.sections[i].align));
    }
    for f in &fields {
        out.extend_from_slice(f);
    }
    for r in &relocs {
        out.extend_from_slice(r);
    }
    // The string table starts with its own size, so offsets count from the 4-byte prefix.
    let mut strtab = StrTab::new(&[0; 4]);
    for s in &syms {
        out.extend_from_slice(&name_field(&s.name, &mut strtab));
        out.u32(s.value);
        out.u16(s.section as u16);
        out.u16(0); // Type
        out.u8(s.class);
        out.u8(s.aux.len() as u8);
        for a in &s.aux {
            out.extend_from_slice(a);
        }
    }
    let len = strtab.data.len() as u32;
    put_u32(&mut strtab.data, 0, len);
    out.extend_from_slice(&strtab.data);
    Ok(out)
}
