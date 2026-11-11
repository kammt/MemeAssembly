//! ELF64 x86-64: relocatable objects laid out exactly like GNU as (BFD) does, and static,
//! non-PIE, libc-free executables.
use std::collections::HashMap;

use super::super::fadec::*;
use super::super::module::{Binding, ObjModule, RelocKind, SymbolId};
use super::dwarf::{self, DebugSection, RelTarget};
use super::link;
use super::{align_up, is_global, Asm, Le, SuffixStrTab};

const SHT_PROGBITS: u32 = 1;
const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_RELA: u32 = 4;
const SHT_NOBITS: u32 = 8;
const SHF_WRITE: u64 = 1;
const SHF_ALLOC: u64 = 2;
const SHF_EXECINSTR: u64 = 4;
const SHF_MERGE: u64 = 0x10;
const SHF_STRINGS: u64 = 0x20;
const SHF_INFO_LINK: u64 = 0x40;

const R_X86_64_64: u32 = 1;
const R_X86_64_PC32: u32 = 2;
const R_X86_64_32: u32 = 10;
const R_X86_64_PLT32: u32 = 4;

const STB_LOCAL: u8 = 0;
const STB_GLOBAL: u8 = 1;
const STT_NOTYPE: u8 = 0;
const STT_FUNC: u8 = 2;
const STT_SECTION: u8 = 3;

/// Base address of non-PIE x86-64 executables (ld's default `-Ttext-segment`).
const EXEC_BASE: u64 = 0x400000;
const PAGE: u64 = 0x1000;

struct Shdr {
    name: String,
    sh_type: u32,
    flags: u64,
    addr: u64,
    data: Vec<u8>,
    /// For SHT_NOBITS.
    nobits_size: u64,
    link: u32,
    info: u32,
    align: u64,
    entsize: u64,
    offset: u64,
}

impl Shdr {
    fn new(name: &str, sh_type: u32, flags: u64, data: Vec<u8>, align: u64) -> Shdr {
        Shdr {
            name: name.into(),
            sh_type,
            flags,
            addr: 0,
            data,
            nobits_size: 0,
            link: 0,
            info: 0,
            align,
            entsize: 0,
            offset: 0,
        }
    }

    fn size(&self) -> u64 {
        if self.sh_type == SHT_NOBITS {
            self.nobits_size
        } else {
            self.data.len() as u64
        }
    }
}

struct ElfSym {
    name: String,
    info: u8,
    shndx: u16,
    value: u64,
}

fn sym_bytes(syms: &[ElfSym], names: &HashMap<String, u32>) -> Vec<u8> {
    let mut b = Vec::with_capacity(24 * syms.len());
    for s in syms {
        b.u32(names[&s.name]);
        b.u8(s.info);
        b.u8(0);
        b.u16(s.shndx);
        b.u64(s.value);
        b.u64(0);
    }
    b
}

