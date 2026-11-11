//! Direct code generation: the translator drives an [`Emitter`] with typed instructions, which
//! are encoded with fadec into an [`ObjModule`]. No assembly text is produced or parsed on this
//! path – the `-S` text output is generated separately by the translator.
//!
//! OWNED BY: the encoder agent (implementation). The *public API* below is the contract with the
//! frontend; extend it additively if needed, but do not change existing signatures without
//! coordinating.
//!
//! Semantics must match what GNU as would have produced from the equivalent `-S` text (the
//! reference compiler pipes that text into gas): same instruction forms (aim for byte-identical
//! encodings incl. short/near jump relaxation), same acceptance/rejection of operand
//! combinations (e.g. ambiguous memory operand size, size mismatches, immediates out of range)
//! – errors are returned as `Err(String)` with a gas-like message.
// Until the translator drives the emitter, most of the API is unused in the binary.
#![allow(dead_code)]

mod encode;
mod layout;
mod regs;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod diff_fadec;
mod x86;
#[cfg(test)]
mod x86_tests;

use std::collections::HashMap;

use super::module::{Binding, DebugInfo, ObjModule, Symbol};
use crate::target::Target;

/// General purpose / SSE registers, as the user can name them in MemeAssembly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg {
    /// 64-bit GPR number 0..=15 (rax, rcx, rdx, rbx, rsp, rbp, rsi, rdi, r8..r15).
    R64(u8),
    /// 32-bit (eax..edi, r8d..r15d).
    R32(u8),
    /// 16-bit (ax..di, r8w..r15w).
    R16(u8),
    /// Low 8-bit: 0..=3 al,cl,dl,bl; 4..=7 spl,bpl,sil,dil (need REX); 8..=15 r8b..r15b.
    R8(u8),
    /// Legacy high-byte registers 0..=3: ah, ch, dh, bh.
    R8H(u8),
    Xmm(u8),
}

impl Reg {
    /// Parse a register name exactly as gas would accept it in the generated text
    /// (`rax`, `r8d`, `spl`, `ah`, ...). Implemented by the encoder agent.
    pub fn parse(name: &str) -> Option<Reg> {
        regs::parse(name)
    }
}

/// Explicit operand size (`BYTE PTR` .. `QWORD PTR`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Byte,
    Word,
    Dword,
    Qword,
    /// `XMMWORD PTR` – only for completeness; the translator never writes it.
    Xmmword,
}

/// Base of a memory operand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Base {
    /// `[reg + disp]` (also `[reg]`). A 32-bit base register means an addr32 (0x67) access,
    /// like gas does for `[eax]`.
    Reg(Reg),
    /// `[rip + symbol + disp]` – symbol may be in .data (→ Pc32 reloc), in .text (resolved
    /// locally where the target's assembler does so, else a Pc32 reloc; see `layout::finish`),
    /// or undefined (→ Pc32 reloc against an undefined global).
    Rip(String),
    /// `[disp]` – absolute address, e.g. `mov rax, [69]` (SIB-encoded disp32, no reloc).
    Abs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mem {
    pub size: Option<Size>,
    pub base: Base,
    pub disp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Reg(Reg),
    /// Immediate exactly as gas would see the number (`0x...` from `%llX`, chars `'a'`/`'\n'`
    /// already converted to their value by the frontend).
    Imm(i64),
    Mem(Mem),
}

