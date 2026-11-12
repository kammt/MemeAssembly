//! Code generation, written once for both kinds of output. [`translate`] walks the analysed
//! program and makes one call on an [`Asm`] per instruction, label or directive. The [`Asm`] is
//! either the `-S` text printer ([`super::asm_text`], GNU as Intel syntax) or, for object files
//! and executables, the backend's [`Emitter`](crate::backend::emitter::Emitter), which encodes
//! the instructions directly into an [`ObjModule`](crate::backend::module::ObjModule)
//! ([`super::encoder`]). Both outputs therefore always contain the same code.

use super::analyser::main_function_exists;
use super::commands::*;
use super::state::*;
use super::log::print_debug;
use crate::backend::emitter::{Base, Cond, Mem, Op, Reg, SectionKind, Size};
use crate::target::Target;

/// Result of an [`Asm`] call; the error is a message for the user.
pub type AsmResult = Result<(), String>;

/// Generates the code of one command (the `code` of its [`COMMAND_LIST`] entry).
pub type CodeGen = fn(&mut dyn Asm, &Ctx) -> AsmResult;

/// An immediate operand. The variants only differ in how the `-S` text writes the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Imm<'a> {
    Dec(i64),
    /// `0x...` in upper-case hex digits; negative values as their 64-bit two's complement.
    Hex(i64),
    /// The value with its source spelling (`'a'`, `65`).
    Text(i64, &'a [u8]),
}

impl Imm<'_> {
    /// The numeric value, whatever the spelling.
    pub fn value(self) -> i64 {
        match self {
            Imm::Dec(v) | Imm::Hex(v) | Imm::Text(v, _) => v,
        }
    }
}

/// An operand of [`Asm::insn`]: like [`Operand`](crate::backend::emitter::Operand), but an
/// immediate keeps its `-S` spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opnd<'a> {
    Reg(Reg),
    Imm(Imm<'a>),
    Mem(Mem),
}

/// What the code generation needs from an assembler. Symbol names are bytes: the `-S` text
/// writes character parameters inside label names as they were written (see
/// [`Asm::char_in_symbol`]).
pub trait Asm {
    /// A non-branch instruction, destination first.
    fn insn(&mut self, op: Op, ops: &[Opnd]) -> AsmResult;
    /// `jmp`/`jcc` to a named label.
    fn jump(&mut self, cond: Option<Cond>, target: &[u8]) -> AsmResult;
    /// `jmp`/`jcc` to the numeric local label `n` (`nf` / `nb`).
    fn jump_local(&mut self, cond: Option<Cond>, n: u32, forward: bool) -> AsmResult;
    /// `call` to a named function.
    fn call(&mut self, target: &[u8]) -> AsmResult;
    /// Defines `name` at the current position.
    fn label(&mut self, name: &[u8]) -> AsmResult;
    /// Defines the numeric local label `n` (`n:`); it may be defined repeatedly.
    fn local_label(&mut self, n: u32);
    /// `.global name`
    fn global(&mut self, name: &[u8]);
    /// `.extern name` (the Windows API functions).
    fn extern_symbol(&mut self, name: &str);
    /// Switches to `.text` or `.data`.
    fn section(&mut self, kind: SectionKind);
    /// `.ascii`: the bytes verbatim.
    fn ascii(&mut self, data: &[u8]);
    /// `n` zero bytes (`.byte 0, ...`)
    fn zeros(&mut self, n: usize);
    /// `.quad` with one 8-byte value per immediate.
    fn quad(&mut self, values: &[Imm]);
    /// Align to `bytes` bytes.
    fn align(&mut self, bytes: u64) -> AsmResult;
    /// Registers an input file for the debug line information; returns its number for
    /// `debug_line`.
    fn debug_file(&mut self, path: &[u8]) -> usize;
    /// The following instructions come from `line` of `file` (`.loc`). Line 0 marks code that
    /// has no source line.
    fn debug_line(&mut self, file: usize, line: u32);
    /// A MemeAssembly function starts at the current position (a DWARF subprogram). Only the
    /// object output records functions; the `-S` text has no directive for them.
    fn debug_function_begin(&mut self, _name: &[u8], _file: usize, _line: u32) {}
    /// The function opened by the last `debug_function_begin` ends here.
    fn debug_function_end(&mut self) {}
    /// The spelling of a character parameter inside a symbol name: gas reads the character
    /// constant in `.L'a'Wins_0` as its value (`.L97Wins_0`).
    fn char_in_symbol(&self, value: u8, text: &[u8], verbatim: bool) -> Vec<u8>;

