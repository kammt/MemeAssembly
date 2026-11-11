//! Section contents as a list of items, branch relaxation and final module assembly.

use super::regs::Ident;
use super::{Cond, Emitter, Reg};
use crate::backend::fadec::{self, *};
use crate::backend::module::{
    DebugFunction, LineRow, ObjModule, Reloc, RelocKind, Section, SectionKind,
};
use crate::target::Target;

pub(super) enum Item {
    /// Fixed bytes (an instruction or data), possibly with RIP-relative fixups.
    Bytes { data: Vec<u8>, fixups: Vec<Fixup> },
    /// A relative branch to a symbol (short/near decided during relaxation).
    Branch { kind: BranchKind, target: usize },
    /// `.align`
    Align { align: u64 },
}

/// A RIP-relative disp32 at `offset` inside its item: it must hold
/// `symbol + disp - (item start + end)`.
pub(super) struct Fixup {
    pub offset: usize,
    pub symbol: usize,
    pub end: usize,
    pub disp: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BranchKind {
    Jmp,
    Jcc(Cond),
    Call,
}

impl BranchKind {
    pub fn mnemonic(self) -> &'static str {
        match self {
            BranchKind::Jmp => "jmp",
            BranchKind::Call => "call",
            BranchKind::Jcc(c) => match c {
                Cond::E => "je",
                Cond::Ne => "jne",
                Cond::Z => "jz",
                Cond::Nz => "jnz",
                Cond::G => "jg",
                Cond::L => "jl",
                Cond::Ge => "jge",
                Cond::Le => "jle",
                Cond::A => "ja",
                Cond::B => "jb",
            },
        }
    }
    #[cfg(test)]
    pub(super) fn fadec_mnemonic(self) -> u64 {
        self.fadec()
    }
    fn fadec(self) -> u64 {
        match self {
            BranchKind::Jmp => FE_JMP,
            BranchKind::Call => FE_CALL,
            BranchKind::Jcc(c) => match c {
                Cond::E | Cond::Z => FE_JZ,
                Cond::Ne | Cond::Nz => FE_JNZ,
                Cond::G => FE_JG,
                Cond::L => FE_JL,
                Cond::Ge => FE_JGE,
                Cond::Le => FE_JLE,
                Cond::A => FE_JA,
                Cond::B => FE_JC,
            },
        }
    }
    fn len(self, long: bool) -> u64 {
        match (self, long) {
            (_, false) => 2,
            (BranchKind::Jcc(_), true) => 6,
            _ => 5,
        }
    }
}

/// `call reg` / `jmp reg` and the other non-symbol identifiers gas accepts (or rejects) as a
/// branch operand.
pub(super) fn indirect_branch(kind: BranchKind, id: Ident, name: &str) -> Result<Vec<u8>, String> {
    let mn = kind.mnemonic();
    let mismatch = || format!("operand type mismatch for `{mn}'");
    // gas emits a PC-relative relocation against the absolute section for these (`call byte`
    // calls address 1); ObjModule relocations always name a symbol, so this is rejected.
    let absolute = || format!("`{mn} {name}': branch to an absolute address is not supported");
    let modrm_base: u8 = match kind {
        BranchKind::Call => 0xd0,
        BranchKind::Jmp => 0xe0,
        BranchKind::Jcc(_) => {
            return Err(match id {
                Ident::PseudoReg => "invalid use of pseudo-register".to_string(),
                Ident::Operator => format!("invalid use of operator \"{name}\""),
                Ident::UnaryOperator => "invalid expression".to_string(),
                Ident::Constant => absolute(),
                _ => mismatch(),
            })
        }
    };
    match id {
        Ident::Gpr(Reg::R64(n)) => {
            let m = if kind == BranchKind::Call { FE_CALLr } else { FE_JMPr };
            fadec::encode(m, &[gp(n)]).map_err(|_| mismatch())
        }
        Ident::Gpr(Reg::R16(n)) => {
            let mut v = vec![0x66];
            if n >= 8 {
                v.push(0x41);
            }
            v.extend([0xff, modrm_base + (n & 7)]);
            Ok(v)
        }
        // APX r16..r31: REX2 prefix (d5, B4 = bit 4 of the register number, B3 = bit 3).
        Ident::ApxGpr(n, w16) => {
            let mut v = if w16 { vec![0x66] } else { Vec::new() };
            v.extend([0xd5, 0x10 | ((n >> 3) & 1), 0xff, modrm_base + (n & 7)]);
            Ok(v)
        }
        Ident::PseudoReg => Err("invalid use of pseudo-register".to_string()),
        Ident::Operator => Err(format!("invalid use of operator \"{name}\"")),
        Ident::UnaryOperator => Err("invalid expression".to_string()),
        Ident::Constant => Err(absolute()),
        Ident::Gpr(_) | Ident::OtherReg | Ident::Symbol => Err(mismatch()),
    }
}

