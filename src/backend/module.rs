//! Format-independent result of code generation: what `emitter::Emitter::finish` produces and
//! `writers` consume.
//!
//! Conventions (both sides MUST follow these):
//! * `sections[0]` is `.text`, `sections[1]` is `.data`. Further sections are not used.
//! * All branches/RIP-relative references whose target is in the *same section* and defined in
//!   this module are fully resolved by the emitter – no relocation is emitted for them.
//! * Every other reference produces a [`Reloc`]. Its value is computed the ELF way:
//!   `S + A - P` for PC-relative kinds (`P` = address of the 4-byte field itself) and `S + A`
//!   for absolute kinds. The bytes at the field are left as zero; writers that need
//!   implicit addends (COFF, Mach-O) translate `addend` themselves.
//! * Assembler-local labels (names starting with `.L`, numeric local labels like `1:`) are
//!   `Symbol { binding: Local, temporary: true }`. Writers must not emit temporary symbols into
//!   object symbol tables (gas doesn't); relocations against them are rewritten to
//!   section-symbol + offset.
//! * Platform name mangling (the leading `_` on macOS) is already done in the assembly text by
//!   the frontend (it passes `_main` etc. to the emitter); writers emit names verbatim.

/// Index into [`ObjModule::symbols`].
pub type SymbolId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Text,
    Data,
}

/// The contents of one section and the relocations against them.
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
    /// 32-bit absolute: R_X86_64_32 / IMAGE_REL_AMD64_ADDR32.
    Abs32,
    /// 64-bit absolute (`.quad sym`): R_X86_64_64 / IMAGE_REL_AMD64_ADDR64 / X86_64_RELOC_UNSIGNED.
    Abs64,
}

/// A field the writer or linker must fill in; see the module docs for how its value is computed.
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

/// A symbol table entry.
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

/// One row of the line table: the instruction at `.text` offset `offset` came from `line` of
/// `DebugInfo::files[file]`. Rows are sorted by offset.
#[derive(Debug, Clone)]
pub struct LineRow {
    pub offset: u64,
    pub file: usize,
    pub line: u32,
}

/// A MemeAssembly function as a DWARF `DW_TAG_subprogram`: `.text` range `[low, high)`.
#[derive(Debug, Clone)]
pub struct DebugFunction {
    pub name: String,
    pub file: usize,
    pub line: u32,
    pub low: u64,
    pub high: u64,
}

/// Source-level debug information for `-g`. Writers turn it into DWARF sections (`.debug_info`,
/// `.debug_abbrev`, `.debug_line`, strings inline) with the relocations the object format
/// requires; executables get them resolved.
#[derive(Debug, Clone, Default)]
pub struct DebugInfo {
    /// Source files as written in the debug info (absolute paths; raw bytes, since paths need
    /// not be UTF-8 – they are written verbatim, like gas does).
    pub files: Vec<Vec<u8>>,
    /// DW_AT_comp_dir (current working directory at compile time, raw bytes).
    pub comp_dir: Vec<u8>,
    /// DW_AT_producer, e.g. "MemeAssembly compiler v1.6".
    pub producer: String,
    pub lines: Vec<LineRow>,
    pub functions: Vec<DebugFunction>,
}

/// The assembled program: sections, symbol table and optional debug information.
#[derive(Debug, Clone, Default)]
pub struct ObjModule {
    pub sections: Vec<Section>,
    pub symbols: Vec<Symbol>,
    /// `-g` debug information; `None` without `-g`.
    pub debug: Option<DebugInfo>,
}

impl ObjModule {
    /// The symbol named `name`, if any.
    pub fn symbol_by_name(&self, name: &str) -> Option<SymbolId> {
        self.symbols.iter().position(|s| s.name == name)
    }
}