    // Layout of the `-S` text; no effect on the code.
    /// Indentation of the following lines.
    fn indent(&mut self, _indent: &'static str) {}
    /// An empty line.
    fn blank(&mut self) {}
    /// A line containing only the indentation.
    fn gap(&mut self) {}
    /// Ends the current line (a local label not followed by an instruction stands alone).
    fn end_line(&mut self) {}
}

/// The current working directory as raw bytes, or the placeholder `(null)` if it can't be
/// determined. Used for the debug information's file paths and compilation directory.
pub(super) fn getcwd_bytes() -> Vec<u8> {
    match std::env::current_dir() {
        #[cfg(unix)]
        Ok(p) => {
            use std::os::unix::ffi::OsStrExt;
            p.as_os_str().as_bytes().to_vec()
        }
        #[cfg(not(unix))]
        Ok(p) => p.to_string_lossy().into_owned().into_bytes(),
        Err(_) => b"(null)".to_vec(),
    }
}

/// The absolute path of an input file as written into the debug information: a relative name
/// is prefixed with the current working directory.
fn debug_file_path(input_file: &[u8]) -> Vec<u8> {
    let mut path = Vec::new();
    if input_file.first() != Some(&b'/') {
        path.extend_from_slice(&getcwd_bytes());
        path.push(b'/');
    }
    path.extend_from_slice(input_file);
    path
}

/// Operand sizes a bully-mode pointer can get, in bytes (index = `computed_index % 4`).
pub(super) const BULLY_POINTER_SIZES: [u32; 4] = [1, 2, 4, 8];

/// In bully mode, the "operand size unknown" checks of the analyser are skipped and the operand
/// size of a pointer is chosen here (in bytes), if it is needed.
fn bully_pointer_size(state: &CompileState, cmd: &ParsedCommand) -> Option<u32> {
    let index = cmd.pointer?;
    (state.bully() && cmd.bully_pointer_needs_size(index))
        .then(|| BULLY_POINTER_SIZES[(state.computed_index % 4) as usize])
}

/// `it's over 9000` compares with the immediate 9000, which an 8-bit operand can never exceed
/// (and which gas would silently truncate to 0x28): with such an operand the command just
/// halts.
fn over_9000_always_halts(state: &CompileState, cmd: &ParsedCommand) -> bool {
    cmd.opcode == Opcode::Over9000
        && match bully_pointer_size(state, cmd) {
            Some(size) => size == 1,
            None => cmd.pointer.is_none() && cmd.param_type(0) == PARAM_REG8,
        }
}

/// Bully mode: an immediate stored to a pointer whose size was chosen randomly is truncated to
/// that size (a QWORD immediate must be a sign-extended 32-bit value).
fn truncate_to_operand_size(value: i64, size: u32) -> i64 {
    match size {
        1 if !(-128..=255).contains(&value) => value & 0xFF,
        2 if !(-32768..=65535).contains(&value) => value & 0xFFFF,
        4 if !(i32::MIN as i64..=u32::MAX as i64).contains(&value) => value & 0xFFFF_FFFF,
        8 => value as i32 as i64,
        _ => value,
    }
}

/// Whether the command is translated (with `-O69420` only the function definitions are).
fn is_translated(state: &CompileState, cmd: &ParsedCommand) -> bool {
    cmd.translate
        && (state.optimisation_level != OPT_69420
            || cmd.opcode == Opcode::FuncDef)
}

/// Whether no command produces any code after bully mode's injected main (no functions at all,
/// everything deleted, or `-O69420`).
fn bully_main_is_empty(state: &CompileState) -> bool {
    let mut commands = state.files.iter().flat_map(|f| &f.functions).flat_map(|f| &f.commands);
    !commands.any(|cmd| is_translated(state, cmd))
}

// ---- operands ----

const RAX: Opnd = Opnd::Reg(Reg::R64(0));
const RCX: Opnd = Opnd::Reg(Reg::R64(1));
const RDX: Opnd = Opnd::Reg(Reg::R64(2));
const RBX: Opnd = Opnd::Reg(Reg::R64(3));
const RSP: Opnd = Opnd::Reg(Reg::R64(4));
const RBP: Opnd = Opnd::Reg(Reg::R64(5));
const RSI: Opnd = Opnd::Reg(Reg::R64(6));
const RDI: Opnd = Opnd::Reg(Reg::R64(7));
const R8: Opnd = Opnd::Reg(Reg::R64(8));
const R9: Opnd = Opnd::Reg(Reg::R64(9));
const R10: Opnd = Opnd::Reg(Reg::R64(10));
const R11: Opnd = Opnd::Reg(Reg::R64(11));
const R12: Opnd = Opnd::Reg(Reg::R64(12));
const R13: Opnd = Opnd::Reg(Reg::R64(13));
const EAX: Opnd = Opnd::Reg(Reg::R32(0));
const XMM0: Opnd = Opnd::Reg(Reg::Xmm(0));

fn dec(v: i64) -> Opnd<'static> {
    Opnd::Imm(Imm::Dec(v))
}

fn hex(v: i64) -> Opnd<'static> {
    Opnd::Imm(Imm::Hex(v))
}

/// `[rip + symbol]`
fn rip(size: Option<Size>, symbol: &str) -> Opnd<'static> {
    Opnd::Mem(Mem { size, base: Base::Rip(symbol.to_string()), disp: 0 })
}