/// Lays out and serializes an ELF file. `sections[0]` must be the NULL section placeholder;
/// `.shstrtab` is appended here. Allocated sections must already have their offsets set
/// (`offset != 0`) when `phdrs` is non-empty; all other sections are placed the way BFD does.
fn serialize(
    e_type: u16,
    entry: u64,
    phdrs: &[u8],
    preplaced_end: u64,
    mut sections: Vec<Shdr>,
    file_prefix: Vec<u8>,
) -> Vec<u8> {
    // .shstrtab: BFD adds .symtab/.strtab/.shstrtab first, then the section names in order.
    let mut shstr = SuffixStrTab::new();
    for n in [".symtab", ".strtab", ".shstrtab"] {
        if n != ".shstrtab" && !sections.iter().any(|s| s.name == n) {
            continue;
        }
        shstr.add(n);
    }
    for s in &sections[1..] {
        shstr.add(&s.name);
    }
    let (shstr_bytes, shstr_off) = shstr.finish();
    sections.push(Shdr::new(".shstrtab", SHT_STRTAB, 0, shstr_bytes, 1));
    let shstrndx = sections.len() - 1;

    // File placement (BFD order): ordinary sections in section order, then .symtab and
    // .strtab, then relocation sections, then .shstrtab.
    let mut order: Vec<usize> = Vec::new();
    let is_rel = |s: &Shdr| s.sh_type == SHT_RELA;
    let is_tab = |s: &Shdr| matches!(s.name.as_str(), ".symtab" | ".strtab" | ".shstrtab");
    order.extend((1..sections.len()).filter(|&i| !is_rel(&sections[i]) && !is_tab(&sections[i])));
    order.extend((1..sections.len()).filter(|&i| matches!(sections[i].name.as_str(), ".symtab" | ".strtab")));
    order.extend((1..sections.len()).filter(|&i| is_rel(&sections[i])));
    order.push(shstrndx);

    let mut out = file_prefix;
    let mut pos = if phdrs.is_empty() { 64 } else { preplaced_end };
    for &i in &order {
        let s = &mut sections[i];
        if s.offset != 0 {
            continue; // placed by the executable layout
        }
        // Like BFD, file offsets are aligned to at most 8 bytes (the ELF64 file alignment):
        // `sh_addralign` alone carries larger alignments, which matter only once loaded.
        pos = align_up(pos, s.align.clamp(1, 8));
        s.offset = pos;
        if s.sh_type != SHT_NOBITS {
            pos += s.data.len() as u64;
        }
    }
    let shoff = align_up(pos, 8);

    // Header.
    let mut h = Vec::with_capacity(64);
    h.extend_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    h.u16(e_type);
    h.u16(62); // EM_X86_64
    h.u32(1);
    h.u64(entry);
    h.u64(if phdrs.is_empty() { 0 } else { 64 });
    h.u64(shoff);
    h.u32(0);
    h.u16(64);
    h.u16(if phdrs.is_empty() { 0 } else { 56 });
    h.u16((phdrs.len() / 56) as u16);
    h.u16(64);
    h.u16(sections.len() as u16);
    h.u16(shstrndx as u16);
    if out.len() < 64 + phdrs.len() {
        out.resize(64 + phdrs.len(), 0);
    }
    out[..64].copy_from_slice(&h);
    out[64..64 + phdrs.len()].copy_from_slice(phdrs);

    for s in &sections[1..] {
        if s.sh_type == SHT_NOBITS || s.data.is_empty() {
            continue;
        }
        let end = s.offset as usize + s.data.len();
        if out.len() < end {
            out.resize(end, 0);
        }
        out[s.offset as usize..end].copy_from_slice(&s.data);
    }
    out.pad_to(shoff, 0);
    for s in &sections {
        if s.sh_type == 0 {
            out.extend_from_slice(&[0; 64]);
            continue;
        }
        out.u32(shstr_off[&s.name]);
        out.u32(s.sh_type);
        out.u64(s.flags);
        out.u64(s.addr);
        out.u64(s.offset);
        out.u64(s.size());
        out.u32(s.link);
        out.u32(s.info);
        out.u64(s.align);
        out.u64(s.entsize);
    }
    out
}

fn null_section() -> Shdr {
    Shdr::new("", 0, 0, Vec::new(), 0)
}

/// Target of an ELF relocation after gas' symbol adjustment.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Target {
    /// Section symbol of `.text`/`.data`.
    Section(usize),
    /// Section symbol of a debug section (index into the DWARF section list).
    Debug(usize),
    Symbol(SymbolId),
}

/// gas keeps relocations against global/undefined symbols and rewrites everything else to
/// "section symbol + offset". Branches to local symbols become plain PC32.
fn adjust(m: &ObjModule, sym: SymbolId, kind: RelocKind, addend: i64) -> (Target, u32, i64) {
    let s = &m.symbols[sym];
    let global = is_global(s);
    let ty = match kind {
        RelocKind::Pc32 => R_X86_64_PC32,
        RelocKind::Branch32 if global => R_X86_64_PLT32,
        RelocKind::Branch32 => R_X86_64_PC32,
        RelocKind::Abs32 => R_X86_64_32,
        RelocKind::Abs64 => R_X86_64_64,
    };
    match s.definition {
        Some((sec, off)) if !global => (Target::Section(sec), ty, addend.wrapping_add(off as i64)),
        _ => (Target::Symbol(sym), ty, addend),
    }
}

