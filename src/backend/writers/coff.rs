//! COFF x86-64 relocatable objects (`pe-x86-64`), laid out exactly like
//! `x86_64-w64-mingw32-as` writes them: sections `.text`/`.data`/`.bss`, then raw data,
//! relocations, symbol table and string table.
use super::super::module::{Binding, ObjModule, RelocKind};
use super::{align_up, is_global, Le};

pub const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const IMAGE_FILE_LINE_NUMS_STRIPPED: u16 = 0x0004;

pub const IMAGE_SCN_CNT_CODE: u32 = 0x20;
pub const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x40;
pub const IMAGE_SCN_CNT_UNINITIALIZED_DATA: u32 = 0x80;
pub const IMAGE_SCN_MEM_EXECUTE: u32 = 0x2000_0000;
pub const IMAGE_SCN_MEM_READ: u32 = 0x4000_0000;
pub const IMAGE_SCN_MEM_WRITE: u32 = 0x8000_0000;

pub const IMAGE_REL_AMD64_ADDR64: u16 = 1;
pub const IMAGE_REL_AMD64_ADDR32: u16 = 2;
pub const IMAGE_REL_AMD64_REL32: u16 = 4;

const IMAGE_SYM_CLASS_EXTERNAL: u8 = 2;
const IMAGE_SYM_CLASS_STATIC: u8 = 3;
const IMAGE_SYM_CLASS_LABEL: u8 = 6;
const IMAGE_SYM_CLASS_FILE: u8 = 103;
const IMAGE_SYM_DEBUG: i16 = -2;

/// Largest section alignment COFF can express (`IMAGE_SCN_ALIGN_8192BYTES`).
const MAX_ALIGN: u64 = 8192;
/// gas' default section alignment for PE/COFF.
const DEFAULT_ALIGN: u64 = 16;

const SECTION_NAMES: [&str; 3] = [".text", ".data", ".bss"];

