//! Direct code generation for object files and executables.
//!
//! The C compiler piped the text of `translator.c` into gcc. Here the same program is issued
//! instruction by instruction to [`Emitter`]: every command of [`COMMAND_LIST`] has an `encode`
//! function next to its `-S` translation pattern, and [`encode_module`] mirrors
//! [`super::translator::write_to_string`] block by block. Both paths must stay semantically
//! identical: `gas(-S text) == encode_module()`.
//!
//! Parameters are converted the way gas reads the generated text: registers by name, decimals
//! from their `0x..` text, characters (`'a'`, `'\n'`, ASCII codes like `010`) with gas' escape and
//! number rules, pointers to memory operands and labels with gas' character-constant scrubbing.

use super::analyser::main_function_exists;
use super::commands::*;
use super::libc::strtoll;
use super::log::{print_debug, VERSION_STRING};
use super::translator::{debug_file_path, param_text};
use crate::backend::emitter::{
    Base, BranchTarget, Cond, Emitter, Mem, Op, Operand, Reg, SectionKind, Size,
};
use crate::backend::module::ObjModule;
use crate::target::Target;

pub type EncResult = Result<(), String>;

/// The `encode` function of a command.
pub type Encode = fn(&mut Enc, &CmdArgs) -> EncResult;

// Fixed registers used by the translation patterns.
const RAX: Operand = Operand::Reg(Reg::R64(0));
const RCX: Operand = Operand::Reg(Reg::R64(1));
const RDX: Operand = Operand::Reg(Reg::R64(2));
const RBX: Operand = Operand::Reg(Reg::R64(3));
const RSP: Operand = Operand::Reg(Reg::R64(4));
const RBP: Operand = Operand::Reg(Reg::R64(5));
const RSI: Operand = Operand::Reg(Reg::R64(6));
const RDI: Operand = Operand::Reg(Reg::R64(7));
const R8: Operand = Operand::Reg(Reg::R64(8));
const R9: Operand = Operand::Reg(Reg::R64(9));
const R10: Operand = Operand::Reg(Reg::R64(10));
const R11: Operand = Operand::Reg(Reg::R64(11));
const R12: Operand = Operand::Reg(Reg::R64(12));
const R13: Operand = Operand::Reg(Reg::R64(13));
const EAX: Operand = Operand::Reg(Reg::R32(0));
const XMM0: Operand = Operand::Reg(Reg::Xmm(0));

fn imm(v: i64) -> Operand {
    Operand::Imm(v)
}

/// `[rip + symbol]`
fn rip(size: Option<Size>, symbol: &str) -> Operand {
    Operand::Mem(Mem { size, base: Base::Rip(symbol.to_string()), disp: 0 })
}

/// `[reg + disp]`
fn mem_reg(size: Option<Size>, reg: Reg, disp: i64) -> Operand {
    Operand::Mem(Mem { size, base: Base::Reg(reg), disp })
}

const RSP_REG: Reg = Reg::R64(4);

/// Emitter plus the per-command context (`{F}` = number of the current file).
pub struct Enc<'a> {
    pub e: &'a mut Emitter,
    pub file_num: usize,
}

impl Enc<'_> {
    fn i(&mut self, op: Op, ops: &[Operand]) -> EncResult {
        self.e.insn(op, ops)
    }
    fn jmp(&mut self, cond: Option<Cond>, symbol: String) -> EncResult {
        self.e.jump(cond, BranchTarget::Symbol(symbol))
    }
    fn jmp_local(&mut self, cond: Option<Cond>, n: u32, forward: bool) -> EncResult {
        self.e.jump(cond, BranchTarget::Local { n, forward })
    }
    fn call(&mut self, symbol: &str) -> EncResult {
        self.e.call(BranchTarget::Symbol(symbol.to_string()))
    }
    fn label(&mut self, name: &str) -> EncResult {
        self.e.label(name)
    }
    fn f(&self) -> usize {
        self.file_num
    }
}

/// The parameters of one command, as the translator substitutes them.
pub struct CmdArgs<'a> {
    state: &'a CompileState,
    cmd: &'a ParsedCommand,
}