/// Section header for a DWARF section (`data` already resolved or with zero fields).
fn debug_shdr(s: &DebugSection, data: Vec<u8>) -> Shdr {
    if s.is_strings() {
        let mut h = Shdr::new(s.name, SHT_PROGBITS, SHF_MERGE | SHF_STRINGS, data, 1);
        h.entsize = 1;
        h
    } else {
        Shdr::new(s.name, SHT_PROGBITS, 0, data, 1)
    }
}

pub fn write_object(m: &ObjModule) -> Vec<u8> {
    // Relocations of .text/.data after gas' symbol adjustment.
    let mut text_data_relocs: [Vec<(u64, Target, u32, i64)>; 2] = [Vec::new(), Vec::new()];
    for (si, sec) in m.sections.iter().enumerate() {
        for r in &sec.relocs {
            let (t, ty, a) = adjust(m, r.symbol, r.kind, r.addend);
            text_data_relocs[si].push((r.offset, t, ty, a));
        }
    }
    // DWARF (`-g`): addresses are `.text` + offset, section offsets are against the debug
    // sections' own section symbols, as gas emits them.
    let debug = m.debug.as_ref().map(|d| dwarf::build(d, m.sections[0].data.len() as u64)).unwrap_or_default();
    let debug_index = |name: &str| debug.iter().position(|s| s.name == name).expect("known debug section");
    let debug_relocs: Vec<Vec<(u64, Target, u32, i64)>> = debug
        .iter()
        .map(|s| {
            s.relocs
                .iter()
                .map(|r| match r.target {
                    RelTarget::Text => (r.offset, Target::Section(0), R_X86_64_64, r.addend),
                    RelTarget::Section(n) => (r.offset, Target::Debug(debug_index(n)), R_X86_64_32, r.addend),
                })
                .collect()
        })
        .collect();

    // ELF section indices: .text=1, [.rela.text], .data, [.rela.data], .bss, then each debug
    // section followed by its relocations.
    let text_idx = 1u16;
    let data_idx = text_idx + 1 + !text_data_relocs[0].is_empty() as u16;
    let sec_idx = [text_idx, data_idx];
    let mut next = data_idx + 1 + !text_data_relocs[1].is_empty() as u16 + 1;
    let mut debug_idx = Vec::new();
    for r in &debug_relocs {
        debug_idx.push(next);
        next += 1 + !r.is_empty() as u16;
    }

    // Symbol table: NULL, section symbols of .text/.data in use, other locals, debug section
    // symbols in use, globals.
    let mut used_sections = [false; 2];
    let mut used_debug = vec![false; debug.len()];
    let mut referenced = vec![false; m.symbols.len()];
    for (_, t, _, _) in text_data_relocs.iter().chain(debug_relocs.iter()).flatten() {
        match *t {
            Target::Section(s) => used_sections[s] = true,
            Target::Debug(s) => used_debug[s] = true,
            Target::Symbol(id) => referenced[id] = true,
        }
    }
    let section_symbol = |shndx: u16| ElfSym { name: String::new(), info: (STB_LOCAL << 4) | STT_SECTION, shndx, value: 0 };
    let mut syms = vec![ElfSym { name: String::new(), info: 0, shndx: 0, value: 0 }];
    let mut section_sym = [0u32; 2];
    for s in 0..2 {
        if used_sections[s] {
            section_sym[s] = syms.len() as u32;
            syms.push(section_symbol(sec_idx[s]));
        }
    }
    let mut sym_index: HashMap<SymbolId, u32> = HashMap::new();
    for (id, s) in m.symbols.iter().enumerate() {
        if let (Some((sec, off)), false, Binding::Local) = (s.definition, s.temporary, s.binding) {
            sym_index.insert(id, syms.len() as u32);
            syms.push(ElfSym { name: s.name.clone(), info: (STB_LOCAL << 4) | STT_NOTYPE, shndx: sec_idx[sec], value: off });
        }
    }
    let mut debug_sym = vec![0u32; debug.len()];
    for (i, used) in used_debug.iter().enumerate() {
        if *used {
            debug_sym[i] = syms.len() as u32;
            syms.push(section_symbol(debug_idx[i]));
        }
    }
    let first_global = syms.len() as u32;
    for (id, s) in m.symbols.iter().enumerate() {
        if !is_global(s) || (s.binding != Binding::Global && !referenced[id]) {
            continue;
        }
        let (shndx, value) = match s.definition {
            Some((sec, off)) => (sec_idx[sec], off),
            None => (0, 0),
        };
        sym_index.insert(id, syms.len() as u32);
        syms.push(ElfSym { name: s.name.clone(), info: (STB_GLOBAL << 4) | STT_NOTYPE, shndx, value });
    }
    let mut strtab = SuffixStrTab::new();
    for s in &syms[1..] {
        strtab.add(&s.name);
    }
    let (strtab_bytes, str_off) = strtab.finish();

    let rela_bytes = |relocs: &[(u64, Target, u32, i64)]| {
        let mut b = Vec::with_capacity(24 * relocs.len());
        for &(off, t, ty, a) in relocs {
            let symi = match t {
                Target::Section(s) => section_sym[s],
                Target::Debug(s) => debug_sym[s],
                Target::Symbol(id) => sym_index[&id],
            };
            b.u64(off);
            b.u64(((symi as u64) << 32) | ty as u64);
            b.u64(a as u64);
        }
        b
    };
    let rela = |sections: &mut Vec<Shdr>, name: &str, relocs: &[(u64, Target, u32, i64)]| {
        if !relocs.is_empty() {
            let mut r = Shdr::new(&format!(".rela{name}"), SHT_RELA, SHF_INFO_LINK, rela_bytes(relocs), 8);
            r.info = (sections.len() - 1) as u32;
            r.entsize = 24;
            sections.push(r);
        }
    };

    // Section headers.
    let mut sections = vec![null_section()];
    let names = [".text", ".data"];
    let flags = [SHF_ALLOC | SHF_EXECINSTR, SHF_ALLOC | SHF_WRITE];
    for s in 0..2 {
        sections.push(Shdr::new(names[s], SHT_PROGBITS, flags[s], m.sections[s].data.clone(), m.sections[s].align));
        rela(&mut sections, names[s], &text_data_relocs[s]);
    }
    sections.push(Shdr::new(".bss", SHT_NOBITS, SHF_ALLOC | SHF_WRITE, Vec::new(), 1));
    for (i, s) in debug.iter().enumerate() {
        debug_assert_eq!(sections.len(), debug_idx[i] as usize);
        sections.push(debug_shdr(s, s.data.clone()));
        rela(&mut sections, s.name, &debug_relocs[i]);
    }
    let mut symtab = Shdr::new(".symtab", SHT_SYMTAB, 0, sym_bytes(&syms, &str_off), 8);
    symtab.entsize = 24;
    symtab.info = first_global;
    symtab.link = sections.len() as u32 + 1;
    sections.push(symtab);
    let symtab_idx = (sections.len() - 1) as u32;
    sections.push(Shdr::new(".strtab", SHT_STRTAB, 0, strtab_bytes, 1));
    for s in sections.iter_mut() {
        if s.sh_type == SHT_RELA {
            s.link = symtab_idx;
        }
    }
    serialize(1, 0, &[], 0, sections, Vec::new())
}

