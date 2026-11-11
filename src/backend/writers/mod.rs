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
//! Object files mirror what GNU as (or, for Mach-O, the Apple/LLVM assembler) emits for the
//! same source. Executables are linked here directly (no crt/libc): see [`link`] for the shared
//! relocation logic and the format modules for their entry stubs.
use super::module::{Binding, ObjModule, RelocKind, Symbol};
use super::OutputKind;
use crate::target::Target;

mod coff;
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
            if r.offset.checked_add(field_size(r.kind)).map_or(true, |end| end > s.data.len() as u64) {
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
    for st in &m.stabs {
        if let super::module::StabValue::Symbol(id) = st.value {
            if id >= m.symbols.len() {
                return internal(format!("stab referencing symbol #{id}"));
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

/// String table with the layout BFD (`elf-strtab.c`) produces: identical strings are stored
/// once and a string that is a suffix of another one shares its storage. Offsets of the other
/// strings follow insertion order, starting after the leading NUL.
pub(crate) struct SuffixStrTab {
    strings: Vec<String>,
}

impl SuffixStrTab {
    pub fn new() -> Self {
        SuffixStrTab { strings: Vec::new() }
    }

    pub fn add(&mut self, s: &str) {
        if !s.is_empty() && !self.strings.iter().any(|x| x == s) {
            self.strings.push(s.to_string());
        }
    }

    /// Returns the table bytes and a lookup function result as a map.
    pub fn finish(self) -> (Vec<u8>, std::collections::HashMap<String, u32>) {
        use std::collections::HashMap;
        // BFD sorts by reversed string, longest first among equal suffixes; a string becomes a
        // suffix of the closest preceding (longer) entry that ends with it.
        let mut order: Vec<usize> = (0..self.strings.len()).collect();
        let rev = |s: &str| s.bytes().rev().collect::<Vec<u8>>();
        order.sort_by(|&a, &b| {
            let (ra, rb) = (rev(&self.strings[a]), rev(&self.strings[b]));
            // Compare reversed strings; when one is a prefix of the other, the longer first.
            let n = ra.len().min(rb.len());
            ra[..n].cmp(&rb[..n]).then(rb.len().cmp(&ra.len()))
        });
        let mut host: Vec<Option<usize>> = vec![None; self.strings.len()];
        let mut last: Option<usize> = None;
        for &i in &order {
            if let Some(l) = last {
                if self.strings[l].len() > self.strings[i].len() && self.strings[l].ends_with(&self.strings[i]) {
                    host[i] = Some(l);
                    continue;
                }
            }
            last = Some(i);
        }
        let mut bytes = vec![0u8];
        let mut offsets: HashMap<String, u32> = HashMap::new();
        for (i, s) in self.strings.iter().enumerate() {
            if host[i].is_none() {
                offsets.insert(s.clone(), bytes.len() as u32);
                bytes.extend_from_slice(s.as_bytes());
                bytes.push(0);
            }
        }
        for (i, s) in self.strings.iter().enumerate() {
            if let Some(h) = host[i] {
                let hs = &self.strings[h];
                let off = offsets[hs] + (hs.len() - s.len()) as u32;
                offsets.insert(s.clone(), off);
            }
        }
        offsets.insert(String::new(), 0);
        (bytes, offsets)
    }
}