/// `[rsp + disp]`
fn stack(size: Option<Size>, disp: i64) -> Opnd<'static> {
    Opnd::Mem(Mem { size, base: Base::Reg(Reg::R64(4)), disp })
}

/// `QWORD PTR [rip + .Ltmp64]`
fn tmp64() -> Opnd<'static> {
    rip(Some(Size::Qword), ".Ltmp64")
}

/// The command being translated, passed to its [`CodeGen`].
pub struct Ctx<'a> {
    state: &'a CompileState,
    cmd: &'a ParsedCommand,
    /// Index of the current file (the suffix of its labels).
    file: usize,
}

impl<'a> Ctx<'a> {
    /// Parameter `i` as an instruction operand.
    fn op(&self, i: usize) -> Result<Opnd<'a>, String> {
        let size = bully_pointer_size(self.state, self.cmd);
        Ok(match &self.cmd.parameters[i] {
            Some(Param::Reg(r)) if self.cmd.is_pointer(i) => {
                let size = size.map(|bytes| match bytes {
                    1 => Size::Byte,
                    2 => Size::Word,
                    4 => Size::Dword,
                    _ => Size::Qword,
                });
                Opnd::Mem(Mem { size, base: Base::Reg(*r), disp: 0 })
            }
            Some(Param::Reg(r)) => Opnd::Reg(*r),
            // Decimal numbers are written as hex (issue #73)
            Some(Param::Decimal { value, .. }) => {
                hex(size.map_or(*value, |size| truncate_to_operand_size(*value, size)))
            }
            Some(Param::Char { value, text, verbatim: true }) => Opnd::Imm(Imm::Text(*value as i64, text)),
            Some(Param::Char { value, .. }) => dec(*value as i64),
            _ => return Err(self.bad_parameter(i)),
        })
    }

    /// The symbol name `prefix`, parameter `i`, `suffix` (labels of `monke`, `who would win`
    /// and the function names). Character parameters are spelled as [`Asm::char_in_symbol`]
    /// says, numbers in hex.
    fn sym(&self, a: &dyn Asm, prefix: &str, i: usize, suffix: &str) -> Result<Vec<u8>, String> {
        let mut name = prefix.as_bytes().to_vec();
        match &self.cmd.parameters[i] {
            Some(Param::Char { value, text, verbatim }) => {
                name.extend_from_slice(&a.char_in_symbol(*value, text, *verbatim))
            }
            Some(Param::Decimal { value, .. }) => {
                name.extend_from_slice(format!("0x{:X}", *value as u64).as_bytes())
            }
            Some(p @ (Param::Reg(_) | Param::MonkeLabel(_) | Param::FuncName { .. })) => {
                name.extend_from_slice(&p.text(self.state.target))
            }
            _ => return Err(self.bad_parameter(i)),
        }
        name.extend_from_slice(suffix.as_bytes());
        Ok(name)
    }

    /// `name_{file}`
    fn file_label(&self, name: &str) -> Vec<u8> {
        format!("{name}_{}", self.file).into_bytes()
    }

    fn bad_parameter(&self, i: usize) -> String {
        format!("internal error: unexpected parameter {:?} for opcode {}", self.cmd.parameters[i], self.cmd.opcode as u8)
    }
}

// ---- the commands of COMMAND_LIST ----
//
// One `CodeGen` per command; its phrase and parameter types are in `COMMAND_LIST`. Jump
// markers that exist once per file (`upgrade`, `banana`, `they're the same picture`, `confused
// stonks`) get the file index as a suffix, so every input file has its own.