/// The crt replacement: `_start` sets up `main(argc, argv, envp)` exactly like glibc's crt1
/// (16-byte aligned stack at the call), then `exit_group(main(...))`. Like glibc
/// (`__libc_start_call_main`), main is called through rax, so rax = &main on entry; the other
/// registers keep the kernel's zeros. Returns the code and the offset of the rip-relative
/// displacement to `main`.
fn entry_stub() -> (Vec<u8>, usize) {
    let mut a = Asm::new();
    a.ins(FE_XOR32rr, &[FE_BP, FE_BP]); // mark the outermost frame, like crt1
    a.ins(FE_MOV64rm, &[FE_DI, mem(FE_SP, 0, FE_NOREG, 0)]); // argc
    a.ins(FE_LEA64rm, &[FE_SI, mem(FE_SP, 0, FE_NOREG, 8)]); // argv
    a.ins(FE_LEA64rm, &[FE_DX, mem(FE_SI, 8, FE_DI, 8)]); // envp = argv + argc + 1
    a.ins(FE_AND64ri, &[FE_SP, -16]);
    let call = a.rip_placeholder(FE_LEA64rm, &[FE_AX, 0], 1); // lea rax, [rip + main]
    a.ins(FE_CALLr, &[FE_AX]);
    a.ins(FE_MOV32rr, &[FE_DI, FE_AX]);
    a.ins(FE_MOV32ri, &[FE_AX, 231]); // exit_group
    a.ins(FE_SYSCALL, &[]);
    a.ins(FE_HLT, &[]);
    (a.code, call)
}

