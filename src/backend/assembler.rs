//! Intel-syntax assembler for the subset of GNU as syntax emitted by the translator.
//! OWNED BY: assembler agent.
use super::module::ObjModule;
use crate::target::Target;

pub fn assemble(_asm: &str, _target: Target) -> Result<ObjModule, String> {
    Err("assembler not implemented yet".into())
}
