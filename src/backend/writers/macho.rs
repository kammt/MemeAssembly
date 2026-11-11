//! Mach-O x86-64: `MH_OBJECT` files with the load commands the LLVM (Apple) assembler writes,
//! and `MH_EXECUTE` images linked directly (dyld + libSystem, entry through `LC_MAIN`).
use super::super::module::{ObjModule, RelocKind};
use super::{align_up, link, put_u64, symbol_groups, uleb, Le, StrTab};

const MH_MAGIC_64: u32 = 0xfeed_facf;
const CPU_TYPE_X86_64: u32 = 0x0100_0007;
const CPU_SUBTYPE_X86_64_ALL: u32 = 3;
/// Set in executables' cpusubtype by ld64 and lld (64-bit libraries).
const CPU_SUBTYPE_LIB64: u32 = 0x8000_0000;
const MH_OBJECT: u32 = 1;
const MH_EXECUTE: u32 = 2;
const MH_NOUNDEFS: u32 = 0x1;
const MH_DYLDLINK: u32 = 0x4;
const MH_TWOLEVEL: u32 = 0x80;
const MH_PIE: u32 = 0x20_0000;

const LC_SEGMENT_64: u32 = 0x19;
const LC_SYMTAB: u32 = 0x2;
const LC_DYSYMTAB: u32 = 0xb;
const LC_LOAD_DYLIB: u32 = 0xc;
const LC_LOAD_DYLINKER: u32 = 0xe;
const LC_UUID: u32 = 0x1b;
const LC_DYLD_INFO_ONLY: u32 = 0x8000_0022;
const LC_MAIN: u32 = 0x8000_0028;
const LC_BUILD_VERSION: u32 = 0x32;
const LC_FUNCTION_STARTS: u32 = 0x26;

const PLATFORM_MACOS: u32 = 1;
/// Minimum OS version recorded in `LC_BUILD_VERSION` (10.14, the first release whose loader
/// understands `LC_BUILD_VERSION`).
const MIN_MACOS: u32 = 0x000a_0e00;
/// SDK version recorded in executables (macOS 15.0; objects record none, like llvm-mc).
const SDK_MACOS: u32 = 0x000f_0000;

const S_ATTR_PURE_INSTRUCTIONS: u32 = 0x8000_0000;
const S_ATTR_SOME_INSTRUCTIONS: u32 = 0x0000_0400;

const N_EXT: u8 = 0x1;
const N_SECT: u8 = 0xe;
const REFERENCED_DYNAMICALLY: u16 = 0x10;

const X86_64_RELOC_UNSIGNED: u32 = 0;
const X86_64_RELOC_SIGNED: u32 = 1;
const X86_64_RELOC_BRANCH: u32 = 2;
const X86_64_RELOC_SIGNED_1: u32 = 6;
const X86_64_RELOC_SIGNED_2: u32 = 7;
const X86_64_RELOC_SIGNED_4: u32 = 8;

const VM_PROT_READ: u32 = 1;
const VM_PROT_WRITE: u32 = 2;
const VM_PROT_EXECUTE: u32 = 4;

const PAGE: u64 = 0x1000;
const PAGEZERO_SIZE: u64 = 0x1_0000_0000;
const DYLD: &str = "/usr/lib/dyld";
const LIBSYSTEM: &str = "/usr/lib/libSystem.B.dylib";

const SECTIONS: [(&str, &str); 2] = [("__text", "__TEXT"), ("__data", "__DATA")];
const HEADER_SIZE: u64 = 32;
const SEGMENT_CMD_SIZE: u64 = 72;
const SECTION_HDR_SIZE: u64 = 80;

fn header(out: &mut Vec<u8>, filetype: u32, ncmds: u32, sizeofcmds: u32, flags: u32) {
    out.u32(MH_MAGIC_64);
    out.u32(CPU_TYPE_X86_64);
    out.u32(if filetype == MH_EXECUTE { CPU_SUBTYPE_X86_64_ALL | CPU_SUBTYPE_LIB64 } else { CPU_SUBTYPE_X86_64_ALL });
    out.u32(filetype);
    out.u32(ncmds);
    out.u32(sizeofcmds);
    out.u32(flags);
    out.u32(0);
}

struct SegmentCmd<'a> {
    name: &'a str,
    vmaddr: u64,
    vmsize: u64,
    fileoff: u64,
    filesize: u64,
    maxprot: u32,
    initprot: u32,
    sections: Vec<SectionHdr>,
}