impl CmdArgs<'_> {
    /// The text that `-S` writes for `{i}`.
    fn text(&self, i: usize) -> Vec<u8> {
        param_text(self.state, self.cmd, i)
    }

    /// `{i}` as an instruction operand.
    fn op(&self, i: usize) -> Result<Operand, String> {
        let cmd = self.cmd;
        let parameter = cmd.param(i);
        if cmd.is_pointer as usize == i + 1 {
            let size = if self.bully_operand_size(i) {
                const SIZES: [Size; 4] = [Size::Byte, Size::Word, Size::Dword, Size::Qword];
                Some(SIZES[(self.state.computed_index % 4) as usize])
            } else {
                None
            };
            return Ok(mem_reg(size, register(parameter)?, 0));
        }
        match cmd.param_types[i] {
            PARAM_DECIMAL => Ok(imm(strtoll(parameter).value)),
            PARAM_CHAR => char_value(parameter).map(imm),
            _ => register(parameter).map(Operand::Reg),
        }
    }

    /// Same condition as in `translator::param_text`.
    fn bully_operand_size(&self, i: usize) -> bool {
        let command = &COMMAND_LIST[self.cmd.opcode as usize];
        self.state.compile_mode == CompileMode::Bully
            && command.used_parameters == 2
            && self.cmd.bully_pointer_needs_size(i)
    }

    /// A symbol name containing `{i}` as gas would read it.
    fn sym(&self, prefix: &str, i: usize, suffix: &str) -> Result<String, String> {
        let mut text = prefix.as_bytes().to_vec();
        text.extend_from_slice(&self.text(i));
        text.extend_from_slice(suffix.as_bytes());
        gas_symbol(&text)
    }
}

/// A register named in the source, or the error gas would give for the text.
fn register(name: &[u8]) -> Result<Reg, String> {
    std::str::from_utf8(name)
        .ok()
        .and_then(Reg::parse)
        .ok_or_else(|| format!("bad register name `{}'", String::from_utf8_lossy(name)))
}

/// gas' `process_escape` (app.c): only these escapes are known, any other `\x` is `x`
/// (so `'\v'` is 'v' and `'\?'` is '?').
fn gas_escape(c: u8) -> u8 {
    match c {
        b'b' => 8,
        b'f' => 12,
        b'n' => b'\n',
        b'r' => b'\r',
        b't' => b'\t',
        other => other,
    }
}

/// Value of a character constant `'x'` / `'\x'` at the start of `text`, plus its length
/// (the closing quote is optional, like in gas).
fn char_constant(text: &[u8]) -> Option<(u8, usize)> {
    if text.first() != Some(&b'\'') {
        return None;
    }
    let (value, mut len) = match text.get(1)? {
        b'\\' => (gas_escape(*text.get(2)?), 3),
        &c => (c, 2),
    };
    if text.get(len) == Some(&b'\'') {
        len += 1;
    }
    Some((value, len))
}

/// A character parameter as gas evaluates its text: `'a'`, an escape like `'\n'`, or an ASCII
/// code (validated with `strtol(.., 10)`, but read by gas with C prefixes: `010` is octal).
pub fn char_value(text: &[u8]) -> Result<i64, String> {
    if let Some((value, len)) = char_constant(text) {
        if len == text.len() {
            return Ok(value as i64);
        }
        return Err(junk(&text[len..]));
    }
    gas_integer(text)
}

fn junk(rest: &[u8]) -> String {
    format!("junk `{}' after expression", String::from_utf8_lossy(rest))
}

/// gas' integer constants: optional sign, `0x` hex, `0b` binary, leading `0` octal, decimal.
pub fn gas_integer(text: &[u8]) -> Result<i64, String> {
    let (negative, digits) = match text.first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let (radix, start) = match digits {
        [b'0', b'x' | b'X', ..] => (16, 2),
        [b'0', b'b' | b'B', ..] => (2, 2),
        [b'0', ..] => (8, 1),
        _ => (10, 0),
    };
    let body = &digits[start..];
    let len = body.iter().take_while(|c| (**c as char).is_digit(radix)).count();
    if len == 0 && start != 1 {
        return Err(format!("bad expression: `{}'", String::from_utf8_lossy(text)));
    }
    if len < body.len() {
        return Err(junk(&body[len..]));
    }
    let mut value: i64 = 0;
    for &c in body {
        let d = (c as char).to_digit(radix).unwrap_or(0) as i64;
        value = value.wrapping_mul(radix as i64).wrapping_add(d);
    }
    Ok(if negative { value.wrapping_neg() } else { value })
}

