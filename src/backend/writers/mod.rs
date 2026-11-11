//! Object/executable file writers. OWNED BY: writers agent.
//!
//! [`write`] turns the format-independent [`ObjModule`] produced by the assembler into the bytes
//! of the file that `gcc` produced in the original compiler:
//!
//! | target  | `OutputKind::Object` (`gcc -c`)      | `OutputKind::Executable` (`gcc`)          |
//! |---------|--------------------------------------|-------------------------------------------|
//! | Linux   | ELF64 relocatable ([`elf`])          | static non-PIE ELF64 executable ([`elf`]) |
//! | Windows | COFF x86-64 object ([`coff`])        | PE32+ console executable ([`pe`])         |
//! | macOS   | Mach-O `MH_OBJECT` ([`macho`])       | Mach-O `MH_EXECUTE` ([`macho`])           |
//!
//! Object files carry the same sections, symbols and relocations as GNU as (or, for Mach-O,
//! the Apple/LLVM assembler) would produce for the same source, in a simpler layout.
//! Executables are linked here directly (no crt/libc): see [`link`] for the shared relocation
//! logic and the format modules for their entry stubs.
use std::collections::HashMap;

use super::module::{Binding, ObjModule, RelocKind, Symbol, SymbolId};
use super::OutputKind;
use crate::target::Target;

mod coff;
mod dwarf;
mod elf;
mod link;
mod macho;
mod pe;

#[cfg(test)]
mod tests;

pub fn write(module: &ObjModule, target: Target, kind: OutputKind) -> Result<Vec<u8>, String> {
    validate(module)?;
    match (target, kind) {
        (Target::Linux, OutputKind::Object) => Ok(elf::write_object(module)),
        (Target::Linux, OutputKind::Executable) => elf::write_executable(module),
        (Target::Windows, OutputKind::Object) => coff::write_object(module),
        (Target::Windows, OutputKind::Executable) => pe::write_executable(module),
        (Target::MacOS, OutputKind::Object) => macho::write_object(module),
        (Target::MacOS, OutputKind::Executable) => macho::write_executable(module),
    }
}

/// Checks the structural invariants of the module contract (see `module.rs`), so that the
/// format writers can index freely.
fn validate(m: &ObjModule) -> Result<(), String> {
    use super::module::SectionKind;
    let internal = |msg: String| Err(format!("internal error: malformed object module: {msg}"));
    if m.sections.len() != 2
        || m.sections[0].kind != SectionKind::Text
        || m.sections[1].kind != SectionKind::Data
    {
        return internal("expected exactly the sections .text and .data".into());
    }
    for (i, s) in m.sections.iter().enumerate() {
        if !s.align.is_power_of_two() {
            return internal(format!("section {i} has alignment {}", s.align));
        }
        for r in &s.relocs {
            if r.symbol >= m.symbols.len() {
                return internal(format!("relocation against symbol #{}", r.symbol));
            }
            if r.offset.checked_add(field_size(r.kind)).is_none_or(|end| end > s.data.len() as u64) {
                return internal(format!("relocation at {:#x} outside section {i}", r.offset));
            }
        }
    }
    for s in &m.symbols {
        if let Some((sec, off)) = s.definition {
            if sec >= m.sections.len() || off > m.sections[sec].data.len() as u64 {
                return internal(format!("symbol `{}' defined outside its section", s.name));
            }
        }
    }
    if let Some(d) = &m.debug {
        let text_len = m.sections[0].data.len() as u64;
        if d.files.is_empty() && (!d.lines.is_empty() || !d.functions.is_empty()) {
            return internal("debug info without source files".into());
        }
        let mut prev = 0;
        for r in &d.lines {
            if r.file >= d.files.len() || r.offset < prev || r.offset > text_len {
                return internal(format!("bad line row {r:?}"));
            }
            prev = r.offset;
        }
        for f in &d.functions {
            if f.file >= d.files.len() || f.low > f.high || f.high > text_len {
                return internal(format!("bad debug function {f:?}"));
            }
        }
    }
    Ok(())
}

/// Size in bytes of the field a relocation patches.
pub(crate) fn field_size(kind: RelocKind) -> u64 {
    match kind {
        RelocKind::Pc32 | RelocKind::Branch32 | RelocKind::Abs32 => 4,
        RelocKind::Abs64 => 8,
    }
}

/// Whether a relocation of this kind is PC-relative.
pub(crate) fn is_pcrel(kind: RelocKind) -> bool {
    matches!(kind, RelocKind::Pc32 | RelocKind::Branch32)
}

/// A symbol that object files list by name (as opposed to temporaries, which are folded into
/// section-relative references). Undefined symbols are always "global" (gas semantics).
pub(crate) fn is_global(s: &Symbol) -> bool {
    s.definition.is_none() || s.binding == Binding::Global
}

pub(crate) fn align_up(v: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two());
    (v + align - 1) & !(align - 1)
}

/// Section names as gas uses them (for diagnostics in the ld style `(.text+0x1b)`).
pub(crate) fn section_name(index: usize) -> &'static str {
    match index {
        0 => ".text",
        _ => ".data",
    }
}

/// Little-endian byte buffer helpers.
pub(crate) trait Le {
    fn u8(&mut self, v: u8);
    fn u16(&mut self, v: u16);
    fn u32(&mut self, v: u32);
    fn u64(&mut self, v: u64);
    fn pad_to(&mut self, len: u64, fill: u8);
    fn name<const N: usize>(&mut self, s: &str);
}