struct SectionHdr {
    sect: &'static str,
    seg: &'static str,
    addr: u64,
    size: u64,
    offset: u32,
    align: u32,
    reloff: u32,
    nreloc: u32,
    flags: u32,
}

impl SegmentCmd<'_> {
    fn size(&self) -> u64 {
        SEGMENT_CMD_SIZE + SECTION_HDR_SIZE * self.sections.len() as u64
    }

    fn write(&self, out: &mut Vec<u8>) {
        out.u32(LC_SEGMENT_64);
        out.u32(self.size() as u32);
        out.name::<16>(self.name);
        out.u64(self.vmaddr);
        out.u64(self.vmsize);
        out.u64(self.fileoff);
        out.u64(self.filesize);
        out.u32(self.maxprot);
        out.u32(self.initprot);
        out.u32(self.sections.len() as u32);
        out.u32(0);
        for s in &self.sections {
            out.name::<16>(s.sect);
            out.name::<16>(s.seg);
            out.u64(s.addr);
            out.u64(s.size);
            out.u32(s.offset);
            out.u32(s.align);
            out.u32(s.reloff);
            out.u32(s.nreloc);
            out.u32(s.flags);
            out.u32(0);
            out.u32(0);
            out.u32(0);
        }
    }
}

fn section_flags(i: usize) -> u32 {
    if i == 0 {
        S_ATTR_PURE_INSTRUCTIONS | S_ATTR_SOME_INSTRUCTIONS
    } else {
        0
    }
}

fn build_version(out: &mut Vec<u8>, sdk: u32) {
    out.u32(LC_BUILD_VERSION);
    out.u32(24);
    out.u32(PLATFORM_MACOS);
    out.u32(MIN_MACOS);
    out.u32(sdk);
    out.u32(0); // ntools
}

fn symtab_cmd(out: &mut Vec<u8>, symoff: u64, nsyms: usize, stroff: u64, strsize: usize) {
    out.u32(LC_SYMTAB);
    out.u32(24);
    out.u32(symoff as u32);
    out.u32(nsyms as u32);
    out.u32(stroff as u32);
    out.u32(strsize as u32);
}

/// `LC_DYSYMTAB` with only the symbol partition (locals, external definitions, undefined).
fn dysymtab_cmd(out: &mut Vec<u8>, nlocal: usize, nextdef: usize, nundef: usize) {
    out.u32(LC_DYSYMTAB);
    out.u32(80);
    out.u32(0);
    out.u32(nlocal as u32);
    out.u32(nlocal as u32);
    out.u32(nextdef as u32);
    out.u32((nlocal + nextdef) as u32);
    out.u32(nundef as u32);
    for _ in 0..12 {
        out.u32(0);
    }
}

struct Nlist {
    name: String,
    n_type: u8,
    n_sect: u8,
    n_desc: u16,
    value: u64,
}

/// Symbol table and string table (padded to 8 bytes) contents.
fn symtab_bytes(syms: &[Nlist]) -> (Vec<u8>, Vec<u8>) {
    let mut names = StrTab::new(&[0]);
    let mut b = Vec::with_capacity(16 * syms.len());
    for s in syms {
        b.u32(names.add(&s.name));
        b.u8(s.n_type);
        b.u8(s.n_sect);
        b.u16(s.n_desc);
        b.u64(s.value);
    }
    let len = align_up(names.data.len() as u64, 8);
    names.data.pad_to(len, 0);
    (b, names.data)
}

fn reloc_entry(out: &mut Vec<u8>, address: u64, symbolnum: u32, pcrel: bool, length: u32, external: bool, ty: u32) {
    out.u32(address as u32);
    out.u32(symbolnum | (pcrel as u32) << 24 | length << 25 | (external as u32) << 27 | ty << 28);
}

const ABS32_UNSUPPORTED: &str = "32-bit absolute addressing is not supported in 64-bit mode";

