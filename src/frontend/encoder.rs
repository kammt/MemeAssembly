//! Direct code generation for object files and executables: the backend's [`Emitter`]
//! implements the translator's [`Asm`], so the same [`translate`] that prints the `-S` text
//! encodes machine code into an [`ObjModule`] for the backend to write.

use super::state::CompileState;
use super::log::VERSION_STRING;
use super::translator::{getcwd_bytes, translate, Asm, AsmResult, Imm, Opnd};
use crate::backend::emitter::{BranchTarget, Cond, Emitter, Op, Operand, SectionKind};
use crate::backend::module::ObjModule;

/// Generates the program into an object module.
pub fn encode_module(state: &CompileState) -> Result<ObjModule, String> {
    let mut e = Emitter::new(state.target);
    if state.use_debug_info {
        e.set_debug_meta(&getcwd_bytes(), &format!("MemeAssembly compiler {VERSION_STRING}"));
    }
    translate(state, &mut e)?;
    e.finish()
}

/// Symbol names are ASCII (labels are built from validated parameters).
fn symbol(name: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(name)
        .map_err(|_| format!("internal error: invalid symbol name `{}'", String::from_utf8_lossy(name)))
}

impl Asm for Emitter {
    fn insn(&mut self, op: Op, ops: &[Opnd]) -> AsmResult {
        let ops: Vec<Operand> = ops
            .iter()
            .map(|o| match o {
                Opnd::Reg(r) => Operand::Reg(*r),
                Opnd::Imm(imm) => Operand::Imm(imm.value()),
                Opnd::Mem(m) => Operand::Mem(m.clone()),
            })
            .collect();
        Emitter::insn(self, op, &ops)
    }
    fn jump(&mut self, cond: Option<Cond>, target: &[u8]) -> AsmResult {
        Emitter::jump(self, cond, BranchTarget::Symbol(symbol(target)?.to_string()))
    }
    fn jump_local(&mut self, cond: Option<Cond>, n: u32, forward: bool) -> AsmResult {
        Emitter::jump(self, cond, BranchTarget::Local { n, forward })
    }
    fn call(&mut self, target: &[u8]) -> AsmResult {
        Emitter::call(self, BranchTarget::Symbol(symbol(target)?.to_string()))
    }
    fn label(&mut self, name: &[u8]) -> AsmResult {
        Emitter::label(self, symbol(name)?)
    }
    fn local_label(&mut self, n: u32) {
        Emitter::local_label(self, n)
    }
    fn global(&mut self, name: &[u8]) {
        Emitter::global(self, &String::from_utf8_lossy(name))
    }
    fn extern_symbol(&mut self, name: &str) {
        Emitter::extern_symbol(self, name)
    }
    fn section(&mut self, kind: SectionKind) {
        Emitter::section(self, kind)
    }
    fn ascii(&mut self, data: &[u8]) {
        self.bytes(data)
    }
    fn zeros(&mut self, n: usize) {
        self.bytes(&vec![0; n])
    }
    fn quad(&mut self, values: &[Imm]) {
        for v in values {
            Emitter::quad(self, v.value());
        }
    }
    fn align(&mut self, bytes: u64) -> AsmResult {
        Emitter::align(self, bytes)
    }
    fn debug_file(&mut self, path: &[u8]) -> usize {
        Emitter::debug_file(self, path)
    }
    fn debug_line(&mut self, file: usize, line: u32) {
        Emitter::debug_line(self, file, line)
    }
    fn debug_function_begin(&mut self, name: &[u8], file: usize, line: u32) {
        Emitter::debug_function_begin(self, &String::from_utf8_lossy(name), file, line)
    }
    fn debug_function_end(&mut self) {
        Emitter::debug_function_end(self)
    }
    /// gas reads character constants in symbol names as their decimal value.
    fn char_in_symbol(&self, value: u8, _text: &[u8], _verbatim: bool) -> Vec<u8> {
        value.to_string().into_bytes()
    }
}
