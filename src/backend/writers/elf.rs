//! ELF64 x86-64: relocatable objects (the sections, symbols and relocations GNU as would
//! write) and static, non-PIE, libc-free executables.
use super::super::fadec::*;
use super::super::module::{ObjModule, Reloc, RelocKind};
use super::dwarf::{self, RelTarget};
use super::link;
use super::{align_up, is_global, symbol_groups, Asm, Le, StrTab};

const SHT_PROGBITS: u32 = 1;
const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_RELA: u32 = 4;
const SHF_WRITE: u64 = 1;
const SHF_ALLOC: u64 = 2;
const SHF_EXECINSTR: u64 = 4;
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
            link: 0,
            info: 0,
            align,
            entsize: 0,
            offset: 0,
        }
    }
}

struct ElfSym {
    name: String,
    info: u8,
    shndx: u16,
    value: u64,
}

/// `.symtab` and `.strtab` contents.
fn symtab_bytes(syms: &[ElfSym]) -> (Vec<u8>, Vec<u8>) {
    let mut names = StrTab::new(&[0]);
    let mut b = Vec::with_capacity(24 * syms.len());
    for s in syms {
        b.u32(names.add(&s.name));
        b.u8(s.info);
        b.u8(0);
        b.u16(s.shndx);
        b.u64(s.value);
        b.u64(0);
    }
    (b, names.data)
}

