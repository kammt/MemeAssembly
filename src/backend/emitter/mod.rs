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
#![allow(dead_code)]

use super::module::ObjModule;
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
        let _ = name;
        todo!("Reg::parse")
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
    /// locally), or undefined (→ Pc32 reloc against an undefined global).
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
    /// Undefined names become undefined globals with a Branch32 reloc (like gas).
    Symbol(String),
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
}

impl Emitter {
    pub fn new(target: Target) -> Emitter {
        Emitter { target }
    }

    /// `.text` / `.data`
    pub fn section(&mut self, _s: SectionKind) {
        todo!()
    }
    /// `.global name`
    pub fn global(&mut self, _name: &str) {
        todo!()
    }
    /// `.extern name` (Windows API imports)
    pub fn extern_symbol(&mut self, _name: &str) {
        todo!()
    }
    /// `name:` – errors on redefinition like gas ("symbol `x' is already defined").
    pub fn label(&mut self, _name: &str) -> Result<(), String> {
        todo!()
    }
    /// Numeric local label `n:` (may be redefined, refs bind to nearest per gas rules).
    pub fn local_label(&mut self, _n: u32) {
        todo!()
    }
    /// A non-branch instruction with gas Intel-syntax operand order (destination first).
    pub fn insn(&mut self, _op: Op, _ops: &[Operand]) -> Result<(), String> {
        todo!()
    }
    /// `jmp target` (cond = None) or `jcc target`. Relaxed short/near like gas.
    pub fn jump(&mut self, _cond: Option<Cond>, _target: BranchTarget) -> Result<(), String> {
        todo!()
    }
    /// `call target`
    pub fn call(&mut self, _target: BranchTarget) -> Result<(), String> {
        todo!()
    }
    /// `.ascii` / `.byte`
    pub fn bytes(&mut self, _data: &[u8]) {
        todo!()
    }
    /// `.quad value`
    pub fn quad(&mut self, _value: i64) {
        todo!()
    }

    // ---- -g debug info (DWARF, replaces STABS) ----
    /// Register a source file (absolute path); returns its index for `debug_line`.
    pub fn debug_file(&mut self, _path: &str) -> usize {
        todo!()
    }
    /// Code emitted from now on (until the next call) belongs to `line` of `file`.
    pub fn debug_line(&mut self, _file: usize, _line: u32) {
        todo!()
    }
    /// A function starts at the current `.text` position.
    pub fn debug_function_begin(&mut self, _name: &str, _file: usize, _line: u32) {
        todo!()
    }
    /// The current function ends at the current `.text` position.
    pub fn debug_function_end(&mut self) {
        todo!()
    }
    pub fn set_debug_meta(&mut self, _comp_dir: &str, _producer: &str) {
        todo!()
    }

    /// Resolve labels, relax branches, produce the module. Errors: undefined numeric local
    /// labels, encoding failures, etc.
    pub fn finish(self) -> Result<ObjModule, String> {
        let _ = self.target;
        todo!()
    }
}