pub fn func_def(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    let name = c.sym(a, "", 0, "")?;
    a.label(&name)
}
pub fn ret(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.insn(Op::Ret, &[])
}
pub fn ret_one(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.insn(Op::Mov, &[RAX, dec(1)])?;
    a.insn(Op::Ret, &[])
}
pub fn ret_zero(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.insn(Op::Xor, &[RAX, RAX])?;
    a.insn(Op::Ret, &[])
}
pub fn call(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    let name = c.sym(a, "", 0, "")?;
    a.call(&name)
}
pub fn push(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Push, &[c.op(0)?])
}
pub fn pop(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Pop, &[c.op(0)?])
}
pub fn and(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::And, &[c.op(0)?, c.op(1)?])
}
pub fn not(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Not, &[c.op(0)?])
}
pub fn zero(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Xor, &[c.op(0)?, c.op(0)?])
}
pub fn mov(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Mov, &[c.op(0)?, c.op(1)?])
}
pub fn upvote(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Add, &[c.op(0)?, dec(1)])
}
pub fn downvote(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Sub, &[c.op(0)?, dec(1)])
}
/// `parry a you filthy casual b`: `b -= a`.
pub fn parry(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Sub, &[c.op(1)?, c.op(0)?])
}
pub fn add(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Add, &[c.op(0)?, c.op(1)?])
}
pub fn shl(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Shl, &[c.op(0)?, dec(1)])
}
pub fn shr(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Shr, &[c.op(0)?, dec(1)])
}
pub fn imul(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Imul, &[c.op(0)?, c.op(1)?])
}
/// `look at what a needs to mimic a fraction of b`: `b = b / a` (signed, the remainder is
/// discarded). The divisor is stored in `.Ltmp64` and the dividend pushed, both before rax and
/// rdx are saved, so either operand may be rax, rdx or rsp. The dividend is reloaded from the
/// stack into rax, `cqo` sign-extends it into rdx:rax for `idiv`, and the quotient is written
/// back to its stack slot and popped into `b`.
pub fn divide(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Mov, &[tmp64(), c.op(0)?])?;
    a.insn(Op::Push, &[c.op(1)?])?;
    a.insn(Op::Push, &[RDX])?;
    a.insn(Op::Push, &[RAX])?;
    a.insn(Op::Mov, &[RAX, stack(Some(Size::Qword), 16)])?;
    a.insn(Op::Cqo, &[])?;
    a.insn(Op::Idiv, &[tmp64()])?;
    a.insn(Op::Mov, &[stack(Some(Size::Qword), 16), RAX])?;
    a.insn(Op::Pop, &[RAX])?;
    a.insn(Op::Pop, &[RDX])?;
    a.insn(Op::Pop, &[c.op(1)?])?;
    a.gap();
    Ok(())
}
/// `a UNLIMITED POWER b`: `a = a ** b` by square-and-multiply, one iteration per exponent bit.
/// The exponent is kept in `.Ltmp64` and the base squared in place on the stack. A negative
/// exponent gives the truncated integer result: 1 for a base of 1, ±1 for -1 (by the exponent's
/// parity), 0 otherwise.
pub fn power(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    let base = || stack(Some(Size::Qword), 0);
    a.insn(Op::Mov, &[tmp64(), c.op(1)?])?;
    a.insn(Op::Push, &[c.op(0)?])?;
    a.insn(Op::Mov, &[c.op(0)?, dec(1)])?;
    a.insn(Op::Cmp, &[tmp64(), dec(0)])?;
    a.jump_local(Some(Cond::Ge), 2, true)?;
    a.insn(Op::And, &[tmp64(), dec(1)])?;
    a.insn(Op::Cmp, &[base(), dec(1)])?;
    a.jump_local(Some(Cond::E), 2, true)?;
    a.insn(Op::Cmp, &[base(), dec(-1)])?;
    a.jump_local(Some(Cond::E), 2, true)?;
    a.insn(Op::Xor, &[c.op(0)?, c.op(0)?])?;
    a.jump_local(None, 3, true)?;
    a.local_label(2);
    a.insn(Op::Test, &[tmp64(), dec(1)])?;
    a.jump_local(Some(Cond::Z), 1, true)?;
    a.insn(Op::Imul, &[c.op(0)?, base()])?;
    a.local_label(1);
    a.insn(Op::Shr, &[tmp64(), dec(1)])?;
    a.jump_local(Some(Cond::Z), 3, true)?;
    a.insn(Op::Push, &[RAX])?;
    a.insn(Op::Mov, &[RAX, stack(Some(Size::Qword), 8)])?;
    a.insn(Op::Imul, &[RAX, RAX])?;
    a.insn(Op::Mov, &[stack(Some(Size::Qword), 8), RAX])?;
    a.insn(Op::Pop, &[RAX])?;
    a.jump_local(None, 2, false)?;
    a.local_label(3);
    a.insn(Op::Add, &[RSP, dec(8)])?;
    a.gap();
    Ok(())
}
pub fn upgrade_label(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.label(&c.file_label(".LUpgradeMarker"))
}
pub fn upgrade_jump(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.jump(None, &c.file_label(".LUpgradeMarker"))
}
pub fn banana_label(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.label(&c.file_label(".LBananaMarker"))
}
pub fn banana_jump(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.jump(None, &c.file_label(".LBananaMarker"))
}
pub fn monke_label(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    let name = c.sym(a, ".L", 0, "")?;
    a.label(&name)
}
pub fn monke_jump(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    let name = c.sym(a, ".L", 0, "")?;
    a.jump(None, &name)
}
/// `who would win? a or b`: jumps to `a wins` if `a > b`, to `b wins` if `a < b` (signed),
/// else continues.
pub fn who_would_win(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    let suffix = format!("Wins_{}", c.file);
    let (first, second) = (c.sym(a, ".L", 0, &suffix)?, c.sym(a, ".L", 1, &suffix)?);
    a.insn(Op::Cmp, &[c.op(0)?, c.op(1)?])?;
    a.jump(Some(Cond::G), &first)?;
    a.jump(Some(Cond::L), &second)
}
pub fn wins_label(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    let name = c.sym(a, ".L", 0, &format!("Wins_{}", c.file))?;
    a.label(&name)
}
pub fn find_difference(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Cmp, &[c.op(0)?, c.op(1)?])?;
    a.jump(Some(Cond::E), &c.file_label(".LSamePicture"))
}
pub fn same_picture_label(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.label(&c.file_label(".LSamePicture"))
}
/// Calls `function` with a 16-byte aligned stack (assuming `rsp` is a multiple of 8): pushes and
/// pops in the program leave the alignment at a command unknown, and on Windows the helpers call
/// the Windows API, which requires it. The Linux/macOS helpers only make a syscall and would not
/// need it, but the same sequence is emitted for every target.
fn aligned_call(a: &mut dyn Asm, function: &str) -> AsmResult {
    a.insn(Op::Test, &[RSP, hex(0xF)])?;
    a.jump_local(Some(Cond::Z), 1, true)?;
    a.insn(Op::Sub, &[RSP, dec(8)])?;
    a.call(function.as_bytes())?;
    a.insn(Op::Add, &[RSP, dec(8)])?;
    a.jump_local(None, 2, true)?;
    a.local_label(1);
    a.call(function.as_bytes())?;
    a.local_label(2);
    a.end_line();
    Ok(())
}
/// Stores the character in `.LCharacter` and calls the `writechar` helper.
pub fn write_char(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Mov, &[rip(Some(Size::Byte), ".LCharacter"), c.op(0)?])?;
    aligned_call(a, "writechar")?;
    a.gap();
    Ok(())
}
/// Calls the `readchar` helper, which reads one byte into `.LCharacter`.
pub fn read_char(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    aligned_call(a, "readchar")?;
    a.insn(Op::Mov, &[c.op(0)?, rip(Some(Size::Byte), ".LCharacter")])?;
    a.gap();
    Ok(())
}
/// `guess I'll die`: crashes by loading from the unmapped absolute address 69.
pub fn die(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.insn(Op::Mov, &[RAX, Opnd::Mem(Mem { size: None, base: Base::Abs, disp: 69 })])
}
/// Jumps to `.LConfusedStonks_<file>`, which [`translate`] places before a random command of
/// the file.
pub fn confused_stonks(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.jump(None, &c.file_label(".LConfusedStonks"))
}
/// No code: instead, the analyser halves the number of commands per use, deleting random ones.
pub fn perfectly_balanced(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.gap();
    Ok(())
}
/// `wait, that's illegal`: zeroes the callee-saved registers rbx, rbp, r12 and r13.
pub fn illegal(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.insn(Op::Xor, &[RBX, RBX])?;
    a.insn(Op::Xor, &[RBP, RBP])?;
    a.insn(Op::Xor, &[R12, R12])?;
    a.insn(Op::Xor, &[R13, R13])
}
pub fn oh_no(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.insn(Op::Nop, &[])
}
/// `it's over 9000 a`: halts (`hlt`, which faults in user mode) unless `a > 9000`.
pub fn over_9000(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    if over_9000_always_halts(c.state, c.cmd) {
        return a.insn(Op::Hlt, &[]);
    }
    a.insn(Op::Cmp, &[c.op(0)?, dec(9000)])?;
    a.jump_local(Some(Cond::G), 1, true)?;
    a.insn(Op::Hlt, &[])?;
    a.local_label(1);
    Ok(())
}
/// Copies rsp to rbp, then pops rsp.
pub fn refuses_to_elaborate(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.insn(Op::Mov, &[RBP, RSP])?;
    a.insn(Op::Pop, &[RSP])
}
/// An endless loop.
pub fn shall_not_pass(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.local_label(1);
    a.insn(Op::Xor, &[RAX, RAX])?;
    a.jump_local(None, 1, false)
}
/// Zeroes rsp, so the next stack access crashes.
pub fn houston(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.insn(Op::Xor, &[RSP, RSP])
}
pub fn rdrand(a: &mut dyn Asm, c: &Ctx) -> AsmResult {
    a.insn(Op::Rdrand, &[c.op(0)?])
}
pub fn air_support(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.insn(Op::Syscall, &[])
}
pub fn trap(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.insn(Op::Int3, &[])
}
/// `or draw 25` at the end of a line: adds 25 to eax instead of the line's command.
pub fn draw_25(a: &mut dyn Asm, _: &Ctx) -> AsmResult {
    a.insn(Op::Add, &[EAX, dec(25)])
}
/// The placeholder entry for invalid commands; never translated because invalid commands are
/// compile errors.
pub fn invalid(_: &mut dyn Asm, c: &Ctx) -> AsmResult {
    Err(format!("internal error: invalid command in line {}", c.cmd.line_num))
}

