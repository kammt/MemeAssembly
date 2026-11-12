//! Section contents as a list of items, branch relaxation and final module assembly.
//!
//! The [`Emitter`] records each section as [`Item`]s whose size may still change (branches
//! not yet relaxed, alignment padding). [`finish`] decides per object format which references
//! the assembler resolves itself, relaxes the branches, then writes the bytes and relocations
//! and converts symbol positions and line rows from item indices to section offsets.

use super::x86::{self, BranchKind};
use super::Emitter;
use crate::backend::module::{
    DebugFunction, LineRow, ObjModule, Reloc, RelocKind, Section, SectionKind,
};
use crate::target::Target;

/// One piece of a section.
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

/// The size of `item` at offset `off` (`long`: a branch encoded with rel32).
fn item_size(item: &Item, long: bool, off: u64) -> u64 {
    match item {
        Item::Bytes { data, .. } => data.len() as u64,
        Item::Branch { kind, .. } => kind.len(long),
        Item::Align { align } => (align - off % align) % align,
    }
}

/// The offset of every item, plus the section's size at the end.
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

/// Lays out the emitter's sections and produces the module (see the module docs). Fails on
/// undefined numeric local labels, out-of-range references and alignments COFF can't represent.
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
                    let end = at + kind.len(long);
                    match sym_addr(*target) {
                        Some((_, addr)) if resolved(si, i, *target, Some(*kind)) => {
                            let rel = i32::try_from(addr as i64 - end as i64)
                                .map_err(|_| "branch out of range".to_string())?;
                            data.extend_from_slice(&x86::branch(*kind, long, rel));
                        }
                        _ => {
                            relocs.push(Reloc {
                                offset: end - 4,
                                kind: RelocKind::Branch32,
                                symbol: *target,
                                addend: -4,
                            });
                            data.extend_from_slice(&x86::branch(*kind, long, 0));
                        }
                    }
                }
                Item::Align { .. } => {
                    // Only `-O-s` aligns, after the final `ret`: the padding never runs, so
                    // `.text` is simply filled with single-byte NOPs.
                    let pad = item_size(it, false, at) as usize;
                    data.resize(data.len() + pad, if si == 0 { 0x90 } else { 0 });
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
            while !data.len().is_multiple_of(16) {
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
