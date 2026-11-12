//! Target operating system. It is chosen at run time with `--target` and defaults to the host,
//! so every target can be produced (and tested) from any host. It decides the object format,
//! symbol naming and the platform-specific parts of the generated code.

/// The operating system the compiled program runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Target {
    Linux,
    #[value(name = "macos")]
    MacOS,
    Windows,
}

impl Target {
    /// The target matching the operating system this compiler was built for (Linux for any
    /// OS other than Windows and macOS).
    pub fn host() -> Target {
        if cfg!(target_os = "windows") {
            Target::Windows
        } else if cfg!(target_os = "macos") {
            Target::MacOS
        } else {
            Target::Linux
        }
    }
}