// ---- runtime helpers ----

/// Installs `killParent` as the SIGINT handler, at the start of main (martyrdom, on unless
/// `--no-martyrdom`).
fn martyrdom_code(a: &mut dyn Asm, target: Target) -> AsmResult {
    a.indent("");
    a.insn(Op::Push, &[RAX])?;
    a.indent("    ");
    for r in [RDI, RSI, RDX, R10, RCX, R11] {
        a.insn(Op::Push, &[r])?;
    }
    a.gap();
    a.insn(Op::Lea, &[RAX, rip(None, "killParent")])?;
    a.insn(Op::Mov, &[rip(None, ".Lsa_handler"), RAX])?;
    if target == Target::MacOS {
        // Signaling SIGINT on MacOS segfaults unless the second qword of the sigaction struct
        // contains the address as well
        a.insn(Op::Mov, &[rip(None, ".Lsa_handler_2"), RAX])?;
    }
    a.blank();
    a.insn(Op::Mov, &[RAX, if target == Target::Linux { dec(13) } else { hex(0x200002E) }])?;
    a.insn(Op::Mov, &[RDI, dec(2)])?;
    a.insn(Op::Lea, &[RSI, rip(None, ".LsigStruct")])?;
    a.insn(Op::Xor, &[RDX, RDX])?;
    a.insn(Op::Mov, &[R10, dec(8)])?;
    a.insn(Op::Syscall, &[])?;
    a.gap();
    for r in [R11, RCX, R10, RDX, RSI, RDI, RAX] {
        a.insn(Op::Pop, &[r])?;
    }
    a.blank();
    Ok(())
}