/// A label as gas reads it from the generated text: character constants inside it are
/// replaced with their decimal value (gas' input scrubber does that, `.L'a'Wins_0` defines
/// `.L97Wins_0`), and anything that can't be part of a symbol is junk.
pub fn gas_symbol(text: &[u8]) -> Result<String, String> {
    let mut name = String::new();
    let mut i = 0;
    while i < text.len() {
        if let Some((value, len)) = char_constant(&text[i..]) {
            name.push_str(&value.to_string());
            i += len;
            continue;
        }
        let c = text[i];
        if !(c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || c == b'$') {
            return Err(junk(&text[i..]));
        }
        name.push(c as char);
        i += 1;
    }
    Ok(name)
}

// ---- encode functions of the command table (same order as COMMAND_LIST) ----

pub fn func_def(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.label(&a.sym("", 0, "")?)
}
pub fn ret(e: &mut Enc, _: &CmdArgs) -> EncResult {
    e.i(Op::Ret, &[])
}
pub fn ret_one(e: &mut Enc, _: &CmdArgs) -> EncResult {
    e.i(Op::Mov, &[RAX, imm(1)])?;
    e.i(Op::Ret, &[])
}
pub fn ret_zero(e: &mut Enc, _: &CmdArgs) -> EncResult {
    e.i(Op::Xor, &[RAX, RAX])?;
    e.i(Op::Ret, &[])
}
pub fn call(e: &mut Enc, a: &CmdArgs) -> EncResult {
    let name = a.sym("", 0, "")?;
    e.call(&name)
}
pub fn push(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Push, &[a.op(0)?])
}
pub fn pop(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Pop, &[a.op(0)?])
}
pub fn and(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::And, &[a.op(0)?, a.op(1)?])
}
pub fn not(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Not, &[a.op(0)?])
}
pub fn zero(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Xor, &[a.op(0)?, a.op(0)?])
}
pub fn mov(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Mov, &[a.op(0)?, a.op(1)?])
}
pub fn upvote(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Add, &[a.op(0)?, imm(1)])
}
pub fn downvote(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Sub, &[a.op(0)?, imm(1)])
}
pub fn parry(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Sub, &[a.op(1)?, a.op(0)?])
}
pub fn add(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Add, &[a.op(0)?, a.op(1)?])
}
pub fn shl(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Shl, &[a.op(0)?, imm(1)])
}
pub fn shr(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Shr, &[a.op(0)?, imm(1)])
}
pub fn imul(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Imul, &[a.op(0)?, a.op(1)?])
}
pub fn divide(e: &mut Enc, a: &CmdArgs) -> EncResult {
    let tmp = || rip(Some(Size::Qword), ".Ltmp64");
    e.i(Op::Mov, &[tmp(), a.op(0)?])?;
    e.i(Op::Push, &[RDX])?;
    e.i(Op::Cqo, &[])?;
    e.i(Op::Push, &[RAX])?;
    e.i(Op::Mov, &[RAX, a.op(1)?])?;
    e.i(Op::Idiv, &[tmp()])?;
    e.i(Op::Push, &[RAX])?;
    e.i(Op::Mov, &[RAX, mem_reg(None, RSP_REG, 8)])?;
    e.i(Op::Pop, &[a.op(1)?])?;
    e.i(Op::Add, &[RSP, imm(8)])?;
    e.i(Op::Pop, &[RDX])
}
pub fn power(e: &mut Enc, a: &CmdArgs) -> EncResult {
    let tmp = || rip(Some(Size::Qword), ".Ltmp64");
    e.i(Op::Mov, &[tmp(), a.op(1)?])?;
    e.i(Op::Cmp, &[tmp(), imm(0)])?;
    e.jmp_local(Some(Cond::Ne), 2, true)?;
    e.i(Op::Xor, &[a.op(0)?, a.op(0)?])?;
    e.i(Op::Inc, &[a.op(0)?])?;
    e.jmp_local(None, 4, true)?;
    e.e.local_label(2);
    e.i(Op::Push, &[a.op(0)?])?;
    e.i(Op::Dec, &[tmp()])?;
    e.e.local_label(3);
    e.i(Op::Imul, &[a.op(0)?, mem_reg(None, RSP_REG, 0)])?;
    e.i(Op::Dec, &[tmp()])?;
    e.jmp_local(Some(Cond::Nz), 3, false)?;
    e.i(Op::Add, &[RSP, imm(8)])?;
    e.e.local_label(4);
    Ok(())
}
pub fn upgrade_label(e: &mut Enc, _: &CmdArgs) -> EncResult {
    let name = format!(".LUpgradeMarker_{}", e.f());
    e.label(&name)
}
pub fn upgrade_jump(e: &mut Enc, _: &CmdArgs) -> EncResult {
    let name = format!(".LUpgradeMarker_{}", e.f());
    e.jmp(None, name)
}
pub fn banana_label(e: &mut Enc, _: &CmdArgs) -> EncResult {
    let name = format!(".LBananaMarker_{}", e.f());
    e.label(&name)
}
pub fn banana_jump(e: &mut Enc, _: &CmdArgs) -> EncResult {
    let name = format!(".LBananaMarker_{}", e.f());
    e.jmp(None, name)
}
pub fn monke_label(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.label(&a.sym(".L", 0, "")?)
}
pub fn monke_jump(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.jmp(None, a.sym(".L", 0, "")?)
}
pub fn who_would_win(e: &mut Enc, a: &CmdArgs) -> EncResult {
    let suffix = format!("Wins_{}", e.f());
    e.i(Op::Cmp, &[a.op(0)?, a.op(1)?])?;
    e.jmp(Some(Cond::G), a.sym(".L", 0, &suffix)?)?;
    e.jmp(Some(Cond::L), a.sym(".L", 1, &suffix)?)
}
pub fn wins_label(e: &mut Enc, a: &CmdArgs) -> EncResult {
    let name = a.sym(".L", 0, &format!("Wins_{}", e.f()))?;
    e.label(&name)
}
pub fn find_difference(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Cmp, &[a.op(0)?, a.op(1)?])?;
    let name = format!(".LSamePicture_{}", e.f());
    e.jmp(Some(Cond::E), name)
}
pub fn same_picture_label(e: &mut Enc, _: &CmdArgs) -> EncResult {
    let name = format!(".LSamePicture_{}", e.f());
    e.label(&name)
}
/// Calls `function` with a 16-byte aligned stack (the `test rsp, 0xF` dance).
fn aligned_call(e: &mut Enc, function: &str) -> EncResult {
    e.i(Op::Test, &[RSP, imm(0xF)])?;
    e.jmp_local(Some(Cond::Z), 1, true)?;
    e.i(Op::Sub, &[RSP, imm(8)])?;
    e.call(function)?;
    e.i(Op::Add, &[RSP, imm(8)])?;
    e.jmp_local(None, 2, true)?;
    e.e.local_label(1);
    e.call(function)?;
    e.e.local_label(2);
    Ok(())
}
pub fn write_char(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Mov, &[rip(Some(Size::Byte), ".LCharacter"), a.op(0)?])?;
    aligned_call(e, "writechar")
}
pub fn read_char(e: &mut Enc, a: &CmdArgs) -> EncResult {
    aligned_call(e, "readchar")?;
    e.i(Op::Mov, &[a.op(0)?, rip(Some(Size::Byte), ".LCharacter")])
}
pub fn die(e: &mut Enc, _: &CmdArgs) -> EncResult {
    e.i(Op::Mov, &[RAX, Operand::Mem(Mem { size: None, base: Base::Abs, disp: 69 })])
}
pub fn confused_stonks(e: &mut Enc, _: &CmdArgs) -> EncResult {
    let name = format!(".LConfusedStonks_{}", e.f());
    e.jmp(None, name)
}
pub fn perfectly_balanced(_: &mut Enc, _: &CmdArgs) -> EncResult {
    Ok(())
}
pub fn illegal(e: &mut Enc, _: &CmdArgs) -> EncResult {
    e.i(Op::Xor, &[RBX, RBX])?;
    e.i(Op::Xor, &[RBP, RBP])?;
    e.i(Op::Xor, &[R12, R12])?;
    // The C translation pattern said `xor r13 r13` (missing comma), which gas rejected.
    e.i(Op::Xor, &[R13, R13])
}
pub fn oh_no(e: &mut Enc, _: &CmdArgs) -> EncResult {
    e.i(Op::Nop, &[])
}
pub fn over_9000(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Cmp, &[a.op(0)?, imm(9000)])?;
    e.jmp_local(Some(Cond::G), 1, true)?;
    e.i(Op::Hlt, &[])?;
    e.e.local_label(1);
    Ok(())
}
pub fn refuses_to_elaborate(e: &mut Enc, _: &CmdArgs) -> EncResult {
    e.i(Op::Mov, &[RBP, RSP])?;
    e.i(Op::Pop, &[RSP])
}
pub fn shall_not_pass(e: &mut Enc, _: &CmdArgs) -> EncResult {
    e.e.local_label(1);
    e.i(Op::Xor, &[RAX, RAX])?;
    e.jmp_local(None, 1, false)
}
pub fn houston(e: &mut Enc, _: &CmdArgs) -> EncResult {
    e.i(Op::Xor, &[RSP, RSP])
}
pub fn rdrand(e: &mut Enc, a: &CmdArgs) -> EncResult {
    e.i(Op::Rdrand, &[a.op(0)?])
}
pub fn air_support(e: &mut Enc, _: &CmdArgs) -> EncResult {
    e.i(Op::Syscall, &[])
}
pub fn trap(e: &mut Enc, _: &CmdArgs) -> EncResult {
    e.i(Op::Int3, &[])
}
pub fn draw_25(e: &mut Enc, _: &CmdArgs) -> EncResult {
    e.i(Op::Add, &[EAX, imm(25)])
}
/// The placeholder entry for invalid commands (translation pattern "0"); never translated
/// because invalid commands are compile errors.
pub fn invalid(_: &mut Enc, _: &CmdArgs) -> EncResult {
    Err("no such instruction: `0'".to_string())
}

