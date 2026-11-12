//! Replaces the original `popen("gcc ... -x assembler -")`: the translator drives
//! [`emitter::Emitter`] (instructions encoded directly in Rust, no assembly text involved),
//! and [`writers`] turn the resulting [`module::ObjModule`] into object files / executables.

pub mod emitter;
pub mod module;
pub mod writers;

use std::path::Path;

use crate::target::Target;
use module::ObjModule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputKind {
    /// Relocatable object (`-O`): ELF64 on Linux, COFF on Windows, Mach-O MH_OBJECT on macOS.
    Object,
    /// Linked executable (default): ELF64 ET_EXEC on Linux, PE32+ on Windows, Mach-O on macOS.
    Executable,
}

/// Write `module` as `kind` for `target` to `output`. Errors are human readable messages
/// (link errors like ld's "undefined reference to `foo'").
pub fn emit(module: &ObjModule, target: Target, kind: OutputKind, output: &Path) -> Result<(), String> {
    let bytes = writers::write(module, target, kind)?;
    std::fs::write(output, &bytes).map_err(|e| format!("cannot write {}: {e}", output.display()))?;
    #[cfg(unix)]
    if kind == OutputKind::Executable {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(output, std::fs::Permissions::from_mode(0o755));
    }
    Ok(())
}
