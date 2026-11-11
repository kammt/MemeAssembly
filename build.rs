//! Builds the vendored fadec encoder and generates Rust constants for its mnemonics.
use std::{env, fs, path::PathBuf};

fn main() {
    let fadec = PathBuf::from("vendor/fadec");
    println!("cargo:rerun-if-changed=vendor/fadec");

    cc::Build::new()
        .file(fadec.join("encode.c"))
        .include(&fadec)
        .std("c11")
        .warnings(false)
        .compile("fadec");

    // Turn `#define FE_ADD64rr 0x...` lines into `pub const FE_ADD64rr: u64 = 0x...;`
    let public = fs::read_to_string(fadec.join("fadec-encode-public.inc"))
        .expect("read fadec-encode-public.inc");
    let mut out = String::with_capacity(public.len());
    for line in public.lines() {
        let mut parts = line.split_whitespace();
        if let (Some("#define"), Some(name), Some(value), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        {
            out.push_str(&format!("pub const {name}: u64 = {value};\n"));
        }
    }
    let dest = PathBuf::from(env::var("OUT_DIR").unwrap()).join("fadec_mnemonics.rs");
    fs::write(dest, out).expect("write fadec_mnemonics.rs");
}