// ---- runtime helpers (same blocks as translator::write_to_string) ----

fn martyrdom_code(e: &mut Emitter, target: Target) -> EncResult {
    for r in [RAX, RDI, RSI, RDX, R10, RCX, R11] {
        e.insn(Op::Push, &[r])?;
    }
    e.insn(Op::Lea, &[RAX, rip(None, "killParent")])?;
    e.insn(Op::Mov, &[rip(None, ".Lsa_handler"), RAX])?;
    if target == Target::MacOS {
        e.insn(Op::Mov, &[rip(None, ".Lsa_handler_2"), RAX])?;
    }
    let sigaction = if target == Target::Linux { 13 } else { 0x200002E };
    e.insn(Op::Mov, &[RAX, imm(sigaction)])?;
    e.insn(Op::Mov, &[RDI, imm(2)])?;
    e.insn(Op::Lea, &[RSI, rip(None, ".LsigStruct")])?;
    e.insn(Op::Xor, &[RDX, RDX])?;
    e.insn(Op::Mov, &[R10, imm(8)])?;
    e.insn(Op::Syscall, &[])?;
    for r in [R11, RCX, R10, RDX, RSI, RDI, RAX] {
        e.insn(Op::Pop, &[r])?;
    }
    Ok(())
}

fn kill_parent(e: &mut Emitter, target: Target) -> EncResult {
    let linux = target == Target::Linux;
    e.label("killParent")?;
    e.insn(Op::Mov, &[RAX, imm(if linux { 110 } else { 0x2000027 })])?;
    e.insn(Op::Syscall, &[])?;
    e.insn(Op::Mov, &[RDI, RAX])?;
    e.insn(Op::Mov, &[RSI, imm(9)])?;
    e.insn(Op::Mov, &[RAX, imm(if linux { 62 } else { 0x2000025 })])?;
    e.insn(Op::Syscall, &[])?;
    e.insn(Op::Mov, &[RDI, imm(0)])?;
    e.insn(Op::Mov, &[RAX, imm(if linux { 60 } else { 0x2000001 })])?;
    e.insn(Op::Syscall, &[])?;
    e.insn(Op::Ret, &[])
}