/// Lays out and serializes an ELF file. `sections[0]` must be the NULL section placeholder;
/// `.shstrtab` is appended here. Allocated sections must already have their offsets set
/// (`offset != 0`) when `phdrs` is non-empty, ending at `preplaced_end`; the other sections
/// follow in index order.
fn serialize(e_type: u16, entry: u64, phdrs: &[u8], preplaced_end: u64, mut sections: Vec<Shdr>) -> Vec<u8> {
    sections.push(Shdr::new(".shstrtab", SHT_STRTAB, 0, Vec::new(), 1));
    let shstrndx = sections.len() - 1;
    let mut shstr = StrTab::new(&[0]);
    let names: Vec<u32> = sections.iter().map(|s| shstr.add(&s.name)).collect();
    sections[shstrndx].data = shstr.data;

    let mut pos = if phdrs.is_empty() { 64 } else { preplaced_end };
    for s in sections[1..].iter_mut().filter(|s| s.offset == 0) {
        // File offsets are aligned to at most 8 bytes (the ELF64 file alignment):
        // `sh_addralign` alone carries larger alignments, which matter only once loaded.
        pos = align_up(pos, s.align.clamp(1, 8));
        s.offset = pos;
        pos += s.data.len() as u64;
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
    let mut out = h;
    out.extend_from_slice(phdrs);
    for s in &sections[1..] {
        let end = s.offset as usize + s.data.len();
        if out.len() < end {
            out.resize(end, 0);
        }
        out[s.offset as usize..end].copy_from_slice(&s.data);
    }
    out.pad_to(shoff, 0);
    for (s, name) in sections.iter().zip(names) {
        if s.sh_type == 0 {
            out.extend_from_slice(&[0; 64]);
            continue;
        }
        out.u32(name);
        out.u32(s.sh_type);
        out.u64(s.flags);
        out.u64(s.addr);
        out.u64(s.offset);
        out.u64(s.data.len() as u64);
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

/// Relocation type, symbol table index and addend of a module relocation. Like gas, references
/// to global and undefined symbols stay against the symbol, everything else becomes "section
/// symbol + offset" (and a branch to a local target a plain PC32).
fn adjust(m: &ObjModule, r: &Reloc, sym_index: &[u32]) -> (u32, u32, i64) {
    let s = &m.symbols[r.symbol];
    let global = is_global(s);
    let ty = match r.kind {
        RelocKind::Pc32 => R_X86_64_PC32,
        RelocKind::Branch32 if global => R_X86_64_PLT32,
        RelocKind::Branch32 => R_X86_64_PC32,
        RelocKind::Abs32 => R_X86_64_32,
        RelocKind::Abs64 => R_X86_64_64,
    };
    match s.definition {
        Some((sec, off)) if !global => (section_symbol(sec), ty, r.addend.wrapping_add(off as i64)),
        _ => (sym_index[r.symbol], ty, r.addend),
    }
}

/// Section index of module section `sec` (`.text` = 1, `.data` = 2, then the DWARF sections),
/// which is also the index of its section symbol.
fn section_symbol(sec: usize) -> u32 {
    sec as u32 + 1
}

pub fn write_object(m: &ObjModule) -> Vec<u8> {
    // Sections with contents: .text, .data, the DWARF sections (`-g`).
    let debug = m.debug.as_ref().map(|d| dwarf::build(d, m.sections[0].data.len() as u64)).unwrap_or_default();
    let mut sections = vec![
        null_section(),
        Shdr::new(".text", SHT_PROGBITS, SHF_ALLOC | SHF_EXECINSTR, m.sections[0].data.clone(), m.sections[0].align),
        Shdr::new(".data", SHT_PROGBITS, SHF_ALLOC | SHF_WRITE, m.sections[1].data.clone(), m.sections[1].align),
    ];
    sections.extend(debug.iter().map(|s| Shdr::new(s.name, SHT_PROGBITS, 0, s.data.clone(), 1)));
    let ncontent = sections.len() - 1;

    // Symbol table: NULL, a section symbol per section above, locals, globals.
    let groups = symbol_groups(m);
    let mut syms = vec![ElfSym { name: String::new(), info: 0, shndx: 0, value: 0 }];
    syms.extend((1..=ncontent).map(|i| ElfSym { name: String::new(), info: (STB_LOCAL << 4) | STT_SECTION, shndx: i as u16, value: 0 }));
    let first_global = syms.len() + groups.locals.len();
    let mut sym_index = vec![0u32; m.symbols.len()];
    for &id in groups.locals.iter().chain(&groups.globals).chain(&groups.undefined) {
        let s = &m.symbols[id];
        let bind = if syms.len() < first_global { STB_LOCAL } else { STB_GLOBAL };
        let (shndx, value) = s.definition.map_or((0, 0), |(sec, off)| (section_symbol(sec) as u16, off));
        sym_index[id] = syms.len() as u32;
        syms.push(ElfSym { name: s.name.clone(), info: (bind << 4) | STT_NOTYPE, shndx, value });
    }

    // Relocations as (offset, symbol index, type, addend) per section. DWARF addresses are
    // `.text` + offset, offsets into other debug sections are against their section symbols.
    let mut relocs: Vec<Vec<(u64, u32, u32, i64)>> = m
        .sections
        .iter()
        .map(|sec| {
            sec.relocs
                .iter()
                .map(|r| {
                    let (symi, ty, addend) = adjust(m, r, &sym_index);
                    (r.offset, symi, ty, addend)
                })
                .collect()
        })
        .collect();
    let index_of = |name: &str| sections.iter().position(|s| s.name == name).expect("known debug section") as u32;
    relocs.extend(debug.iter().map(|s| {
        s.relocs
            .iter()
            .map(|r| match r.target {
                RelTarget::Text => (r.offset, section_symbol(0), R_X86_64_64, r.addend),
                RelTarget::Section(n) => (r.offset, index_of(n), R_X86_64_32, r.addend),
            })
            .collect()
    }));

    let symtab_idx = sections.len() as u32;
    let (symtab_bytes, strtab_bytes) = symtab_bytes(&syms);
    let mut symtab = Shdr::new(".symtab", SHT_SYMTAB, 0, symtab_bytes, 8);
    symtab.entsize = 24;
    symtab.info = first_global as u32;
    symtab.link = symtab_idx + 1;
    sections.push(symtab);
    sections.push(Shdr::new(".strtab", SHT_STRTAB, 0, strtab_bytes, 1));
    for (i, rel) in relocs.iter().enumerate().filter(|(_, r)| !r.is_empty()) {
        let mut b = Vec::with_capacity(24 * rel.len());
        for &(off, symi, ty, addend) in rel {
            b.u64(off);
            b.u64(((symi as u64) << 32) | ty as u64);
            b.u64(addend as u64);
        }
        let target = i + 1;
        let mut r = Shdr::new(&format!(".rela{}", sections[target].name), SHT_RELA, SHF_INFO_LINK, b, 8);
        r.info = target as u32;
        r.link = symtab_idx;
        r.entsize = 24;
        sections.push(r);
    }
    serialize(1, 0, &[], 0, sections)
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

    let (linked, main_addr) = link::link_program(m, "main", link::NO_MAIN, &addrs, &|_| None)?;
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
            sections.push(Shdr::new(s.name, SHT_PROGBITS, 0, s.resolved(text_addr), 1));
        }
    }

    // Symbol table: locals (non-temporary), then globals including `_start`.
    let groups = symbol_groups(m);
    let sym = |id: usize, bind: u8| {
        let s = &m.symbols[id];
        let (sec, off) = s.definition.expect("defined");
        ElfSym { name: s.name.clone(), info: bind << 4, shndx: if sec == 0 { 1 } else { data_shndx }, value: addrs[sec] + off }
    };
    let mut syms = vec![ElfSym { name: String::new(), info: 0, shndx: 0, value: 0 }];
    syms.extend(groups.locals.iter().map(|&id| sym(id, STB_LOCAL)));
    let first_global = syms.len() as u32;
    syms.push(ElfSym { name: "_start".into(), info: (STB_GLOBAL << 4) | STT_FUNC, shndx: 1, value: stub_addr });
    syms.extend(groups.globals.iter().map(|&id| sym(id, STB_GLOBAL)));
    let (symtab_bytes, strtab_bytes) = symtab_bytes(&syms);
    let mut symtab = Shdr::new(".symtab", SHT_SYMTAB, 0, symtab_bytes, 8);
    symtab.entsize = 24;
    symtab.info = first_global;
    symtab.link = sections.len() as u32 + 1;
    sections.push(symtab);
    sections.push(Shdr::new(".strtab", SHT_STRTAB, 0, strtab_bytes, 1));

    Ok(serialize(2, stub_addr, &ph, file_end, sections))
}