pub fn write_object(m: &ObjModule) -> Result<Vec<u8>, String> {
    // Object-file addresses: sections laid out back to back from 0.
    let sizes = [m.sections[0].data.len() as u64, m.sections[1].data.len() as u64];
    let data_addr = align_up(sizes[0], m.sections[1].align);
    let addrs = [0, data_addr];
    let vmsize = data_addr + sizes[1];

    // Symbol table: locals (module order), external definitions and undefined symbols
    // (each sorted by name, as LC_DYSYMTAB documents). Temporary symbols are not emitted.
    let mut groups = symbol_groups(m);
    groups.globals.sort_by(|&a, &b| m.symbols[a].name.cmp(&m.symbols[b].name));
    groups.undefined.sort_by(|&a, &b| m.symbols[a].name.cmp(&m.symbols[b].name));
    let mut sym_index = vec![u32::MAX; m.symbols.len()];
    let mut syms = Vec::new();
    let external = groups.globals.iter().chain(&groups.undefined);
    for (&id, ext) in groups.locals.iter().map(|id| (id, 0)).chain(external.map(|id| (id, N_EXT))) {
        sym_index[id] = syms.len() as u32;
        let s = &m.symbols[id];
        let (n_type, n_sect, value) = match s.definition {
            Some((sec, off)) => (N_SECT | ext, sec as u8 + 1, addrs[sec] + off),
            None => (N_EXT, 0, 0),
        };
        syms.push(Nlist { name: s.name.clone(), n_type, n_sect, n_desc: 0, value });
    }

    // Relocations (emitted in reverse order, like LLVM). A reference to a temporary label is
    // non-extern: it refers to the label's section, the target address being implicit in the
    // field. (Without MH_SUBSECTIONS_VIA_SYMBOLS the linker never splits sections into atoms.)
    let mut contents = [m.sections[0].data.clone(), m.sections[1].data.clone()];
    let mut relocs: [Vec<u8>; 2] = [Vec::new(), Vec::new()];
    for (si, sec) in m.sections.iter().enumerate() {
        for r in sec.relocs.iter().rev() {
            let s = &m.symbols[r.symbol];
            let field = r.offset as usize;
            let buf = &mut contents[si];
            let external = !(s.temporary && s.definition.is_some());
            debug_assert!(!external || sym_index[r.symbol] != u32::MAX, "`{}' not in the symbol table", s.name);
            match r.kind {
                RelocKind::Abs32 => return Err(ABS32_UNSUPPORTED.into()),
                RelocKind::Abs64 => {
                    if external {
                        put_u64(buf, field, r.addend as u64);
                        reloc_entry(&mut relocs[si], r.offset, sym_index[r.symbol], false, 3, true, X86_64_RELOC_UNSIGNED);
                    } else {
                        let (tsec, off) = s.definition.expect("temporary symbols are defined");
                        put_u64(buf, field, (addrs[tsec] + off).wrapping_add(r.addend as u64));
                        reloc_entry(&mut relocs[si], r.offset, tsec as u32 + 1, false, 3, false, X86_64_RELOC_UNSIGNED);
                    }
                }
                RelocKind::Pc32 | RelocKind::Branch32 => {
                    let p = addrs[si] + r.offset;
                    if external {
                        // Like LLVM, the field holds the addend + 4, and SIGNED_j marks an
                        // instruction with j immediate bytes after the field (the addend is
                        // exactly -4 - j); ld64 adds j back to the field and computes
                        // S + field + j - (P + 4 + j).
                        let ty = match (r.kind, -(r.addend + 4)) {
                            (RelocKind::Branch32, _) => X86_64_RELOC_BRANCH,
                            (_, 1) => X86_64_RELOC_SIGNED_1,
                            (_, 2) => X86_64_RELOC_SIGNED_2,
                            (_, 4) => X86_64_RELOC_SIGNED_4,
                            _ => X86_64_RELOC_SIGNED,
                        };
                        let v = i32::try_from(r.addend + 4).map_err(|_| format!("addend out of range for `{}'", s.name))?;
                        buf[field..field + 4].copy_from_slice(&v.to_le_bytes());
                        reloc_entry(&mut relocs[si], r.offset, sym_index[r.symbol], true, 2, true, ty);
                    } else {
                        // The field holds the displacement; the linker finds the target section at
                        // P + 4 + j + field, where j is the SIGNED_j bias. Pick j so that this
                        // is the symbol itself (j = trailing immediate bytes), or at least inside
                        // the target section (its end included, for labels at the end).
                        let (tsec, off) = s.definition.expect("temporary symbols are defined");
                        let target = addrs[tsec] + off;
                        let disp = target as i64 + r.addend - p as i64;
                        let v = i32::try_from(disp).map_err(|_| format!("relocation out of range for `{}'", s.name))?;
                        buf[field..field + 4].copy_from_slice(&v.to_le_bytes());
                        let lands = |j: i64| {
                            let dst = target as i64 + r.addend + 4 + j;
                            dst >= addrs[tsec] as i64 && dst <= (addrs[tsec] + sizes[tsec]) as i64
                        };
                        let candidates = [(0, X86_64_RELOC_SIGNED), (1, X86_64_RELOC_SIGNED_1), (2, X86_64_RELOC_SIGNED_2), (4, X86_64_RELOC_SIGNED_4)];
                        let exact = candidates.iter().find(|(j, _)| *j == -(r.addend + 4));
                        let ty = if r.kind == RelocKind::Branch32 && lands(0) {
                            X86_64_RELOC_BRANCH
                        } else {
                            exact
                                .filter(|(j, _)| lands(*j))
                                .or_else(|| candidates.iter().find(|(j, _)| lands(*j)))
                                .map(|&(_, t)| t)
                                .ok_or_else(|| format!("relocation against `{}' points outside its section", s.name))?
                        };
                        reloc_entry(&mut relocs[si], r.offset, tsec as u32 + 1, true, 2, false, ty);
                    }
                }
            }
        }
    }

    // Layout: header, load commands, section data (padded to 8), relocations, symbols,
    // strings.
    let ncmds = 4;
    let sizeofcmds = SEGMENT_CMD_SIZE + 2 * SECTION_HDR_SIZE + 24 + 24 + 80;
    let data_start = HEADER_SIZE + sizeofcmds;
    let mut pos = data_start + align_up(vmsize, 8);
    let mut reloff = [0u64; 2];
    for i in 0..2 {
        if !relocs[i].is_empty() {
            reloff[i] = pos;
            pos += relocs[i].len() as u64;
        }
    }
    let symoff = pos;
    let (symbols, strtab) = symtab_bytes(&syms);
    let stroff = symoff + 16 * syms.len() as u64;

    let segment = SegmentCmd {
        name: "",
        vmaddr: 0,
        vmsize,
        fileoff: data_start,
        filesize: vmsize,
        maxprot: VM_PROT_READ | VM_PROT_WRITE | VM_PROT_EXECUTE,
        initprot: VM_PROT_READ | VM_PROT_WRITE | VM_PROT_EXECUTE,
        sections: (0..2)
            .map(|i| SectionHdr {
                sect: SECTIONS[i].0,
                seg: SECTIONS[i].1,
                addr: addrs[i],
                size: sizes[i],
                offset: (data_start + addrs[i]) as u32,
                align: m.sections[i].align.trailing_zeros(),
                reloff: reloff[i] as u32,
                nreloc: (relocs[i].len() / 8) as u32,
                flags: section_flags(i),
            })
            .collect(),
    };

    let mut out = Vec::new();
    header(&mut out, MH_OBJECT, ncmds, sizeofcmds as u32, 0);
    segment.write(&mut out);
    build_version(&mut out, 0);
    symtab_cmd(&mut out, symoff, syms.len(), stroff, strtab.len());
    dysymtab_cmd(&mut out, groups.locals.len(), groups.globals.len(), groups.undefined.len());
    debug_assert_eq!(out.len() as u64, data_start);
    for i in 0..2 {
        out.pad_to(data_start + addrs[i], 0);
        out.extend_from_slice(&contents[i]);
    }
    out.pad_to(data_start + align_up(vmsize, 8), 0);
    for r in &relocs {
        out.extend_from_slice(r);
    }
    out.extend_from_slice(&symbols);
    out.extend_from_slice(&strtab);
    Ok(out)
}