/// `writechar` / `readchar` via the Windows API.
fn windows_io_function(e: &mut Emitter, name: &str, std_handle: i64, api: &str) -> EncResult {
    e.label(name)?;
    for r in [RCX, RAX, RDX, R8, R9] {
        e.insn(Op::Push, &[r])?;
    }
    e.insn(Op::Sub, &[RSP, imm(32)])?;
    e.insn(Op::Mov, &[RCX, imm(std_handle)])?;
    e.call(BranchTarget::Symbol("GetStdHandle".to_string()))?;
    e.insn(Op::Mov, &[RCX, RAX])?;
    e.insn(Op::Lea, &[RDX, rip(None, ".LCharacter")])?;
    e.insn(Op::Mov, &[R8, imm(1)])?;
    e.insn(Op::Lea, &[R9, rip(None, ".Ltmp64")])?;
    e.insn(Op::Mov, &[mem_reg(Some(Size::Qword), RSP_REG, 32), imm(0)])?;
    e.call(BranchTarget::Symbol(api.to_string()))?;
    e.insn(Op::Add, &[RSP, imm(32)])?;
    for r in [R9, R8, RDX, RAX, RCX] {
        e.insn(Op::Pop, &[r])?;
    }
    e.insn(Op::Ret, &[])
}

/// `writechar` / `readchar` via syscalls (Linux, macOS).
fn unix_io_function(e: &mut Emitter, name: &str, fd: i64, syscall: i64) -> EncResult {
    e.label(name)?;
    for r in [RCX, R11, RAX, RDI, RSI, RDX] {
        e.insn(Op::Push, &[r])?;
    }
    e.insn(Op::Mov, &[RDX, imm(1)])?;
    e.insn(Op::Lea, &[RSI, rip(None, ".LCharacter")])?;
    e.insn(Op::Mov, &[RDI, imm(fd)])?;
    e.insn(Op::Mov, &[RAX, imm(syscall)])?;
    e.insn(Op::Syscall, &[])?;
    for r in [RDX, RSI, RDI, RAX, R11, RCX] {
        e.insn(Op::Pop, &[r])?;
    }
    e.insn(Op::Ret, &[])
}