/// `killParent`, the SIGINT handler of martyrdom: sends SIGKILL to the parent process
/// (`getppid` + `kill`), then exits with status 0. The syscall numbers are Linux's or macOS's
/// (BSD class `0x2000000`).
fn kill_parent(a: &mut dyn Asm, target: Target) -> AsmResult {
    let linux = target == Target::Linux;
    a.indent("");
    a.label(b"killParent")?;
    a.indent("    ");
    a.insn(Op::Mov, &[RAX, if linux { dec(110) } else { hex(0x2000027) }])?;
    a.insn(Op::Syscall, &[])?;
    a.blank();
    a.insn(Op::Mov, &[RDI, RAX])?;
    a.insn(Op::Mov, &[RSI, dec(9)])?;
    a.insn(Op::Mov, &[RAX, if linux { dec(62) } else { hex(0x2000025) }])?;
    a.insn(Op::Syscall, &[])?;
    a.blank();
    a.insn(Op::Mov, &[RDI, dec(0)])?;
    // exit(0)
    a.insn(Op::Mov, &[RAX, if linux { dec(60) } else { hex(0x2000001) }])?;
    a.insn(Op::Syscall, &[])?;
    a.insn(Op::Ret, &[])?;
    a.blank();
    Ok(())
}

/// `writechar` / `readchar` via the Windows API (`GetStdHandle`, then `WriteFile`/`ReadFile` of
/// one byte at `.LCharacter`, the byte count going to `.Ltmp64`). The volatile general-purpose
/// registers are preserved. The 48-byte frame holds the 32 bytes of shadow space plus the 5th
/// argument (`lpOverlapped` = NULL at `[rsp + 32]`) above them, and keeps the stack 16-byte
/// aligned for the calls: the helper is entered with rsp ≡ 8 (mod 16) and pushes 7 registers.
fn windows_io_function(a: &mut dyn Asm, name: &str, std_handle: i64, api: &str) -> AsmResult {
    a.blank();
    a.blank();
    a.indent("");
    a.label(name.as_bytes())?;
    a.indent("\t");
    for r in [RCX, RAX, RDX, R8, R9, R10, R11] {
        a.insn(Op::Push, &[r])?;
    }
    a.insn(Op::Sub, &[RSP, dec(48)])?;
    a.insn(Op::Mov, &[RCX, dec(std_handle)])?;
    a.call(b"GetStdHandle")?;
    a.insn(Op::Mov, &[RCX, RAX])?;
    a.insn(Op::Lea, &[RDX, rip(None, ".LCharacter")])?;
    a.insn(Op::Mov, &[R8, dec(1)])?;
    a.insn(Op::Lea, &[R9, rip(None, ".Ltmp64")])?;
    a.insn(Op::Mov, &[stack(Some(Size::Qword), 32), dec(0)])?;
    a.call(api.as_bytes())?;
    a.insn(Op::Add, &[RSP, dec(48)])?;
    for r in [R11, R10, R9, R8, RDX, RAX, RCX] {
        a.insn(Op::Pop, &[r])?;
    }
    a.insn(Op::Ret, &[])
}

