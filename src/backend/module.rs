//! Format-independent result of assembling: what `assembler` produces and `writers` consume.
//!
//! Conventions (both sides MUST follow these):
//! * `sections[0]` is `.text`, `sections[1]` is `.data`. Further sections are not used.
//! * All branches/RIP-relative references whose target is in the *same section* and defined in
//!   this module are fully resolved by the assembler – no relocation is emitted for them.
//! * Every other reference produces a [`Reloc`]. Its value is computed the ELF way:
//!   `S + A - P` for PC-relative kinds (`P` = address of the 4-byte field itself) and `S + A`
//!   for absolute kinds. The bytes at the field are left as zero; writers that need
//!   implicit addends (COFF, Mach-O) translate `addend` themselves.
//! * Assembler-local labels (names starting with `.L`, numeric local labels like `1:`) are
//!   `Symbol { binding: Local, temporary: true }`. Writers must not emit temporary symbols into
//!   object symbol tables (gas doesn't); relocations against them are rewritten to
//!   section-symbol + offset.
//! * Platform name mangling (the leading `_` on macOS) is already done in the assembly text by
//!   the frontend; writers emit names verbatim.

pub type SymbolId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Text,
    Data,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub kind: SectionKind,
    pub data: Vec<u8>,
    pub relocs: Vec<Reloc>,
    /// Required alignment in bytes (power of two).
    pub align: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocKind {
    /// 32-bit PC-relative data reference (`[rip + sym]`): R_X86_64_PC32 / IMAGE_REL_AMD64_REL32 /
    /// X86_64_RELOC_SIGNED.
    Pc32,
    /// 32-bit PC-relative branch/call target: R_X86_64_PLT32 / IMAGE_REL_AMD64_REL32 /
    /// X86_64_RELOC_BRANCH.
    Branch32,
    /// 32-bit absolute (used by STABS values): R_X86_64_32 / IMAGE_REL_AMD64_ADDR32.
    Abs32,
    /// 64-bit absolute (`.quad sym`): R_X86_64_64 / IMAGE_REL_AMD64_ADDR64 / X86_64_RELOC_UNSIGNED.
    Abs64,
}

#[derive(Debug, Clone)]
pub struct Reloc {
    /// Offset of the relocated field inside its section.
    pub offset: u64,
    pub kind: RelocKind,
    pub symbol: SymbolId,
    pub addend: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    Local,
    Global,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub binding: Binding,
    /// `Some((section index, offset))` if defined here, `None` for undefined/external symbols
    /// (`.extern`, or referenced but never defined – gas treats those as undefined globals).
    pub definition: Option<(usize, u64)>,
    /// Assembler-temporary label (`.L*`, numeric local labels) – never emitted to symbol tables.
    pub temporary: bool,
}

/// Value of a STABS entry.
#[derive(Debug, Clone)]
pub enum StabValue {
    Const(u32),
    /// Address of a symbol (needs an Abs32 relocation in `.stab` for relocatable output).
    Symbol(SymbolId),
}

/// One `.stabs`/`.stabn` directive, in source order. `string` is `None` for `.stabn`.
#[derive(Debug, Clone)]
pub struct Stab {
    pub string: Option<String>,
    pub n_type: u8,
    pub n_other: u8,
    pub n_desc: u16,
    pub value: StabValue,
}

#[derive(Debug, Clone, Default)]
pub struct ObjModule {
    pub sections: Vec<Section>,
    pub symbols: Vec<Symbol>,
    /// STABS debug entries (only present with `-g`). Writers emit `.stab`/`.stabstr` the way
    /// gas does (including the leading header entry) for ELF; other formats ignore them.
    pub stabs: Vec<Stab>,
}

impl ObjModule {
    pub fn symbol_by_name(&self, name: &str) -> Option<SymbolId> {
        self.symbols.iter().position(|s| s.name == name)
    }
}
