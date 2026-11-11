//! Target operating system. The original C compiler selected this at *build* time via
//! `-D LINUX` / `-D MACOS` / `-D WINDOWS`. The Rust port keeps the host as the default but
//! makes it a runtime value so that every target can be produced (and tested) from any host.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Linux,
    MacOS,
    Windows,
}

impl Target {
    /// The target matching the operating system this compiler was built for
    /// (equivalent to the C build's platform macro).
    pub fn host() -> Target {
        if cfg!(target_os = "windows") {
            Target::Windows
        } else if cfg!(target_os = "macos") {
            Target::MacOS
        } else {
            Target::Linux
        }
    }

    pub fn from_name(name: &str) -> Option<Target> {
        match name {
            "linux" => Some(Target::Linux),
            "macos" => Some(Target::MacOS),
            "windows" => Some(Target::Windows),
            _ => None,
        }
    }
}