/// Non-branch instructions used by the translation patterns, the optimisation-level filler and
/// the runtime helpers (writechar/readchar/killParent/martyrdom/main stub).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Mov, Movups, Lea, Push, Pop,
    Add, Sub, And, Xor, Cmp, Test, Not, Inc, Dec,
    Shl, Shr, Imul, Idiv, Cqo,
    Ret, Nop, Hlt, Int3, Syscall, Rdrand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    E, Ne, Z, Nz, G, L, Ge, Le, A, B,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchTarget {
    /// A named label/function (`.LUpgradeMarker_0`, `writechar`, `GetStdHandle`, user function).
    /// Undefined names become undefined globals with a Branch32 reloc (like gas). Branches to
    /// defined symbols are resolved or keep a Branch32 reloc per object format (ELF: calls to
    /// global symbols; Mach-O: anything outside the current atom); see `layout::finish`.
    Symbol(String),
    /// A quoted symbol name (gas `call "$"`): taken verbatim, without char-constant
    /// substitution or register/`$` interpretation. Otherwise like `Symbol`.
    Quoted(String),
    /// Numeric local label reference `Nf` (forward = true) / `Nb`.
    Local { n: u32, forward: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Text,
    Data,
}

pub struct Emitter {
    target: Target,
    /// Current section (0 = .text, 1 = .data).
    cur: usize,
    sections: [Vec<layout::Item>; 2],
    /// Largest `.align` seen per section.
    aligns: [u64; 2],
    syms: Vec<Sym>,
    /// Label definitions per section in definition order: `(item index, symbol)`. Mach-O atoms
    /// (see `layout::finish`) depend on the order of labels at the same position.
    defs: [Vec<(usize, usize)>; 2],
    by_name: HashMap<String, usize>,
    /// Number of definitions so far of each numeric local label.
    fb_count: HashMap<u32, u32>,
    /// Counter for anonymous location-counter symbols (`$`, `.`).
    dot_count: u32,
    debug: Option<Dbg>,
}

struct Sym {
    name: String,
    global: bool,
    /// `(section, item index)`: the label sits at the start of that item.
    def: Option<(usize, usize)>,
    /// An assembler-local label that never reaches the symbol table: `.L…` for ELF, `L…` for
    /// Mach-O (where `.L…` labels are ordinary local symbols), both for COFF, plus numeric local
    /// labels and `$`/`.`.
    temporary: bool,
}

#[derive(Default)]
struct Dbg {
    info: DebugInfo,
    /// The last `debug_line` not yet attached to an instruction (gas: a pending `.loc`).
    pending: Option<(usize, u32)>,
    /// `(item index in .text, file, line)`, in `.loc` order (several rows may share an address).
    rows: Vec<(usize, usize, u32)>,
    /// `(name, file, line, begin item, end item)`
    funcs: Vec<(String, usize, u32, usize, Option<usize>)>,
}

/// Characters gas accepts in symbol names (besides the first-char restriction on digits).
fn symbol_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '$'
}

fn valid_symbol(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(symbol_char)
        && !name.starts_with(|c: char| c.is_ascii_digit())
}

impl Emitter {
    pub fn new(target: Target) -> Emitter {
        Emitter {
            target,
            cur: 0,
            sections: [Vec::new(), Vec::new()],
            aligns: [1, 1],
            syms: Vec::new(),
            defs: [Vec::new(), Vec::new()],
            by_name: HashMap::new(),
            fb_count: HashMap::new(),
            dot_count: 0,
            debug: None,
        }
    }

    /// `.text` / `.data`
    pub fn section(&mut self, s: SectionKind) {
        self.cur = match s {
            SectionKind::Text => 0,
            SectionKind::Data => 1,
        };
    }
    /// `.global name`
    pub fn global(&mut self, name: &str) {
        let id = self.sym(name);
        self.syms[id].global = true;
        self.syms[id].temporary = false;
    }
    /// `.extern name` (Windows API imports). gas ignores `.extern`: the symbol only appears in
    /// the output if it is referenced.
    pub fn extern_symbol(&mut self, _name: &str) {}