/// gas's diagnostic for a label definition whose name contains characters that can't be part of
/// a symbol (e.g. `.L[rax]Wins_0:`).
pub(super) fn bad_label_message(name: &str) -> String {
    let head: String = name.chars().take_while(|c| super::symbol_char(*c)).collect();
    if name.starts_with('.') {
        format!("unknown pseudo-op: `{}'", head.to_ascii_lowercase())
    } else {
        format!("invalid character in label `{name}'")
    }
}

/// gas's diagnostic for a symbol reference that isn't a valid symbol name (e.g. the
/// `.L<param>Wins_0` labels of "who would win?" with a pointer parameter): gas parses as much of
/// it as an Intel expression as it can (`sym`, `[...]` index terms, `+`/`-` with a number or
/// symbol) and reports the rest as junk.
pub(super) fn bad_reference_message(name: &str) -> String {
    let sym_len = |s: &str| s.find(|c: char| !super::symbol_char(c)).unwrap_or(s.len());
    let mut r = &name[sym_len(name)..];
    loop {
        r = r.trim_start();
        if r.starts_with('[') {
            match r.find(']') {
                Some(i) => r = &r[i + 1..],
                None => break,
            }
        } else if r.starts_with(['+', '-']) {
            r = r[1..].trim_start();
            if r.starts_with(|c: char| c.is_ascii_digit()) {
                let hex = r.starts_with("0x") || r.starts_with("0X");
                let skip = if hex { 2 } else { 0 };
                let n = r[skip..]
                    .find(|c: char| !(if hex { c.is_ascii_hexdigit() } else { c.is_ascii_digit() }))
                    .map_or(r.len(), |i| i + skip);
                r = &r[n..];
            } else {
                r = &r[sym_len(r)..];
            }
        } else {
            break;
        }
    }
    format!("junk `{r}' after expression")
}

/// The NOP sequences LLVM (Apple's `as`, default CPU penryn) pads code with: NOPs of at most 10
/// bytes, longest first.
fn llvm_nop_fill(out: &mut Vec<u8>, mut pad: u64) {
    while pad > 0 {
        let n = pad.min(10);
        out.extend_from_slice(NOPS[n as usize - 1]);
        pad -= n;
    }
}

