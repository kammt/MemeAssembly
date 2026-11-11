//! Replaces the original `popen("gcc ... -x assembler -")`: assembles the generated Intel-syntax
//! assembly text with fadec and writes object files / executables directly.

pub mod assembler;
pub mod fadec;
pub mod module;
pub mod writers;

use std::path::Path;

use crate::target::Target;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputKind {
    /// Relocatable object (`-O`): ELF64 on Linux, COFF on Windows, Mach-O MH_OBJECT on macOS.
    Object,
    /// Linked executable (default): ELF64 ET_EXEC on Linux, PE32+ on Windows, Mach-O on macOS.
    Executable,
}

/// Assemble `asm` (exactly the text that `-S` would write) and write `kind` for `target`
/// to `output`. Errors are human readable messages (the gas-like diagnostic).
pub fn emit(asm: &str, target: Target, kind: OutputKind, output: &Path) -> Result<(), String> {
    let module = assembler::assemble(asm, target)?;
    let bytes = writers::write(&module, target, kind)?;
    write_output(output, &bytes, kind)
}

fn write_output(output: &Path, bytes: &[u8], kind: OutputKind) -> Result<(), String> {
    std::fs::write(output, bytes).map_err(|e| format!("cannot write {}: {e}", output.display()))?;
    #[cfg(unix)]
    if kind == OutputKind::Executable {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(output, std::fs::Permissions::from_mode(0o755));
    }
    let _ = kind;
    Ok(())
}