/// Translates one command (C: `translateToAssembly`).
fn encode_command(
    state: &CompileState,
    enc: &mut Enc,
    cmd: &ParsedCommand,
    debug_file: Option<usize>,
) -> EncResult {
    let command = &COMMAND_LIST[cmd.opcode as usize];
    let is_def = command.command_type == COMMAND_TYPE_FUNC_DEF;
    if !is_def && state.optimisation_level == OPT_69420 {
        print_debug(state.log_level, "\tCommand is not a function declaration, abort.", &[]);
        return Ok(());
    }
    if let (Some(file), false) = (debug_file, is_def) {
        enc.e.debug_line(file, cmd.line_num as u32);
    }

    (command.encode)(enc, &CmdArgs { state, cmd })?;

    let e = &mut *enc.e;
    match state.optimisation_level {
        OPT_1 => e.insn(Op::Nop, &[])?,
        OPT_2 => {
            e.insn(Op::Push, &[RAX])?;
            e.insn(Op::Pop, &[RAX])?;
        }
        OPT_3 => {
            e.insn(Op::Movups, &[mem_reg(None, RSP_REG, 8), XMM0])?;
            e.insn(Op::Movups, &[XMM0, mem_reg(None, RSP_REG, 8)])?;
        }
        // If we get here, this was a function declaration
        OPT_69420 => {
            e.insn(Op::Xor, &[RAX, RAX])?;
            e.insn(Op::Ret, &[])?;
        }
        _ => {}
    }
    Ok(())
}