/// Export trie with terminal nodes for `names` (regular exports at image offsets). Nodes are
/// written root first, children in order, every node fully expanded (one edge per name from
/// the root, which is valid as no exported name is a prefix of another one here).
fn export_trie(exports: &[(String, u64)]) -> Vec<u8> {
    // Root node: no terminal info, one edge per export to a leaf.
    // Offsets of children are ULEB128 and depend on the root size, so iterate to a fixpoint.
    let leaf = |addr: u64| {
        let mut info = Vec::new();
        uleb(&mut info, 0); // flags: EXPORT_SYMBOL_FLAGS_KIND_REGULAR
        uleb(&mut info, addr);
        let mut node = Vec::new();
        uleb(&mut node, info.len() as u64);
        node.extend_from_slice(&info);
        node.push(0); // no children
        node
    };
    let leaves: Vec<Vec<u8>> = exports.iter().map(|(_, a)| leaf(*a)).collect();
    let mut root_size = 0usize;
    loop {
        let mut root = vec![0u8, exports.len() as u8];
        let mut off = root_size;
        for (i, (name, _)) in exports.iter().enumerate() {
            root.extend_from_slice(name.as_bytes());
            root.push(0);
            uleb(&mut root, off as u64);
            off += leaves[i].len();
        }
        if root.len() == root_size {
            let mut out = root;
            for l in &leaves {
                out.extend_from_slice(l);
            }
            let len = align_up(out.len() as u64, 8);
            out.pad_to(len, 0);
            return out;
        }
        root_size = root.len();
    }
}

