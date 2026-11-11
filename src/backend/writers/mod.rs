//! Object/executable file writers. OWNED BY: writers agent.
use super::module::ObjModule;
use super::OutputKind;
use crate::target::Target;

pub fn write(_module: &ObjModule, _target: Target, _kind: OutputKind) -> Result<Vec<u8>, String> {
    Err("writers not implemented yet".into())
}