pub fn write_executable(m: &ObjModule) -> Result<Vec<u8>, String> {
    let main = m
        .symbol_by_name("main")
        .filter(|&id| m.symbols[id].definition.is_some() && m.symbols[id].binding == Binding::Global);

    let (stub, call_field) = entry_stub();
    let text = &m.sections[0];
    let data = &m.sections[1];

    // Program headers: PT_LOAD (R, ELF+program headers), PT_LOAD (RX, .text), PT_LOAD (RW,
    // .data), PT_GNU_STACK (RWX: the original linked with `-z execstack`).
    let has_data = !data.data.is_empty();
    let nphdr = 3 + has_data as u64;
    let hdr_end = 64 + 56 * nphdr;

    // Places a segment after (file_end, vaddr_end).
    // Like `ld -z separate-code`, every segment starts on a page of its own, in the file
    // and in memory, so the code segment maps nothing but code.
    let place = |file_end: u64, vaddr_end: u64, align: u64| -> (u64, u64) {
        (align_up(file_end, PAGE), align_up(vaddr_end, align.max(PAGE)))
    };
    // .text output section = module .text followed by the entry stub.
    let text_align = text.align.max(16);
    let (text_off, text_addr) = place(hdr_end, EXEC_BASE + hdr_end, text_align);
    let stub_off_in_text = align_up(text.data.len() as u64, 16);
    let text_size = stub_off_in_text + stub.len() as u64;
    let (data_off, data_addr) = place(text_off + text_size, text_addr + text_size, data.align.max(1));
    let addrs = [text_addr, data_addr];

    let none = |_: &str| None;
    let linked = link::relocate(m, &addrs, &none);
    let mut errors = Vec::new();
    if main.is_none() {
        errors.push(link::missing_entry("main"));
    }
    let linked = match linked {
        Ok(l) if errors.is_empty() => l,
        Ok(_) => return Err(errors.join("\n")),
        Err(e) => {
            errors.push(e);
            return Err(errors.join("\n"));
        }
    };
    let (main_sec, main_off) = m.symbols[main.expect("checked above")].definition.expect("defined");
    let main_addr = addrs[main_sec] + main_off;
    let stub_addr = text_addr + stub_off_in_text;
    let mut text_bytes = linked.sections[0].clone();
    text_bytes.resize(stub_off_in_text as usize, 0);
    let mut stub = stub;
    let rel = main_addr as i128 - (stub_addr + call_field as u64 + 4) as i128;
    let rel = i32::try_from(rel).map_err(|_| "(.text): relocation truncated to fit: R_X86_64_PC32 against `main'".to_string())?;
    stub[call_field..call_field + 4].copy_from_slice(&rel.to_le_bytes());
    text_bytes.extend_from_slice(&stub);

    // Program headers.
    let mut ph = Vec::new();
    let mut phdr = |ty: u32, flags: u32, off: u64, vaddr: u64, filesz: u64, memsz: u64, align: u64| {
        ph.u32(ty);
        ph.u32(flags);
        ph.u64(off);
        ph.u64(vaddr);
        ph.u64(vaddr);
        ph.u64(filesz);
        ph.u64(memsz);
        ph.u64(align);
    };
    const PT_LOAD: u32 = 1;
    const PT_GNU_STACK: u32 = 0x6474_e551;
    const PF_X: u32 = 1;
    const PF_W: u32 = 2;
    const PF_R: u32 = 4;
    phdr(PT_LOAD, PF_R, 0, EXEC_BASE, hdr_end, hdr_end, PAGE);
    phdr(PT_LOAD, PF_R | PF_X, text_off, text_addr, text_size, text_size, PAGE);
    if has_data {
        let n = data.data.len() as u64;
        phdr(PT_LOAD, PF_R | PF_W, data_off, data_addr, n, n, PAGE);
    }
    phdr(PT_GNU_STACK, PF_R | PF_W | PF_X, 0, 0, 0, 0, 16);

    // Sections.
    let mut sections = vec![null_section()];
    let mut t = Shdr::new(".text", SHT_PROGBITS, SHF_ALLOC | SHF_EXECINSTR, text_bytes, text_align);
    t.addr = text_addr;
    t.offset = text_off;
    sections.push(t);
    let mut data_shndx = 0u16;
    if has_data {
        let mut d = Shdr::new(".data", SHT_PROGBITS, SHF_ALLOC | SHF_WRITE, linked.sections[1].clone(), data.align.max(1));
        d.addr = data_addr;
        d.offset = data_off;
        sections.push(d);
        data_shndx = 2;
    }
    let file_end = if has_data { data_off + data.data.len() as u64 } else { text_off + text_size };
    // DWARF with final addresses (the CU covers the module's code, not the entry stub).
    if let Some(d) = &m.debug {
        for s in dwarf::build(d, text.data.len() as u64) {
            sections.push(debug_shdr(&s, s.resolved(text_addr)));
        }
    }

    // Symbol table: locals (non-temporary), then globals including `_start`.
    let shndx = |sec: usize| if sec == 0 { 1 } else { data_shndx };
    let mut syms = vec![ElfSym { name: String::new(), info: 0, shndx: 0, value: 0 }];
    for s in &m.symbols {
        if let (Some((sec, off)), false, Binding::Local) = (s.definition, s.temporary, s.binding) {
            syms.push(ElfSym { name: s.name.clone(), info: STT_NOTYPE, shndx: shndx(sec), value: addrs[sec] + off });
        }
    }
    let first_global = syms.len() as u32;
    syms.push(ElfSym { name: "_start".into(), info: (STB_GLOBAL << 4) | STT_FUNC, shndx: 1, value: stub_addr });
    for s in &m.symbols {
        if let (Some((sec, off)), Binding::Global) = (s.definition, s.binding) {
            syms.push(ElfSym { name: s.name.clone(), info: STB_GLOBAL << 4, shndx: shndx(sec), value: addrs[sec] + off });
        }
    }
    let mut strtab = SuffixStrTab::new();
    for s in &syms[1..] {
        strtab.add(&s.name);
    }
    let (strtab_bytes, str_off) = strtab.finish();
    let mut symtab = Shdr::new(".symtab", SHT_SYMTAB, 0, sym_bytes(&syms, &str_off), 8);
    symtab.entsize = 24;
    symtab.info = first_global;
    symtab.link = sections.len() as u32 + 1;
    sections.push(symtab);
    sections.push(Shdr::new(".strtab", SHT_STRTAB, 0, strtab_bytes, 1));

    Ok(serialize(2, stub_addr, &ph, file_end, sections, Vec::new()))
}
