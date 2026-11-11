# Vendored fadec

This is the x86-64 **encoder** of [fadec](https://github.com/aengelke/fadec) by Alexis Engelke
(BSD-3-Clause, see `LICENSE`). Upstream commit: see `UPSTREAM_COMMIT`.

Only the files needed for encoding are vendored. The encoder tables
(`fadec-encode-public.inc`, `fadec-encode-private.inc`) are pre-generated so that building
MemeAssembly needs neither Python nor Meson:

    python3 parseinstrs.py encode instrs.txt fadec-encode-public.inc fadec-encode-private.inc --64

`build.rs` compiles `encode.c` with the `cc` crate and turns the `#define FE_*` lines of
`fadec-encode-public.inc` into Rust constants (`crate::backend::fadec::FE_*`).