    /// `name:` – errors on redefinition like gas ("symbol `x' is already defined").
    ///
    /// `name` is taken as gas would read it from the `-S` text: character constants are
    /// converted to numbers first (`.L'a'Wins_0` is the label `.L97Wins_0`). gas tolerates a
    /// second definition at exactly the same position.
    pub fn label(&mut self, name: &str) -> Result<(), String> {
        let name = &char_constants(name);
        if !valid_symbol(name) {
            return Err(layout::bad_label_message(name));
        }
        if matches!(name.as_str(), ".text" | ".data" | ".bss") && self.target != Target::Windows {
            return Err(format!("symbol `{name}' is already defined"));
        }
        let id = self.sym(name);
        let here = (self.cur, self.sections[self.cur].len());
        match self.syms[id].def {
            Some(d) if d == here => Ok(()),
            Some(_) => Err(format!("symbol `{name}' is already defined")),
            None => {
                self.define(id);
                Ok(())
            }
        }
    }
    /// Numeric local label `n:` (may be redefined, refs bind to nearest per gas rules).
    pub fn local_label(&mut self, n: u32) {
        let inst = self.fb_count.get(&n).copied().unwrap_or(0) + 1;
        self.fb_count.insert(n, inst);
        let id = self.sym(&fb_name(n, inst));
        self.define(id);
    }
    /// A non-branch instruction with gas Intel-syntax operand order (destination first).
    pub fn insn(&mut self, op: Op, ops: &[Operand]) -> Result<(), String> {
        let e = encode::encode(op, ops)?;
        let mut fixups = Vec::new();
        if let Some((field, sym, disp)) = e.rip {
            let id = self.sym_ref(&sym)?;
            fixups.push(layout::Fixup { offset: field, symbol: id, end: e.bytes.len(), disp });
        }
        self.push_insn(layout::Item::Bytes { data: e.bytes, fixups });
        Ok(())
    }
    /// `jmp target` (cond = None) or `jcc target`. Relaxed short/near like gas.
    pub fn jump(&mut self, cond: Option<Cond>, target: BranchTarget) -> Result<(), String> {
        let kind = match cond {
            None => layout::BranchKind::Jmp,
            Some(c) => layout::BranchKind::Jcc(c),
        };
        self.branch(kind, target)
    }
    /// `call target`
    pub fn call(&mut self, target: BranchTarget) -> Result<(), String> {
        self.branch(layout::BranchKind::Call, target)
    }
    /// `.ascii` / `.byte`
    pub fn bytes(&mut self, data: &[u8]) {
        self.push(layout::Item::Bytes { data: data.to_vec(), fixups: Vec::new() });
    }
    /// `.quad value`
    pub fn quad(&mut self, value: i64) {
        self.bytes(&value.to_le_bytes());
    }
    /// `.align n` (additive extension; used for `-O-s`), with the meaning the directive has for
    /// the target's assembler: gas aligns to `n` bytes, Apple's `as` (Mach-O) to `2**n` bytes and
    /// rejects `n >= 32` ("invalid alignment value"). In `.text` the padding is filled with the
    /// same NOP sequences that assembler uses, in `.data` with zeros. The section's alignment
    /// becomes at least that. COFF (Windows) can't represent alignments above 8192: like gas,
    /// `finish` then fails ("nonrepresentable section on output").
    pub fn align(&mut self, n: u64) -> Result<(), String> {
        let n = if self.target == Target::MacOS {
            if n >= 32 {
                return Err("invalid alignment value".into());
            }
            1 << n
        } else {
            n
        };
        if !n.is_power_of_two() {
            return Err("alignment not a power of 2".into());
        }
        self.aligns[self.cur] = self.aligns[self.cur].max(n);
        self.push(layout::Item::Align { align: n });
        Ok(())
    }

    // ---- -g debug info (DWARF, replaces STABS) ----
    /// Register a source file (absolute path, raw bytes as in the file system); returns its
    /// index for `debug_line`.
    pub fn debug_file(&mut self, path: &[u8]) -> usize {
        let d = self.debug.get_or_insert_with(Dbg::default);
        if let Some(i) = d.info.files.iter().position(|f| f == path) {
            return i;
        }
        d.info.files.push(path.to_vec());
        d.info.files.len() - 1
    }
    /// Like gas's `.loc file line`: the next instruction starts a line-table row for `line` of
    /// `file`. Consecutive calls without an instruction in between each keep their row (gas
    /// flushes a pending `.loc` at the current address when it sees the next one). Line 0 is
    /// accepted, but like gas ("incomplete location information") it produces no row: the code
    /// after it stays in the previous row.
    pub fn debug_line(&mut self, file: usize, line: u32) {
        let idx = self.sections[0].len();
        let d = self.debug.get_or_insert_with(Dbg::default);
        if let Some((f, l)) = d.pending.replace((file, line)) {
            if l != 0 {
                d.rows.push((idx, f, l));
            }
        }
    }
    /// A function starts at the current `.text` position.
    pub fn debug_function_begin(&mut self, name: &str, file: usize, line: u32) {
        let pos = self.sections[0].len();
        let d = self.debug.get_or_insert_with(Dbg::default);
        d.funcs.push((name.to_string(), file, line, pos, None));
    }
    /// The current function ends at the current `.text` position.
    pub fn debug_function_end(&mut self) {
        let pos = self.sections[0].len();
        let d = self.debug.get_or_insert_with(Dbg::default);
        if let Some(f) = d.funcs.iter_mut().rev().find(|f| f.4.is_none()) {
            f.4 = Some(pos);
        }
    }
    /// `comp_dir` is the raw bytes of the working directory.
    pub fn set_debug_meta(&mut self, comp_dir: &[u8], producer: &str) {
        let d = self.debug.get_or_insert_with(Dbg::default);
        d.info.comp_dir = comp_dir.to_vec();
        d.info.producer = producer.to_string();
    }

    /// Resolve labels, relax branches, produce the module. Errors: undefined numeric local
    /// labels, encoding failures, etc.
    pub fn finish(self) -> Result<ObjModule, String> {
        layout::finish(self)
    }

    // ---- internals ----