/// x86 NOPs of 1 to 11 bytes.
const NOPS: [&[u8]; 11] = [
        &[0x90],
        &[0x66, 0x90],
        &[0x0f, 0x1f, 0x00],
        &[0x0f, 0x1f, 0x40, 0x00],
        &[0x0f, 0x1f, 0x44, 0x00, 0x00],
        &[0x66, 0x0f, 0x1f, 0x44, 0x00, 0x00],
        &[0x0f, 0x1f, 0x80, 0x00, 0x00, 0x00, 0x00],
        &[0x0f, 0x1f, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[0x66, 0x0f, 0x1f, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[0x66, 0x2e, 0x0f, 0x1f, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[0x66, 0x66, 0x2e, 0x0f, 0x1f, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00],
    ];

/// The NOP sequences gas uses to pad code (`.align` in `.text`).
fn nop_fill(out: &mut Vec<u8>, pad: u64) {
    let mut n = pad;
    // More than 7 maximal NOPs: jump over the padding instead.
    if n / 11 > 7 {
        if n - 2 <= 127 {
            out.extend([0xeb, (n - 2) as u8]);
            n -= 2;
        } else {
            out.push(0xe9);
            out.extend(((n - 5) as u32).to_le_bytes());
            n -= 5;
        }
    }
    out.reserve(n as usize);
    let rem = (n % 11) as usize;
    if rem != 0 {
        out.extend_from_slice(NOPS[rem - 1]);
    }
    for _ in 0..n / 11 {
        out.extend_from_slice(NOPS[10]);
    }
}

fn item_size(item: &Item, long: bool, off: u64) -> u64 {
    match item {
        Item::Bytes { data, .. } => data.len() as u64,
        Item::Branch { kind, .. } => kind.len(long),
        Item::Align { align } => (align - off % align) % align,
    }
}

fn offsets(items: &[Item], long: &[bool]) -> Vec<u64> {
    let mut offs = Vec::with_capacity(items.len() + 1);
    let mut off = 0u64;
    for (i, it) in items.iter().enumerate() {
        offs.push(off);
        off += item_size(it, long[i], off);
    }
    offs.push(off);
    offs
}

/// Mach-O atoms: for every item of each section and for every defined symbol, the
/// non-temporary symbol starting the atom it belongs to (`None` before the first one). A label
/// starts a new atom right where it is defined, so a temporary label defined just before a
/// non-temporary one at the same position still belongs to the previous atom, as in LLVM.
fn atoms(em: &Emitter) -> (Vec<Vec<Option<usize>>>, Vec<Option<usize>>) {
    let mut sym_atom = vec![None; em.syms.len()];
    let mut pos_atom = Vec::new();
    for (si, items) in em.sections.iter().enumerate() {
        let mut defs = em.defs[si].iter().peekable();
        let mut cur = None;
        let mut atoms = Vec::with_capacity(items.len());
        for i in 0..=items.len() {
            while let Some(&(_, id)) = defs.next_if(|d| d.0 <= i) {
                let s = &em.syms[id];
                if !s.temporary || s.global {
                    cur = Some(id);
                }
                sym_atom[id] = cur;
            }
            atoms.push(cur);
        }
        pos_atom.push(atoms);
    }
    (pos_atom, sym_atom)
}

pub(super) fn finish(em: Emitter) -> Result<ObjModule, String> {
    // Undefined numeric local labels (gas reports them at the end of assembly).
    let mut errors = Vec::new();
    for s in &em.syms {
        if s.def.is_none() {
            if let Some((n, inst)) = s.name.split_once('\u{2}') {
                if !n.starts_with("dot") && !n.is_empty() {
                    errors.push(format!(
                        "local label `\"{n}\" (instance number {inst} of a fb label)' is not defined"
                    ));
                }
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }

    // Which references the assembler resolves itself; all others become relocations. Only
    // references to symbols defined in the same section can be resolved, and per object format:
    // * ELF (gas): references to global symbols keep their relocation (`call` → PLT32,
    //   `[rip + sym]` → PC32), so they stay interposable; gas does resolve (and relax) `jmp` and
    //   `jcc` to them. References to local symbols are resolved.
    // * Mach-O (LLVM, i.e. Apple's `as`): only references within one atom (the code from a
    //   non-temporary symbol up to the next one) are resolved; other references get external
    //   relocations (the writer rebases those to temporary labels onto their atom).
    // * COFF (mingw gas): all same-section references are resolved.
    let (pos_atom, sym_atom) = atoms(&em);
    let resolved = |si: usize, item: usize, target: usize, kind: Option<BranchKind>| -> bool {
        let s = &em.syms[target];
        if s.def.map(|d| d.0) != Some(si) {
            return false;
        }
        match em.target {
            Target::Windows => true,
            Target::Linux => !s.global || matches!(kind, Some(BranchKind::Jmp | BranchKind::Jcc(_))),
            Target::MacOS => pos_atom[si][item] == sym_atom[target],
        }
    };

    // Relaxation: every branch the assembler resolves starts short (calls are always near);
    // grow to rel32 until all displacements fit (sizes only ever grow, so this terminates).
    let mut longs: Vec<Vec<bool>> = Vec::new();
    let mut offs: Vec<Vec<u64>> = Vec::new();
    for (si, items) in em.sections.iter().enumerate() {
        let mut long: Vec<bool> = items
            .iter()
            .enumerate()
            .map(|(i, it)| match it {
                Item::Branch { kind: BranchKind::Call, .. } => true,
                Item::Branch { kind, target } => !resolved(si, i, *target, Some(*kind)),
                _ => false,
            })
            .collect();
        loop {
            let o = offsets(items, &long);
            let mut changed = false;
            for (i, it) in items.iter().enumerate() {
                if let Item::Branch { target, .. } = it {
                    if !long[i] {
                        let t = o[em.syms[*target].def.unwrap().1] as i64;
                        let d = t - (o[i] as i64 + 2);
                        if !(-128..=127).contains(&d) {
                            long[i] = true;
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                offs.push(o);
                break;
            }
        }
        longs.push(long);
    }

    let sym_addr = |id: usize| -> Option<(usize, u64)> {
        em.syms[id].def.map(|(s, i)| (s, offs[s][i]))
    };

    let mut sections = Vec::new();
    for (si, items) in em.sections.iter().enumerate() {
        let o = &offs[si];
        let mut data: Vec<u8> = Vec::with_capacity(o[items.len()] as usize);
        let mut relocs = Vec::new();
        for (i, it) in items.iter().enumerate() {
            let at = o[i];
            debug_assert_eq!(data.len() as u64, at);
            match it {
                Item::Bytes { data: d, fixups } => {
                    let start = data.len();
                    data.extend_from_slice(d);
                    for f in fixups {
                        match sym_addr(f.symbol) {
                            Some((_, addr)) if resolved(si, i, f.symbol, None) => {
                                let v = addr as i64 + f.disp - (at as i64 + f.end as i64);
                                let v = i32::try_from(v)
                                    .map_err(|_| "relocation out of range".to_string())?;
                                data[start + f.offset..start + f.offset + 4]
                                    .copy_from_slice(&v.to_le_bytes());
                            }
                            _ => relocs.push(Reloc {
                                offset: at + f.offset as u64,
                                kind: RelocKind::Pc32,
                                symbol: f.symbol,
                                addend: f.disp - (f.end - f.offset) as i64,
                            }),
                        }
                    }
                }
                Item::Branch { kind, target } => {
                    let long = longs[si][i];
                    let m = kind.fadec() | if long { FE_JMPL } else { 0 };
                    match sym_addr(*target) {
                        Some((_, addr)) if resolved(si, i, *target, Some(*kind)) => {
                            let b = fadec::encode_branch(m, at, addr)
                                .map_err(|_| format!("cannot encode `{}'", kind.mnemonic()))?;
                            debug_assert_eq!(b.len() as u64, kind.len(long));
                            data.extend_from_slice(&b);
                        }
                        _ => {
                            let mut b = fadec::encode_branch(m, 0, 0)
                                .map_err(|_| format!("cannot encode `{}'", kind.mnemonic()))?;
                            let n = b.len();
                            b[n - 4..].fill(0);
                            relocs.push(Reloc {
                                offset: at + n as u64 - 4,
                                kind: RelocKind::Branch32,
                                symbol: *target,
                                addend: -4,
                            });
                            data.extend_from_slice(&b);
                        }
                    }
                }
                Item::Align { align } => {
                    let pad = item_size(it, false, at);
                    let _ = align;
                    if si == 0 && em.target == Target::MacOS {
                        llvm_nop_fill(&mut data, pad);
                    } else if si == 0 {
                        nop_fill(&mut data, pad);
                    } else {
                        data.resize(data.len() + pad as usize, 0);
                    }
                }
            }
        }
        let mut align = em.aligns[si];
        if em.target == Target::Windows {
            // gas for PE/COFF aligns .text/.data to 16 bytes and pads their size accordingly
            // (.text with NOPs, .data with zeros).
            align = align.max(16);
            if align > 8192 {
                let name = if si == 0 { ".text" } else { ".data" };
                return Err(format!(
                    "section {name}: alignment 2**{} not representable\nnonrepresentable section on output",
                    align.trailing_zeros()
                ));
            }
            let fill = if si == 0 { 0x90 } else { 0 };
            while data.len() % 16 != 0 {
                data.push(fill);
            }
        }
        sections.push(Section {
            kind: if si == 0 { SectionKind::Text } else { SectionKind::Data },
            data,
            relocs,
            align,
        });
    }

    let symbols = em
        .syms
        .iter()
        .map(|s| {
            let mut sym = s.to_symbol();
            sym.definition = s.def.map(|(sec, i)| (sec, offs[sec][i]));
            sym
        })
        .collect();

    let debug = em.debug.map(|d| {
        let mut info = d.info;
        let text = &offs[0];
        info.lines = d
            .rows
            .iter()
            .map(|&(i, file, line)| LineRow { offset: text[i], file, line })
            .collect();
        let end = text[text.len() - 1];
        info.functions = d
            .funcs
            .iter()
            .map(|(name, file, line, b, e)| DebugFunction {
                name: name.clone(),
                file: *file,
                line: *line,
                low: text[*b],
                high: e.map_or(end, |e| text[e]),
            })
            .collect();
        info
    });

    Ok(ObjModule { sections, symbols, debug })
}