/// `writechar` / `readchar` via the `write`/`read` syscall of one byte at `.LCharacter` (Linux,
/// macOS). The registers the syscall uses or clobbers are preserved.
fn unix_io_function(a: &mut dyn Asm, name: &str, fd: i64, syscall: Opnd) -> AsmResult {
    let read = fd == 0;
    a.blank();
    a.blank();
    a.indent("");
    a.label(name.as_bytes())?;
    a.indent("\t");
    for r in [RCX, R11, RAX, RDI, RSI, RDX] {
        a.insn(Op::Push, &[r])?;
    }
    if read {
        a.blank();
    }
    a.insn(Op::Mov, &[RDX, dec(1)])?;
    a.insn(Op::Lea, &[RSI, rip(None, ".LCharacter")])?;
    a.insn(Op::Mov, &[RDI, dec(fd)])?;
    a.insn(Op::Mov, &[RAX, syscall])?;
    a.insn(Op::Syscall, &[])?;
    if read {
        a.blank();
    }
    for r in [RDX, RSI, RDI, RAX, R11, RCX] {
        a.insn(Op::Pop, &[r])?;
    }
    if !read {
        a.gap();
    }
    a.insn(Op::Ret, &[])
}

/// Translates one command: its line-table row, its code and the filler instructions of the
/// optimisation level (`-O69420` translates only function definitions, as `return 0`).
fn translate_command(
    a: &mut dyn Asm,
    state: &CompileState,
    cmd: &ParsedCommand,
    file: usize,
    debug_file: Option<usize>,
) -> AsmResult {
    let command = cmd.opcode.command();
    let is_def = cmd.opcode == Opcode::FuncDef;
    if !is_def && state.optimisation_level == OPT_69420 {
        print_debug(state.log_level, format!("line {}: not translated, -O69420 only translates function definitions", cmd.line_num));
        return Ok(());
    }

    // Function definitions get a row too, so the martyrdom prologue that follows `main:` and
    // `break <function>` are attributed to the definition line.
    if let Some(f) = debug_file {
        a.debug_line(f, cmd.line_num as u32);
    }

    a.indent(if is_def { "" } else { "\t" });
    (command.code)(a, &Ctx { state, cmd, file })?;
    a.end_line();

    a.indent("\t");
    match state.optimisation_level {
        OPT_1 => a.insn(Op::Nop, &[])?,
        OPT_2 => {
            a.insn(Op::Push, &[RAX])?;
            a.insn(Op::Pop, &[RAX])?;
        }
        OPT_3 => {
            a.insn(Op::Movups, &[stack(None, 8), XMM0])?;
            a.insn(Op::Movups, &[XMM0, stack(None, 8)])?;
        }
        // If we get here, this was a function declaration
        OPT_69420 => {
            a.insn(Op::Xor, &[RAX, RAX])?;
            a.insn(Op::Ret, &[])?;
        }
        _ => {}
    }
    Ok(())
}