impl Le for Vec<u8> {
    fn u8(&mut self, v: u8) {
        self.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.extend_from_slice(&v.to_le_bytes());
    }
    fn pad_to(&mut self, len: u64, fill: u8) {
        let len = usize::try_from(len).expect("file too large");
        assert!(self.len() <= len, "layout error: {} > {}", self.len(), len);
        self.resize(len, fill);
    }
    /// Fixed-size, NUL padded name field (section/segment names).
    fn name<const N: usize>(&mut self, s: &str) {
        let mut b = [0u8; N];
        b[..s.len()].copy_from_slice(s.as_bytes());
        self.extend_from_slice(&b);
    }
}

pub(crate) fn put_u32(buf: &mut [u8], off: usize, v: u32) {
    buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

pub(crate) fn put_u64(buf: &mut [u8], off: usize, v: u64) {
    buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

/// Encodes a short instruction sequence with fadec. Every element is `(mnemonic, operands)`;
/// branches go through [`Asm::branch`].
pub(crate) struct Asm {
    pub code: Vec<u8>,
}

impl Asm {
    pub fn new() -> Asm {
        Asm { code: Vec::new() }
    }

    pub fn ins(&mut self, mnem: u64, ops: &[super::fadec::FeOp]) -> &mut Self {
        let bytes = super::fadec::encode(mnem, ops)
            .unwrap_or_else(|_| panic!("fadec failed to encode stub instruction {mnem:#x}"));
        self.code.extend_from_slice(&bytes);
        self
    }

    /// Emits a rel32 call/jmp (`mnem` = `FE_CALL`/`FE_JMP`) whose displacement is filled in
    /// later; returns the offset of the 4-byte displacement field.
    pub fn branch_placeholder(&mut self, mnem: u64) -> usize {
        let at = self.code.len() as u64;
        let bytes = super::fadec::encode_branch(mnem | super::fadec::FE_JMPL, at, at)
            .expect("fadec failed to encode stub branch");
        self.code.extend_from_slice(&bytes);
        self.code.len() - 4
    }

    /// Emits an instruction with a `[rip + disp32]` memory operand at operand position
    /// `mem_index` and returns the offset of the displacement field (left zero). The
    /// displacement must be the last 4 bytes of the instruction (no trailing immediate).
    pub fn rip_placeholder(&mut self, mnem: u64, ops: &[super::fadec::FeOp], mem_index: usize) -> usize {
        use super::fadec::{mem, FE_IP, FE_NOREG};
        let mut o = ops.to_vec();
        // fadec's rip-relative offsets are relative to the instruction start; a large value
        // forces the disp32 form.
        o[mem_index] = mem(FE_IP, 0, FE_NOREG, 0x1000_0000);
        let bytes = super::fadec::encode(mnem, &o).expect("fadec failed to encode stub instruction");
        let disp = 0x1000_0000i32.wrapping_sub(bytes.len() as i32);
        assert_eq!(&bytes[bytes.len() - 4..], &disp.to_le_bytes(), "disp32 must end the instruction");
        self.code.extend_from_slice(&bytes[..bytes.len() - 4]);
        self.code.extend_from_slice(&[0; 4]);
        self.code.len() - 4
    }
}

/// A string table: each distinct string once, NUL terminated, in insertion order, after a
/// format-specific prefix (ELF/Mach-O: a NUL, COFF: the 4-byte size field). Offsets count from
/// the start of the table, prefix included; the empty string is offset 0.
pub(crate) struct StrTab {
    pub data: Vec<u8>,
    offsets: HashMap<String, u32>,
}

impl StrTab {
    pub fn new(prefix: &[u8]) -> StrTab {
        StrTab { data: prefix.to_vec(), offsets: HashMap::from([(String::new(), 0)]) }
    }

    pub fn add(&mut self, s: &str) -> u32 {
        if let Some(&o) = self.offsets.get(s) {
            return o;
        }
        let o = self.data.len() as u32;
        self.data.extend_from_slice(s.as_bytes());
        self.data.push(0);
        self.offsets.insert(s.to_string(), o);
        o
    }
}

/// ULEB128.
pub(crate) fn uleb(b: &mut Vec<u8>, mut v: u64) {
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            b.push(byte);
            return;
        }
        b.push(byte | 0x80);
    }
}

/// The symbols an object's symbol table lists, each group in module order: defined
/// non-temporary locals, defined globals, and undefined symbols that are referenced or declared
/// global (an unused `.extern` is dropped, like gas does).
pub(crate) struct SymbolGroups {
    pub locals: Vec<SymbolId>,
    pub globals: Vec<SymbolId>,
    pub undefined: Vec<SymbolId>,
}

pub(crate) fn symbol_groups(m: &ObjModule) -> SymbolGroups {
    let mut referenced = vec![false; m.symbols.len()];
    for r in m.sections.iter().flat_map(|s| &s.relocs) {
        referenced[r.symbol] = true;
    }
    let mut g = SymbolGroups { locals: Vec::new(), globals: Vec::new(), undefined: Vec::new() };
    for (id, s) in m.symbols.iter().enumerate() {
        match (s.definition, s.binding) {
            (Some(_), Binding::Local) if !s.temporary => g.locals.push(id),
            (Some(_), Binding::Global) => g.globals.push(id),
            (None, b) if referenced[id] || b == Binding::Global => g.undefined.push(id),
            _ => {}
        }
    }
    g
}