/// Rebase opcodes: `REBASE_TYPE_POINTER` at each (segment index, segment offset).
fn rebase_info(sites: &[(u8, u64)]) -> Vec<u8> {
    const REBASE_OPCODE_DONE: u8 = 0x00;
    const REBASE_OPCODE_SET_TYPE_IMM: u8 = 0x10;
    const REBASE_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB: u8 = 0x20;
    const REBASE_OPCODE_DO_REBASE_IMM_TIMES: u8 = 0x50;
    const REBASE_TYPE_POINTER: u8 = 1;
    if sites.is_empty() {
        return Vec::new();
    }
    let mut b = vec![REBASE_OPCODE_SET_TYPE_IMM | REBASE_TYPE_POINTER];
    for &(seg, off) in sites {
        b.push(REBASE_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB | seg);
        uleb(&mut b, off);
        b.push(REBASE_OPCODE_DO_REBASE_IMM_TIMES | 1);
    }
    b.push(REBASE_OPCODE_DONE);
    let len = align_up(b.len() as u64, 8);
    b.pad_to(len, 0);
    b
}

/// Deterministic UUID derived from the image contents (FNV-1a, two lanes), marked as a
/// version 4 / RFC 4122 variant UUID.
fn uuid(bytes: &[u8]) -> [u8; 16] {
    let mut h = [0xcbf2_9ce4_8422_2325u64, 0x8422_2325_cbf2_9ce4u64];
    for &b in bytes {
        for (k, x) in h.iter_mut().enumerate() {
            *x ^= b as u64 + k as u64;
            *x = x.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    let mut u = [0u8; 16];
    u[..8].copy_from_slice(&h[0].to_le_bytes());
    u[8..].copy_from_slice(&h[1].to_le_bytes());
    u[6] = (u[6] & 0x0f) | 0x40;
    u[8] = (u[8] & 0x3f) | 0x80;
    u
}

fn padded_cmd_string(out: &mut Vec<u8>, cmd: u32, header_words: &[u32], s: &str) {
    let fixed = 8 + 4 * header_words.len() as u64;
    let size = align_up(fixed + s.len() as u64 + 1, 8);
    out.u32(cmd);
    out.u32(size as u32);
    for &w in header_words {
        out.u32(w);
    }
    let start = out.len() as u64 - fixed;
    out.extend_from_slice(s.as_bytes());
    out.pad_to(start + size, 0);
}

fn dylinker_cmd_size() -> u64 {
    align_up(12 + DYLD.len() as u64 + 1, 8)
}

fn dylib_cmd_size() -> u64 {
    align_up(24 + LIBSYSTEM.len() as u64 + 1, 8)
}

pub fn write_executable(m: &ObjModule) -> Result<Vec<u8>, String> {
    let text = &m.sections[0];
    let data = &m.sections[1];
    let has_data = !data.data.is_empty();
    if m.sections.iter().flat_map(|s| &s.relocs).any(|r| r.kind == RelocKind::Abs32) {
        return Err(ABS32_UNSUPPORTED.into());
    }

    // Load commands: __PAGEZERO, __TEXT(__text), [__DATA(__data)], __LINKEDIT,
    // LC_DYLD_INFO_ONLY, LC_SYMTAB, LC_DYSYMTAB, LC_LOAD_DYLINKER, LC_UUID, LC_BUILD_VERSION,
    // LC_MAIN, LC_FUNCTION_STARTS, LC_LOAD_DYLIB.
    let nsegs = 3 + has_data as u64;
    let sizeofcmds = SEGMENT_CMD_SIZE * nsegs
        + SECTION_HDR_SIZE * (1 + has_data as u64)
        + 48
        + 24
        + 80
        + dylinker_cmd_size()
        + 24
        + 24
        + 24
        + 16
        + dylib_cmd_size();
    let ncmds = nsegs as u32 + 9;

    // Alignment above a page can't be honoured by the loader (segments are page aligned).
    let text_align = text.align.clamp(16, PAGE);
    let data_align = data.align.clamp(1, PAGE);
    let text_base = PAGEZERO_SIZE;
    let text_off = align_up(HEADER_SIZE + sizeofcmds, text_align);
    let text_seg_size = align_up(text_off + text.data.len() as u64, PAGE);
    let data_off = text_seg_size; // __DATA starts on the next page (file offset = vm offset)
    let data_seg_size = if has_data { align_up(data.data.len() as u64, PAGE) } else { 0 };
    let linkedit_off = data_off + data_seg_size;
    let addrs = [text_base + text_off, text_base + data_off];

    let missing = "undefined reference to `_main' (entry point)";
    let (linked, main_addr) = link::link_program(m, "_main", missing, &addrs, &|_| None)?;
    let entryoff = main_addr - text_base;

    // Absolute pointers must be rebased when ASLR slides the image. The __TEXT segment is
    // read-only, so with a pointer in code the image is not position independent instead.
    let text_pointers = linked.abs64_sites.iter().any(|&(s, _)| s == 0);
    let mut rebase_sites: Vec<(u8, u64)> = Vec::new();
    if !text_pointers {
        let data_seg_index = 2; // __PAGEZERO, __TEXT, __DATA
        rebase_sites = linked.abs64_sites.iter().map(|&(_, off)| (data_seg_index, off)).collect();
        rebase_sites.sort();
    }
    let rebase = rebase_info(&rebase_sites);

    // Symbols: locals, then external definitions sorted by name (with __mh_execute_header).
    let groups = symbol_groups(m);
    let sym = |id: usize, n_type: u8| {
        let s = &m.symbols[id];
        let (sec, off) = s.definition.expect("defined");
        Nlist { name: s.name.clone(), n_type, n_sect: sec as u8 + 1, n_desc: 0, value: addrs[sec] + off }
    };
    let mut syms: Vec<Nlist> = groups.locals.iter().map(|&id| sym(id, N_SECT)).collect();
    let nlocal = syms.len();
    let mut ext: Vec<Nlist> = vec![Nlist {
        name: "__mh_execute_header".into(),
        n_type: N_SECT | N_EXT,
        n_sect: 1,
        n_desc: REFERENCED_DYNAMICALLY,
        value: text_base,
    }];
    ext.extend(groups.globals.iter().map(|&id| sym(id, N_SECT | N_EXT)));
    ext.sort_by(|a, b| a.name.cmp(&b.name));
    let nextdef = ext.len();
    let exports: Vec<(String, u64)> = ext.iter().map(|s| (s.name.clone(), s.value - text_base)).collect();
    syms.extend(ext);
    let trie = export_trie(&exports);
    let (symbols, strtab) = symtab_bytes(&syms);

    // Function starts: the code symbols' addresses as ULEB128 deltas (the first one from the
    // image start), zero terminated.
    let mut starts: Vec<u64> = m
        .symbols
        .iter()
        .filter(|s| !s.temporary)
        .filter_map(|s| match s.definition {
            Some((0, off)) => Some(addrs[0] + off),
            _ => None,
        })
        .collect();
    starts.sort_unstable();
    starts.dedup();
    let mut function_starts = Vec::new();
    let mut prev = text_base;
    for a in starts {
        uleb(&mut function_starts, a - prev);
        prev = a;
    }
    if !function_starts.is_empty() {
        function_starts.push(0);
        let len = align_up(function_starts.len() as u64, 8);
        function_starts.pad_to(len, 0);
    }

    // __LINKEDIT: rebase info, export trie, function starts, symbol table, string table.
    let rebase_off = linkedit_off;
    let export_off = rebase_off + rebase.len() as u64;
    let function_starts_off = export_off + trie.len() as u64;
    let symoff = function_starts_off + function_starts.len() as u64;
    let stroff = symoff + symbols.len() as u64;
    let linkedit_size = stroff + strtab.len() as u64 - linkedit_off;

    let mut segments = vec![
        SegmentCmd { name: "__PAGEZERO", vmaddr: 0, vmsize: PAGEZERO_SIZE, fileoff: 0, filesize: 0, maxprot: 0, initprot: 0, sections: vec![] },
        SegmentCmd {
            name: "__TEXT",
            vmaddr: text_base,
            vmsize: text_seg_size,
            fileoff: 0,
            filesize: text_seg_size,
            maxprot: VM_PROT_READ | VM_PROT_EXECUTE,
            initprot: VM_PROT_READ | VM_PROT_EXECUTE,
            sections: vec![SectionHdr {
                sect: "__text",
                seg: "__TEXT",
                addr: addrs[0],
                size: text.data.len() as u64,
                offset: text_off as u32,
                align: text_align.trailing_zeros(),
                reloff: 0,
                nreloc: 0,
                flags: section_flags(0),
            }],
        },
    ];
    if has_data {
        segments.push(SegmentCmd {
            name: "__DATA",
            vmaddr: text_base + data_off,
            vmsize: data_seg_size,
            fileoff: data_off,
            filesize: data_seg_size,
            maxprot: VM_PROT_READ | VM_PROT_WRITE,
            initprot: VM_PROT_READ | VM_PROT_WRITE,
            sections: vec![SectionHdr {
                sect: "__data",
                seg: "__DATA",
                addr: addrs[1],
                size: data.data.len() as u64,
                offset: data_off as u32,
                align: data_align.trailing_zeros(),
                reloff: 0,
                nreloc: 0,
                flags: 0,
            }],
        });
    }
    segments.push(SegmentCmd {
        name: "__LINKEDIT",
        vmaddr: text_base + linkedit_off,
        vmsize: align_up(linkedit_size, PAGE),
        fileoff: linkedit_off,
        filesize: linkedit_size,
        maxprot: VM_PROT_READ,
        initprot: VM_PROT_READ,
        sections: vec![],
    });

    let mut flags = MH_NOUNDEFS | MH_DYLDLINK | MH_TWOLEVEL;
    if !text_pointers {
        flags |= MH_PIE;
    }
    let mut out = Vec::new();
    header(&mut out, MH_EXECUTE, ncmds, sizeofcmds as u32, flags);
    for s in &segments {
        s.write(&mut out);
    }
    out.u32(LC_DYLD_INFO_ONLY);
    out.u32(48);
    out.u32(if rebase.is_empty() { 0 } else { rebase_off as u32 });
    out.u32(rebase.len() as u32);
    for _ in 0..6 {
        out.u32(0); // bind, weak bind, lazy bind
    }
    out.u32(export_off as u32);
    out.u32(trie.len() as u32);
    symtab_cmd(&mut out, symoff, syms.len(), stroff, strtab.len());
    dysymtab_cmd(&mut out, nlocal, nextdef, 0);
    padded_cmd_string(&mut out, LC_LOAD_DYLINKER, &[12], DYLD);
    out.u32(LC_UUID);
    out.u32(24);
    let uuid_at = out.len();
    out.extend_from_slice(&[0; 16]);
    build_version(&mut out, SDK_MACOS);
    out.u32(LC_MAIN);
    out.u32(24);
    out.u64(entryoff);
    out.u64(0); // stack size: default
    out.u32(LC_FUNCTION_STARTS);
    out.u32(16);
    out.u32(function_starts_off as u32);
    out.u32(function_starts.len() as u32);
    // dylib: name offset, timestamp, current version, compatibility version.
    padded_cmd_string(&mut out, LC_LOAD_DYLIB, &[24, 2, 0x0505_0100, 0x0001_0000], LIBSYSTEM);
    debug_assert_eq!(out.len() as u64, HEADER_SIZE + sizeofcmds);

    out.pad_to(text_off, 0);
    out.extend_from_slice(&linked.sections[0]);
    out.pad_to(text_seg_size, 0);
    if has_data {
        out.extend_from_slice(&linked.sections[1]);
        out.pad_to(data_off + data_seg_size, 0);
    }
    out.extend_from_slice(&rebase);
    out.extend_from_slice(&trie);
    out.extend_from_slice(&function_starts);
    out.extend_from_slice(&symbols);
    out.extend_from_slice(&strtab);
    let id = uuid(&out);
    out[uuid_at..uuid_at + 16].copy_from_slice(&id);
    Ok(out)
}