fn align_flag(align: u64) -> u32 {
    (align.trailing_zeros() + 1) << 20
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
fn name_field(name: &str, strtab: &mut Vec<u8>) -> [u8; 8] {
    let mut f = [0u8; 8];
    if name.len() <= 8 {
        f[..name.len()].copy_from_slice(name.as_bytes());
    } else {
        let off = strtab.len() as u32 + 4;
        strtab.extend_from_slice(name.as_bytes());
        strtab.push(0);
        f[4..].copy_from_slice(&off.to_le_bytes());
    }
    f
}

pub fn write_object(m: &ObjModule) -> Result<Vec<u8>, String> {
    let mut aligns = [DEFAULT_ALIGN; 3];
    for (i, s) in m.sections.iter().enumerate() {
        if s.align > MAX_ALIGN {
            return Err(format!("section {}: alignment 2**{} not representable", SECTION_NAMES[i], s.align.trailing_zeros()));
        }
        aligns[i] = s.align.max(DEFAULT_ALIGN);
    }

    // Raw data, padded to the section alignment (code with NOPs).
    let raw: Vec<Vec<u8>> = m
        .sections
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let mut d = s.data.clone();
            if !d.is_empty() {
                d.pad_to(align_up(d.len() as u64, aligns[i]), if i == 0 { 0x90 } else { 0 });
            }
            d
        })
        .collect();

    // Like mingw-as, PC-relative references to a symbol in the same section (global or not)
    // are resolved here; everything else becomes a relocation.
    let local_pcrel = |si: usize, r: &super::super::module::Reloc| {
        matches!(r.kind, RelocKind::Pc32 | RelocKind::Branch32) && m.symbols[r.symbol].definition.is_some_and(|(t, _)| t == si)
    };
    let nreloc: Vec<u16> = m.sections.iter().enumerate().map(|(si, s)| s.relocs.iter().filter(|r| !local_pcrel(si, r)).count() as u16).collect();

    // Symbol table: .file, local labels, section symbols, globals, undefined symbols.
    let mut syms: Vec<CoffSym> = Vec::new();
    let mut file_aux = [0u8; 18];
    file_aux[..4].copy_from_slice(b"fake");
    syms.push(CoffSym { name: ".file".into(), value: 0, section: IMAGE_SYM_DEBUG, class: IMAGE_SYM_CLASS_FILE, aux: vec![file_aux] });
    for s in &m.symbols {
        if let (Some((sec, off)), false, Binding::Local) = (s.definition, s.temporary, s.binding) {
            let class = if sec == 0 { IMAGE_SYM_CLASS_LABEL } else { IMAGE_SYM_CLASS_STATIC };
            syms.push(CoffSym { name: s.name.clone(), value: off as u32, section: sec as i16 + 1, class, aux: Vec::new() });
        }
    }
    let mut section_sym = [0u32; 3];
    let index = |syms: &Vec<CoffSym>| syms.iter().map(|s| 1 + s.aux.len() as u32).sum::<u32>();
    for i in 0..3 {
        section_sym[i] = index(&syms);
        let (len, nrel) = match m.sections.get(i) {
            Some(s) => (s.data.len() as u32, nreloc[i]),
            None => (0, 0),
        };
        let mut aux = [0u8; 18];
        aux[..4].copy_from_slice(&len.to_le_bytes());
        aux[4..6].copy_from_slice(&nrel.to_le_bytes());
        syms.push(CoffSym { name: SECTION_NAMES[i].into(), value: 0, section: i as i16 + 1, class: IMAGE_SYM_CLASS_STATIC, aux: vec![aux] });
    }
    let mut sym_index = vec![u32::MAX; m.symbols.len()];
    for (id, s) in m.symbols.iter().enumerate() {
        if let (Some((sec, off)), Binding::Global) = (s.definition, s.binding) {
            sym_index[id] = index(&syms);
            syms.push(CoffSym { name: s.name.clone(), value: off as u32, section: sec as i16 + 1, class: IMAGE_SYM_CLASS_EXTERNAL, aux: Vec::new() });
        }
    }
    // Undefined symbols that are actually referenced (gas drops unused `.extern`s).
    let mut referenced = vec![false; m.symbols.len()];
    for (si, sec) in m.sections.iter().enumerate() {
        for r in sec.relocs.iter().filter(|r| !local_pcrel(si, r)) {
            referenced[r.symbol] = true;
        }
    }
    for (id, s) in m.symbols.iter().enumerate() {
        if s.definition.is_none() && (referenced[id] || s.binding == Binding::Global) {
            sym_index[id] = index(&syms);
            syms.push(CoffSym { name: s.name.clone(), value: 0, section: 0, class: IMAGE_SYM_CLASS_EXTERNAL, aux: Vec::new() });
        }
    }

    // Relocations: defined symbols (local or global) become section-relative, the symbol
    // offset going into the field; undefined symbols are referenced by name.
    let mut fields = raw;
    let mut relocs: [Vec<u8>; 2] = [Vec::new(), Vec::new()];
    for (si, sec) in m.sections.iter().enumerate() {
        for r in &sec.relocs {
            let s = &m.symbols[r.symbol];
            if local_pcrel(si, r) {
                let (_, off) = s.definition.expect("checked");
                let v = off as i64 + r.addend - r.offset as i64;
                let v = i32::try_from(v).map_err(|_| format!("relocation out of range for `{}'", s.name))?;
                fields[si][r.offset as usize..r.offset as usize + 4].copy_from_slice(&v.to_le_bytes());
                continue;
            }
            let (symi, base) = match s.definition {
                Some((tsec, off)) => (section_sym[tsec], off as i64),
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
                    f[o..o + 4].copy_from_slice(&v.to_le_bytes());
                    IMAGE_REL_AMD64_REL32
                }
                RelocKind::Abs32 => {
                    let v = base + r.addend;
                    let v = i32::try_from(v).map_err(|_| format!("relocation addend out of range for `{}'", s.name))?;
                    f[o..o + 4].copy_from_slice(&v.to_le_bytes());
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
    let mut pos = 20 + 40 * 3u32;
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
    let symtab_ptr = pos;
    let nsyms = index(&syms);

    let mut out = Vec::new();
    out.u16(IMAGE_FILE_MACHINE_AMD64);
    out.u16(3);
    out.u32(0); // TimeDateStamp
    out.u32(symtab_ptr);
    out.u32(nsyms);
    out.u16(0); // SizeOfOptionalHeader
    out.u16(IMAGE_FILE_LINE_NUMS_STRIPPED);
    let chars = [
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE,
        IMAGE_SCN_CNT_UNINITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE,
    ];
    for i in 0..3 {
        out.name::<8>(SECTION_NAMES[i]);
        out.u32(0); // VirtualSize
        out.u32(0); // VirtualAddress
        out.u32(fields.get(i).map_or(0, |f| f.len() as u32));
        out.u32(raw_ptr.get(i).copied().unwrap_or(0));
        out.u32(reloc_ptr.get(i).copied().unwrap_or(0));
        out.u32(0); // PointerToLinenumbers
        out.u16(nreloc.get(i).copied().unwrap_or(0));
        out.u16(0);
        out.u32(chars[i] | align_flag(aligns[i]));
    }
    for f in &fields {
        out.extend_from_slice(f);
    }
    for r in &relocs {
        out.extend_from_slice(r);
    }
    let mut strtab = Vec::new();
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
    out.u32(strtab.len() as u32 + 4);
    out.extend_from_slice(&strtab);
    Ok(out)
}