/// Generates the whole program: the global function symbols, `.data` (the I/O byte, a scratch
/// qword and the martyrdom `sigaction` struct), `killParent`, bully mode's injected main, the
/// commands of every file and the `writechar`/`readchar` helpers. Windows gets no `sigaction`
/// struct or `killParent`: martyrdom is not implemented there.
pub fn translate(state: &CompileState, a: &mut dyn Asm) -> AsmResult {
    let target = state.target;

    // Define all functions as global
    for function in state.files.iter().flat_map(|f| &f.functions) {
        let def = &function.commands[0];
        if def.translate {
            a.global(&def.param_text(0, target));
        }
    }

    if target == Target::Windows {
        // To interact with the Windows API, we need to reference the needed functions
        a.blank();
        for name in ["GetStdHandle", "WriteFile", "ReadFile"] {
            a.extern_symbol(name);
        }
    }

    a.blank();
    a.section(SectionKind::Data);
    a.indent("\t");
    a.label(b".LCharacter")?;
    a.ascii(b"a");
    a.label(b".Ltmp64")?;
    a.zeros(8);
    // Struct for martyrdom command
    if target != Target::Windows {
        a.label(b".LsigStruct")?;
        a.indent("\t\t");
        a.label(b".Lsa_handler")?;
        a.quad(&[Imm::Dec(0)]);
        if target == Target::Linux {
            a.quad(&[Imm::Text(0x04000000, b"0x04000000")]);
        } else {
            a.label(b".Lsa_handler_2")?;
            a.quad(&[Imm::Dec(0)]);
        }
        a.quad(&[Imm::Dec(0), Imm::Dec(0)]);
        a.blank();
    }

    a.blank();
    a.blank();
    a.section(SectionKind::Text);
    a.indent("\t");
    a.gap();
    a.blank();
    a.indent("");
    a.label(b".Ltext0")?;

    if target != Target::Windows {
        kill_parent(a, target)?;
    }

    // In bully mode, a missing main function is "fixed" by making the code at the start main
    // (only for executables, so never part of the -S text).
    if state.bully()
        && state.output_mode == OutputMode::Executable
        && !main_function_exists(state)
    {
        // The entry symbol is the target's (`_main` on macOS). The martyrdom prologue is left
        // out on Windows, which has no killParent/.LsigStruct.
        let main_name = state.main_function_name();
        a.global(main_name);
        a.label(main_name)?;
        if target != Target::Windows {
            martyrdom_code(a, target)?;
        }
        // Without any code to fall into, main would run into writechar (or off the end of the
        // text section): return 0 instead.
        if bully_main_is_empty(state) {
            a.insn(Op::Xor, &[RAX, RAX])?;
            a.insn(Op::Ret, &[])?;
        }
    }

    let main_name = state.main_function_name();
    let mut last_debug_file = None;
    for (i, file) in state.files.iter().enumerate() {
        let debug_file = state.use_debug_info.then(|| a.debug_file(&debug_file_path(&file.file_name)));
        last_debug_file = debug_file.or(last_debug_file);

        // The confused stonks label is only needed if the file jumps to it
        let uses_confused_stonks = file
            .functions
            .iter()
            .flat_map(|f| &f.commands)
            .any(|cmd| cmd.opcode == Opcode::ConfusedStonks && is_translated(state, cmd));

        let mut line = 0usize;
        for function in &file.functions {
            let commands = &function.commands;
            let function_name = commands[0].param_text(0, target);
            if let Some(f) = debug_file {
                a.debug_function_begin(&function_name, f, commands[0].line_num as u32);
            }

            for (k, current_command) in commands.iter().enumerate() {
                if target != Target::Windows && state.martyrdom && k == 1 && *function_name == *main_name {
                    martyrdom_code(a, target)?;
                }

                // Place the confused stonks label now if it should be at this position
                if uses_confused_stonks && line == file.random_index {
                    a.indent("\t");
                    a.label(format!(".LConfusedStonks_{i}").as_bytes())?;
                }

                if current_command.translate {
                    translate_command(a, state, current_command, i, debug_file)?;
                }
                line += 1;
            }

            if debug_file.is_some() {
                a.debug_function_end();
            }
        }
    }

    // With -O69420 all commands are optimised out, so these are not needed
    if state.optimisation_level != OPT_69420 {
        // The helpers are not source code: line 0 instead of the last command's line
        if let Some(f) = last_debug_file {
            a.debug_line(f, 0);
        }
        if target == Target::Windows {
            windows_io_function(a, "writechar", -11, "WriteFile")?;
            windows_io_function(a, "readchar", -10, "ReadFile")?;
        } else {
            let linux = target == Target::Linux;
            unix_io_function(a, "writechar", 1, if linux { dec(1) } else { hex(0x2000004) })?;
            unix_io_function(a, "readchar", 0, if linux { dec(0) } else { hex(0x2000003) })?;
        }
    }

    if state.optimisation_level == OPT_S {
        // 512 MiB (on Windows, COFF can't represent the section alignment: an error like gas's)
        a.align(1 << 29)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn immediate_truncation() {
        assert_eq!(truncate_to_operand_size(255, 1), 255);
        assert_eq!(truncate_to_operand_size(-128, 1), -128);
        assert_eq!(truncate_to_operand_size(300, 1), 44);
        assert_eq!(truncate_to_operand_size(65536, 2), 0);
        assert_eq!(truncate_to_operand_size(-1, 4), -1);
        assert_eq!(truncate_to_operand_size(4294967296, 4), 0);
        assert_eq!(truncate_to_operand_size(2147483648, 8), i32::MIN as i64);
        assert_eq!(truncate_to_operand_size(-5, 8), -5);
    }

    fn command(opcode: Opcode, params: [Option<Param>; 2], pointer: Option<usize>) -> ParsedCommand {
        let mut c = ParsedCommand::new(1);
        c.opcode = opcode;
        c.parameters = params;
        c.pointer = pointer;
        c
    }

    fn decimal(value: i64) -> Option<Param> {
        Some(Param::Decimal { value, text: value.to_string().into_bytes() })
    }

    #[test]
    fn operands() {
        let mut state = CompileState::new(Target::Linux);
        let c = command(Opcode::Mov, [Some(Param::Reg(Reg::R64(0))), decimal(-5)], None);
        let ctx = Ctx { state: &state, cmd: &c, file: 0 };
        assert_eq!(ctx.op(0), Ok(RAX));
        assert_eq!(ctx.op(1), Ok(hex(-5)));

        let c = command(Opcode::Mov, [Some(Param::Reg(Reg::R64(3))), Some(Param::Reg(Reg::R32(0)))], Some(0));
        let ctx = Ctx { state: &state, cmd: &c, file: 0 };
        assert_eq!(ctx.op(0), Ok(Opnd::Mem(Mem { size: None, base: Base::Reg(Reg::R64(3)), disp: 0 })));

        // Bully mode guesses the operand size from computed_index (69 % 4 == 1 → WORD PTR) and
        // truncates the immediate to it
        state.compile_mode = CompileMode::Bully;
        let c = command(Opcode::Mov, [Some(Param::Reg(Reg::R64(3))), decimal(65537)], Some(0));
        let ctx = Ctx { state: &state, cmd: &c, file: 0 };
        assert_eq!(ctx.op(0), Ok(Opnd::Mem(Mem { size: Some(Size::Word), base: Base::Reg(Reg::R64(3)), disp: 0 })));
        assert_eq!(ctx.op(1), Ok(hex(1)));
    }
}