    /// Look up or create the symbol `name`.
    fn sym(&mut self, name: &str) -> usize {
        if let Some(&id) = self.by_name.get(name) {
            return id;
        }
        let id = self.syms.len();
        let temporary = match self.target {
            Target::Linux => name.starts_with(".L"),
            Target::MacOS => name.starts_with('L'),
            Target::Windows => name.starts_with(".L") || name.starts_with('L'),
        } || name.contains('\u{2}');
        self.syms.push(Sym { name: name.to_string(), global: false, def: None, temporary });
        self.by_name.insert(name.to_string(), id);
        id
    }

    /// Define symbol `id` at the current position.
    fn define(&mut self, id: usize) {
        let here = (self.cur, self.sections[self.cur].len());
        self.syms[id].def = Some(here);
        self.defs[here.0].push((here.1, id));
    }

    /// A reference from an instruction operand (`[rip + name]`).
    fn sym_ref(&mut self, name: &str) -> Result<usize, String> {
        if name == "$" || name == "." {
            return Ok(self.dot());
        }
        if !valid_symbol(name) {
            return Err(layout::bad_reference_message(name));
        }
        Ok(self.sym(name))
    }

    /// Anonymous symbol at the current location (gas `$` / `.`).
    fn dot(&mut self) -> usize {
        self.dot_count += 1;
        let id = self.sym(&format!("\u{2}dot{}", self.dot_count));
        self.define(id);
        id
    }

    fn push(&mut self, item: layout::Item) {
        self.sections[self.cur].push(item);
    }

    /// Push an instruction; it gets the pending line-table row (gas emits `.loc` rows only for
    /// instructions, not for data directives).
    fn push_insn(&mut self, item: layout::Item) {
        if self.cur == 0 {
            if let Some((file, line)) = self.debug.as_mut().and_then(|d| d.pending.take()) {
                if line != 0 {
                    let idx = self.sections[0].len();
                    self.debug.as_mut().unwrap().rows.push((idx, file, line));
                }
            }
        }
        self.push(item);
    }

    fn branch(&mut self, kind: layout::BranchKind, target: BranchTarget) -> Result<(), String> {
        let target = match target {
            BranchTarget::Local { n, forward } => {
                let count = self.fb_count.get(&n).copied().unwrap_or(0);
                if forward {
                    self.sym(&fb_name(n, count + 1))
                } else if count == 0 {
                    return Err(format!("backward ref to unknown label \"{n}:\""));
                } else {
                    self.sym(&fb_name(n, count))
                }
            }
            BranchTarget::Quoted(s) => {
                if s.is_empty() {
                    return Err("empty symbol name".into());
                }
                self.sym(&s)
            }
            BranchTarget::Symbol(s) => {
                let s = char_constants(&s);
                if s == "$" || s == "." {
                    self.dot()
                } else {
                    match regs::classify(&s) {
                        regs::Ident::Symbol => self.sym_ref(&s)?,
                        other => {
                            let bytes = layout::indirect_branch(kind, other, &s)?;
                            self.push_insn(layout::Item::Bytes { data: bytes, fixups: Vec::new() });
                            return Ok(());
                        }
                    }
                }
            }
        };
        self.push_insn(layout::Item::Branch { kind, target });
        Ok(())
    }
}

/// Value of the escape `\\c` in a gas character constant.
pub(super) fn escape_value(c: char) -> u32 {
    match c {
        'b' => 8,
        'f' => 12,
        'n' => 10,
        'r' => 13,
        't' => 9,
        '\'' => 39,
        '"' => 34,
        c => c as u32,
    }
}

/// Replace gas character constants (`'a'`, `'\\n'`, `' '`) by their decimal value, as gas does
/// when reading the line.
pub(super) fn char_constants(s: &str) -> String {
    if !s.contains('\'') {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '\'' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i >= chars.len() {
            out.push('0');
            break;
        }
        let mut v = chars[i] as u32;
        if chars[i] == '\\' && i + 1 < chars.len() {
            i += 1;
            v = escape_value(chars[i]);
        }
        i += 1;
        if i < chars.len() && chars[i] == '\'' {
            i += 1;
        }
        out.push_str(&v.to_string());
    }
    out
}

/// Internal (never emitted) name of instance `inst` of numeric local label `n`.
fn fb_name(n: u32, inst: u32) -> String {
    format!("{n}\u{2}{inst}")
}

impl Sym {
    fn to_symbol(&self) -> Symbol {
        let undefined = self.def.is_none();
        Symbol {
            name: self.name.clone(),
            binding: if self.global || undefined { Binding::Global } else { Binding::Local },
            definition: None,
            // gas turns undefined `.L` references into ordinary undefined globals.
            temporary: self.temporary && !self.global && !undefined,
        }
    }
}