/// Generates the program directly into an object module (the `-O` / executable path).
pub fn encode_module(state: &CompileState) -> Result<ObjModule, String> {
    let target = state.target;
    let mut e = Emitter::new(target);

    if state.use_debug_info {
        let cwd = String::from_utf8_lossy(&super::translator::getcwd_bytes()).into_owned();
        e.set_debug_meta(&cwd, &format!("MemeAssembly compiler {VERSION_STRING}"));
    }

    // Define all functions as global
    for file in 0..state.files.len() {
        for func in 0..state.files[file].functions.len() {
            let def = &state.function_commands(file, func)[0];
            if def.translate {
                e.global(&gas_symbol(def.param(0))?);
            }
        }
    }

    if target == Target::Windows {
        for name in ["GetStdHandle", "WriteFile", "ReadFile"] {
            e.extern_symbol(name);
        }
    }

    e.section(SectionKind::Data);
    e.label(".LCharacter")?;
    e.bytes(b"a");
    e.label(".Ltmp64")?;
    e.bytes(&[0; 8]);
    match target {
        Target::Linux => {
            e.label(".LsigStruct")?;
            e.label(".Lsa_handler")?;
            e.quad(0);
            e.quad(0x04000000);
            e.quad(0);
            e.quad(0);
        }
        Target::MacOS => {
            e.label(".LsigStruct")?;
            e.label(".Lsa_handler")?;
            e.quad(0);
            e.label(".Lsa_handler_2")?;
            e.quad(0);
            e.quad(0);
            e.quad(0);
        }
        Target::Windows => {}
    }

    e.section(SectionKind::Text);
    e.label(".Ltext0")?;

    if target != Target::Windows {
        kill_parent(&mut e, target)?;
    }

    // In bully mode, a missing main function is "fixed" by making the code at the start main
    if state.compile_mode == CompileMode::Bully
        && state.output_mode == OutputMode::Executable
        && !main_function_exists(state)
    {
        // See translator.rs: `_main` on macOS, no martyrdom code on Windows.
        let main_name = String::from_utf8_lossy(state.main_function_name()).into_owned();
        e.global(&main_name);
        e.label(&main_name)?;
        if target != Target::Windows {
            martyrdom_code(&mut e, target)?;
        }
    }

    let main_name = state.main_function_name();
    for (i, file) in state.files.iter().enumerate() {
        let debug_file = state
            .use_debug_info
            .then(|| e.debug_file(&String::from_utf8_lossy(&debug_file_path(&file.file_name))));

        let mut line = 0usize;
        for func in 0..file.functions.len() {
            let commands = state.function_commands(i, func);
            let function_name = commands[0].param(0);
            if let Some(f) = debug_file {
                let name = String::from_utf8_lossy(function_name);
                e.debug_function_begin(&name, f, commands[0].line_num as u32);
            }

            for (k, current_command) in commands.iter().enumerate() {
                if target != Target::Windows && state.martyrdom && k == 1 && function_name == main_name {
                    martyrdom_code(&mut e, target)?;
                }

                if line == file.random_index {
                    e.label(&format!(".LConfusedStonks_{i}"))?;
                }

                if current_command.translate {
                    let mut enc = Enc { e: &mut e, file_num: i };
                    encode_command(state, &mut enc, current_command, debug_file)?;
                }
                line += 1;
            }

            if debug_file.is_some() {
                e.debug_function_end();
            }
        }
    }

    // With -O69420 all commands are optimised out, so these are not needed
    if state.optimisation_level != OPT_69420 {
        if target == Target::Windows {
            windows_io_function(&mut e, "writechar", -11, "WriteFile")?;
            windows_io_function(&mut e, "readchar", -10, "ReadFile")?;
        } else {
            let linux = target == Target::Linux;
            unix_io_function(&mut e, "writechar", 1, if linux { 1 } else { 0x2000004 })?;
            unix_io_function(&mut e, "readchar", 0, if linux { 0 } else { 0x2000003 })?;
        }
    }

    // TODO(backend interface): `-O-s` ends the text with
    // `.align 536870912`. The Emitter has no alignment directive yet, so the padding is missing
    // on this path until `Emitter::align` exists.

    e.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_constants() {
        assert_eq!(char_value(b"'a'"), Ok(97));
        assert_eq!(char_value(b"' '"), Ok(32));
        assert_eq!(char_value(b"'''"), Ok(39));
        assert_eq!(char_value(b"'\\'"), Ok(39));
        assert_eq!(char_value(b"'\\n'"), Ok(10));
        assert_eq!(char_value(b"'\\t'"), Ok(9));
        assert_eq!(char_value(b"'\\f'"), Ok(12));
        assert_eq!(char_value(b"'\\b'"), Ok(8));
        // gas doesn't know \v and \?, they stand for the letter itself
        assert_eq!(char_value(b"'\\v'"), Ok(b'v' as i64));
        assert_eq!(char_value(b"'\\?'"), Ok(b'?' as i64));
        assert_eq!(char_value(b"'\\\"'"), Ok(b'"' as i64));
        assert_eq!(char_value(b"'\\\\'"), Ok(b'\\' as i64));
        assert_eq!(char_value(&[b'\'', 0xE4, b'\'']), Ok(0xE4));
    }

    #[test]
    fn ascii_codes_are_read_like_gas() {
        assert_eq!(char_value(b"65"), Ok(65));
        assert_eq!(char_value(b"+65"), Ok(65));
        assert_eq!(char_value(b"-0"), Ok(0));
        assert_eq!(char_value(b"0"), Ok(0));
        assert_eq!(char_value(b"010"), Ok(8));
        assert_eq!(char_value(b"00255"), Ok(173));
        assert_eq!(char_value(b"09"), Err("junk `9' after expression".to_string()));
        assert_eq!(char_value(b"08"), Err("junk `8' after expression".to_string()));
    }

    #[test]
    fn gas_numbers() {
        assert_eq!(gas_integer(b"0x45"), Ok(0x45));
        assert_eq!(gas_integer(b"0xFFFFFFFFFFFFFFFF"), Ok(-1));
        assert_eq!(gas_integer(b"0b101"), Ok(5));
        assert_eq!(gas_integer(b"-12"), Ok(-12));
    }

    #[test]
    fn symbols_are_scrubbed_like_gas() {
        assert_eq!(gas_symbol(b".L'a'Wins_0"), Ok(".L97Wins_0".to_string()));
        assert_eq!(gas_symbol(b".L'\\n'Wins_3"), Ok(".L10Wins_3".to_string()));
        assert_eq!(gas_symbol(b".L0x45Wins_0"), Ok(".L0x45Wins_0".to_string()));
        assert_eq!(gas_symbol(b".LraxWins_1"), Ok(".LraxWins_1".to_string()));
        assert_eq!(gas_symbol(b"_main"), Ok("_main".to_string()));
        assert!(gas_symbol(b".L[rax]Wins_0").is_err());
    }

    fn state_with(compile_mode: CompileMode) -> CompileState {
        let mut s = CompileState::new(Target::Linux);
        s.compile_mode = compile_mode;
        s
    }

    fn command(opcode: u8, params: &[&str], types: [u8; 2], is_pointer: u8) -> ParsedCommand {
        let mut c = ParsedCommand::new(1);
        c.opcode = opcode;
        for (i, p) in params.iter().enumerate() {
            c.parameters[i] = Some(p.as_bytes().to_vec());
        }
        c.param_types = types;
        c.is_pointer = is_pointer;
        c
    }

    #[test]
    fn decimal_and_char_operands() {
        let state = state_with(CompileMode::Noob);
        // "{p} is brilliant, but I like {p}" (mov)
        let c = command(10, &["rax", "-5"], [PARAM_REG64, PARAM_DECIMAL], 0);
        let a = CmdArgs { state: &state, cmd: &c };
        assert_eq!(a.op(1), Ok(Operand::Imm(-5)));
        assert_eq!(a.text(1), b"0xFFFFFFFFFFFFFFFB");

        let c = command(10, &["al", "'\\n'"], [PARAM_REG8, PARAM_CHAR], 0);
        let a = CmdArgs { state: &state, cmd: &c };
        assert_eq!(a.op(1), Ok(Operand::Imm(10)));
    }

    #[test]
    fn register_and_pointer_operands() {
        let state = state_with(CompileMode::Noob);
        let c = command(10, &["rbx", "eax"], [PARAM_REG64, PARAM_REG32], 1);
        let a = CmdArgs { state: &state, cmd: &c };
        assert_eq!(a.op(0), Ok(mem_reg(None, Reg::R64(3), 0)));
        assert_eq!(a.op(1), Ok(Operand::Reg(Reg::R32(0))));

        // Bully mode guesses the operand size from computedIndex (69 % 4 == 1 → WORD PTR)
        let state = state_with(CompileMode::Bully);
        let c = command(10, &["rbx", "5"], [PARAM_REG64, PARAM_DECIMAL], 1);
        let a = CmdArgs { state: &state, cmd: &c };
        assert_eq!(a.op(0), Ok(mem_reg(Some(Size::Word), Reg::R64(3), 0)));
        assert_eq!(a.text(0), b"WORD PTR [rbx]");
    }
}
