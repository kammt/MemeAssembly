//! Tests for the emitter, plus a test-only "text → Emitter" shim used by the differential
//! harness (`tests/emitter_diff.sh`): the harness feeds the same Intel-syntax lines to GNU as
//! and, through this shim, to the [`Emitter`], and compares the results.
//!
//! The shim only exists to drive the typed API from a corpus; it emulates the bits of gas's
//! text handling (character constants, number syntax, labels on the same line, ...) that the
//! MemeAssembly frontend resolves itself before calling the emitter.

use super::*;
use crate::backend::module::{Binding, RelocKind};
use std::fmt::Write as _;

// ------------------------------------------------------------------------------------------
// gas-like preprocessing
// ------------------------------------------------------------------------------------------

/// gas `process_escape` for character constants.
/// Emulates gas's input scrubber for one line: removes comments, converts `'c'` character
/// constants to decimal numbers and splits `;`-separated statements.
fn scrub(line: &str) -> Vec<String> {
    let mut stmts = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut in_str = false;
    if line.trim_start().starts_with('/') {
        return stmts;
    }
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            cur.push(c);
            if c == '\\' && i + 1 < chars.len() {
                cur.push(chars[i + 1]);
                i += 1;
            } else if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => {
                in_str = true;
                cur.push(c);
                i += 1;
            }
            '#' => break,
            ';' => {
                stmts.push(std::mem::take(&mut cur));
                i += 1;
            }
            '\'' => {
                // Character constant: up to the closing quote (or one char / one escape).
                let mut j = i + 1;
                if j < chars.len() && chars[j] == '\\' {
                    j += 1;
                }
                j += 1;
                if j < chars.len() && chars[j] == '\'' {
                    j += 1;
                }
                let j = j.min(chars.len());
                let lit: String = chars[i..j].iter().collect();
                cur.push_str(&char_constants(&lit));
                i = j;
                if j == chars.len() && lit.len() == 1 {
                    break;
                }
            }
            _ => {
                cur.push(c);
                i += 1;
            }
        }
    }
    stmts.push(cur);
    stmts
}

/// Parse a gas integer literal at the start of `s`. Returns (value, rest).
fn parse_number(s: &str) -> Option<(i64, &str)> {
    let b = s.as_bytes();
    if b.is_empty() || !b[0].is_ascii_digit() {
        return None;
    }
    let (radix, start) = if b.len() > 1 && b[0] == b'0' && (b[1] == b'x' || b[1] == b'X') {
        (16, 2)
    } else if b.len() > 1 && b[0] == b'0' && (b[1] == b'b' || b[1] == b'B') {
        (2, 2)
    } else if b[0] == b'0' {
        (8, 1)
    } else {
        (10, 0)
    };
    let mut end = start;
    while end < b.len() && (b[end] as char).is_digit(radix) {
        end += 1;
    }
    let digits = &s[start..end];
    let v = if digits.is_empty() {
        0
    } else {
        u64::from_str_radix(digits, radix).ok()? as i64
    };
    Some((v, &s[end..]))
}

/// Parse an expression of the form `[+-~]* number`.
fn parse_imm(s: &str) -> Result<i64, String> {
    let s = s.trim();
    let mut neg = Vec::new();
    let mut rest = s;
    loop {
        let r = rest.trim_start();
        if let Some(r2) = r.strip_prefix('-') {
            neg.push('-');
            rest = r2;
        } else if let Some(r2) = r.strip_prefix('+') {
            rest = r2;
        } else if let Some(r2) = r.strip_prefix('~') {
            neg.push('~');
            rest = r2;
        } else {
            rest = r;
            break;
        }
    }
    let (mut v, tail) = parse_number(rest).ok_or_else(|| format!("bad expression `{s}'"))?;
    let tail = tail.trim();
    if !tail.is_empty() {
        return Err(format!("junk `{tail}' after expression"));
    }
    for op in neg.iter().rev() {
        v = if *op == '-' { v.wrapping_neg() } else { !v };
    }
    Ok(v)
}

fn parse_size(s: &str) -> Option<(Size, &str)> {
    let lower = s.to_ascii_lowercase();
    for (kw, sz) in [
        ("byte", Size::Byte),
        ("word", Size::Word),
        ("dword", Size::Dword),
        ("qword", Size::Qword),
        ("xmmword", Size::Xmmword),
    ] {
        if let Some(r) = lower.strip_prefix(kw) {
            let r = r.trim_start();
            if r.starts_with("ptr") {
                let consumed = s.len() - r.len() + 3;
                return Some((sz, &s[consumed..]));
            }
        }
    }
    None
}

fn parse_operand(s: &str) -> Result<Operand, String> {
    let s = s.trim();
    let (size, body) = match parse_size(s) {
        Some((sz, r)) => (Some(sz), r.trim()),
        None => (None, s),
    };
    if body.starts_with('[') && body.ends_with(']') {
        let inner = &body[1..body.len() - 1];
        let mut base: Option<Base> = None;
        let mut disp: i64 = 0;
        // split into signed terms
        let mut terms: Vec<(bool, String)> = Vec::new();
        let mut cur = String::new();
        let mut neg = false;
        for c in inner.chars() {
            if c == '+' || c == '-' {
                if !cur.trim().is_empty() {
                    terms.push((neg, cur.trim().to_string()));
                    cur.clear();
                    neg = c == '-';
                } else if c == '-' {
                    neg = !neg;
                }
            } else {
                cur.push(c);
            }
        }
        if !cur.trim().is_empty() {
            terms.push((neg, cur.trim().to_string()));
        }
        let mut rip = false;
        for (neg, t) in terms {
            if t.eq_ignore_ascii_case("rip") {
                rip = true;
                continue;
            }
            if let Some(r) = Reg::parse(&t) {
                if base.is_some() {
                    return Err("shim: index registers unsupported".into());
                }
                base = Some(Base::Reg(r));
                continue;
            }
            if let Some((v, rest)) = parse_number(&t) {
                if !rest.is_empty() {
                    return Err(format!("junk `{rest}' after expression"));
                }
                disp = if neg { disp.wrapping_sub(v) } else { disp.wrapping_add(v) };
                continue;
            }
            if rip {
                base = Some(Base::Rip(t));
                continue;
            }
            return Err(format!("shim: unsupported memory term `{t}'"));
        }
        let base = match base {
            Some(b) => b,
            None if rip => return Err("shim: bare rip".into()),
            None => Base::Abs,
        };
        return Ok(Operand::Mem(Mem { size, base, disp }));
    }
    if size.is_some() {
        return Err(format!("shim: bad memory operand `{s}'"));
    }
    if let Some(r) = Reg::parse(body) {
        return Ok(Operand::Reg(r));
    }
    // `r13 r13`
    let first = body.split_whitespace().next().unwrap_or("");
    if Reg::parse(first).is_some() {
        return Err(format!("junk `{}' after expression", body[first.len()..].trim()));
    }
    parse_imm(body).map(Operand::Imm)
}

fn split_operands(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut cur = String::new();
    for c in s.chars() {
        match c {
            '[' => depth += 1,
            ']' => depth -= 1,
            ',' if depth == 0 => {
                out.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    if !cur.trim().is_empty() || !out.is_empty() {
        out.push(cur);
    }
    out.into_iter().map(|o| o.trim().to_string()).collect()
}

fn op_by_name(m: &str) -> Option<Op> {
    Some(match m {
        "mov" => Op::Mov,
        "movups" => Op::Movups,
        "lea" => Op::Lea,
        "push" => Op::Push,
        "pop" => Op::Pop,
        "add" => Op::Add,
        "sub" => Op::Sub,
        "and" => Op::And,
        "xor" => Op::Xor,
        "cmp" => Op::Cmp,
        "test" => Op::Test,
        "not" => Op::Not,
        "inc" => Op::Inc,
        "dec" => Op::Dec,
        "shl" => Op::Shl,
        "shr" => Op::Shr,
        "imul" => Op::Imul,
        "idiv" => Op::Idiv,
        "cqo" => Op::Cqo,
        "ret" => Op::Ret,
        "nop" => Op::Nop,
        "hlt" => Op::Hlt,
        "int3" => Op::Int3,
        "syscall" => Op::Syscall,
        "rdrand" => Op::Rdrand,
        _ => return None,
    })
}

fn cond_by_name(m: &str) -> Option<Option<Cond>> {
    Some(match m {
        "jmp" => None,
        "je" => Some(Cond::E),
        "jne" => Some(Cond::Ne),
        "jz" => Some(Cond::Z),
        "jnz" => Some(Cond::Nz),
        "jg" => Some(Cond::G),
        "jl" => Some(Cond::L),
        "jge" => Some(Cond::Ge),
        "jle" => Some(Cond::Le),
        "ja" => Some(Cond::A),
        "jb" => Some(Cond::B),
        _ => return None,
    })
}

fn branch_target(s: &str) -> BranchTarget {
    let s = s.trim();
    if s.len() >= 2 && s[..s.len() - 1].bytes().all(|b| b.is_ascii_digit()) {
        let n: u32 = s[..s.len() - 1].parse().unwrap_or(0);
        match s.as_bytes()[s.len() - 1] {
            b'f' | b'F' => return BranchTarget::Local { n, forward: true },
            b'b' | b'B' => return BranchTarget::Local { n, forward: false },
            _ => {}
        }
    }
    BranchTarget::Symbol(s.to_string())
}

fn parse_ascii(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    let inner = s
        .strip_prefix('"')
        .and_then(|r| r.strip_suffix('"'))
        .ok_or("shim: bad string")?;
    let mut out = Vec::new();
    let mut it = inner.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            let e = it.next().ok_or("shim: bad escape")?;
            out.push(escape_value(e) as u8);
        } else {
            out.push(c as u8);
        }
    }
    Ok(out)
}

fn is_label_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '$'
}

/// Feed one statement to the emitter.
fn statement(em: &mut Emitter, stmt: &str) -> Result<(), String> {
    let mut s = stmt.trim();
    // Labels (possibly several, possibly followed by an instruction).
    loop {
        let n = s.chars().take_while(|c| is_label_char(*c)).count();
        if n > 0 && s[n..].starts_with(':') {
            let name = &s[..n];
            if name.bytes().all(|b| b.is_ascii_digit()) {
                em.local_label(name.parse().map_err(|_| "shim: bad local label")?);
            } else {
                em.label(name)?;
            }
            s = s[n + 1..].trim();
            continue;
        }
        break;
    }
    if s.is_empty() {
        return Ok(());
    }
    // A label made of characters gas doesn't accept (who-would-win labels with odd params).
    if s.ends_with(':') && !s.contains(',') {
        let first = s.split_whitespace().next().unwrap_or("").to_ascii_lowercase();
        if op_by_name(&first).is_none() && cond_by_name(&first).is_none() && first != "call" {
            return em.label(&s[..s.len() - 1]);
        }
    }
    let (mn, rest) = match s.find(|c: char| c.is_whitespace()) {
        Some(i) => (&s[..i], s[i..].trim()),
        None => (s, ""),
    };
    let mnl = mn.to_ascii_lowercase();
    if mnl.starts_with('.') {
        return directive(em, &mnl, rest);
    }
    if let Some(op) = op_by_name(&mnl) {
        let ops = split_operands(rest)
            .iter()
            .map(|o| parse_operand(o))
            .collect::<Result<Vec<_>, _>>()?;
        return em.insn(op, &ops);
    }
    if mnl == "call" {
        return em.call(branch_target(rest));
    }
    if let Some(c) = cond_by_name(&mnl) {
        return em.jump(c, branch_target(rest));
    }
    if let Some(c) = s.chars().next().filter(|c| c.is_ascii_digit()) {
        return Err(format!("junk at end of line, first unrecognized character is `{c}'"));
    }
    Err(format!("no such instruction: `{}'", s.to_ascii_lowercase()))
}

fn directive(em: &mut Emitter, d: &str, rest: &str) -> Result<(), String> {
    match d {
        ".intel_syntax" | ".file" | ".loc" | ".stabs" | ".stabn" => Ok(()),
        ".global" | ".globl" => {
            em.global(rest.trim());
            Ok(())
        }
        ".extern" => {
            em.extern_symbol(rest.trim());
            Ok(())
        }
        ".data" => {
            em.section(SectionKind::Data);
            Ok(())
        }
        ".text" => {
            em.section(SectionKind::Text);
            Ok(())
        }
        ".ascii" => {
            let b = parse_ascii(rest)?;
            em.bytes(&b);
            Ok(())
        }
        ".byte" => {
            for v in rest.split(',') {
                em.bytes(&[parse_imm(v)? as u8]);
            }
            Ok(())
        }
        ".quad" => {
            for v in rest.split(',') {
                em.quad(parse_imm(v)?);
            }
            Ok(())
        }
        ".align" | ".balign" => em.align(parse_imm(rest)? as u64),
        _ => Err(format!("unknown pseudo-op: `{d}'")),
    }
}

/// Run a whole source through the shim. Returns `(errors as (line, msg), module)`.
pub(super) fn assemble_text(src: &str, target: Target) -> (Vec<(usize, String)>, Option<ObjModule>) {
    let mut em = Emitter::new(target);
    let mut errors = Vec::new();
    for (i, line) in src.lines().enumerate() {
        for stmt in scrub(line) {
            if let Err(e) = statement(&mut em, &stmt) {
                errors.push((i + 1, e));
            }
        }
    }
    match em.finish() {
        Ok(m) if errors.is_empty() => (errors, Some(m)),
        Ok(_) => (errors, None),
        Err(e) => {
            errors.extend(e.lines().map(|l| (0, l.to_string())));
            (errors, None)
        }
    }
}

/// CRC-32 (zlib polynomial), used to compare huge sections (`-O-s`) without dumping them.
fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, t) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *t = c;
    }
    let mut c = !0u32;
    for &b in data {
        c = table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    !c
}

/// Text dump consumed by `tests/emitter_diff.py`.
pub(super) fn dump(errors: &[(usize, String)], module: Option<&ObjModule>) -> String {
    let mut out = String::new();
    for (l, e) in errors {
        let _ = writeln!(out, "ERR {l} {e}");
    }
    let Some(m) = module else { return out };
    for (si, sec) in m.sections.iter().enumerate() {
        let name = if si == 0 { "text" } else { "data" };
        if sec.data.len() > 1 << 20 {
            let _ = writeln!(out, "SEC {name} {} {} CRC:{:08x}", sec.align, sec.data.len(), crc32(&sec.data));
        } else {
            let hex: String = sec.data.iter().map(|b| format!("{b:02x}")).collect();
            let _ = writeln!(out, "SEC {name} {} {} {hex}", sec.align, sec.data.len());
        }
        for r in &sec.relocs {
            let s = &m.symbols[r.symbol];
            let kind = match r.kind {
                RelocKind::Pc32 => "Pc32",
                RelocKind::Branch32 => "Branch32",
                RelocKind::Abs32 => "Abs32",
                RelocKind::Abs64 => "Abs64",
            };
            let def = match s.definition {
                Some((sec, off)) => format!("{} {off}", if sec == 0 { "text" } else { "data" }),
                None => "UND 0".into(),
            };
            let scope = if s.temporary {
                "temp"
            } else if s.binding == Binding::Local {
                "local"
            } else {
                "global"
            };
            let _ = writeln!(
                out,
                "REL {name} {} {kind} {} {} {} {def}",
                r.offset,
                s.name.replace(' ', "\\x20"),
                r.addend,
                scope
            );
        }
    }
    for s in &m.symbols {
        if s.temporary {
            continue;
        }
        let def = match s.definition {
            Some((sec, off)) => format!("{} {off}", if sec == 0 { "text" } else { "data" }),
            None => "UND 0".into(),
        };
        let bind = if s.binding == Binding::Global { "GLOBAL" } else { "LOCAL" };
        let _ = writeln!(out, "SYM {} {bind} {def}", s.name.replace(' ', "\\x20"));
    }
    out
}

/// Differential-harness entry point: for every `*.s` in `$EMITTER_CORPUS` write `*.emit`.
/// Files whose name contains `.win.` are assembled for Windows, `.mac.` for macOS.
#[test]
#[ignore]
fn dump_corpus() {
    let Ok(dir) = std::env::var("EMITTER_CORPUS") else {
        eprintln!("EMITTER_CORPUS not set");
        return;
    };
    let mut n = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().is_none_or(|e| e != "s") {
            continue;
        }
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        let target = if name.contains(".win.") {
            Target::Windows
        } else if name.contains(".mac.") {
            Target::MacOS
        } else {
            Target::Linux
        };
        let src = std::fs::read_to_string(&p).unwrap();
        let (errors, module) = assemble_text(&src, target);
        std::fs::write(p.with_extension("emit"), dump(&errors, module.as_ref())).unwrap();
        n += 1;
    }
    eprintln!("dumped {n} files");
}

// ------------------------------------------------------------------------------------------
// Unit tests (expected bytes taken from GNU as 2.46)
// ------------------------------------------------------------------------------------------

fn text_of(src: &str) -> Vec<u8> {
    let (errors, m) = assemble_text(src, Target::Linux);
    assert!(errors.is_empty(), "{errors:?}");
    m.unwrap().sections[0].data.clone()
}

fn err_of(src: &str) -> String {
    let (errors, _) = assemble_text(src, Target::Linux);
    assert!(!errors.is_empty(), "expected an error for {src}");
    errors[0].1.clone()
}

fn hex(s: &str) -> Vec<u8> {
    let s: String = s.split_whitespace().collect();
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

#[test]
fn basic_forms() {
    assert_eq!(text_of("mov rax, 1"), hex("48 c7 c0 01 00 00 00"));
    assert_eq!(text_of("mov rax, 0xFFFFFFFF"), hex("48 b8 ff ff ff ff 00 00 00 00"));
    assert_eq!(text_of("add ecx, 0x1FFFFFFFF"), hex("81 c1 ff ff ff ff"));
    assert_eq!(text_of("add eax, 1"), hex("83 c0 01"));
    assert_eq!(text_of("push 0xFF"), hex("68 ff 00 00 00"));
    assert_eq!(text_of("push -0"), hex("6a 00"));
    assert_eq!(text_of("shl rax, 1"), hex("48 d1 e0"));
    assert_eq!(text_of("test rsp, 0xF"), hex("48 f7 c4 0f 00 00 00"));
    assert_eq!(text_of("mov rax, [69]"), hex("48 8b 04 25 45 00 00 00"));
    assert_eq!(text_of("mov rax, [eax]"), hex("67 48 8b 00"));
    assert_eq!(text_of("imul eax, 0x80000000"), hex("69 c0 00 00 00 80"));
    assert_eq!(text_of("imul ax, 0x12345"), hex("66 69 c0 45 23"));
    assert_eq!(text_of("call r8"), hex("41 ff d0"));
    assert_eq!(text_of("call $"), hex("e8 fb ff ff ff"));
}

#[test]
fn rip_relative_and_relocs() {
    let (e, m) = assemble_text(
        ".data\n.Lc: .ascii \"a\"\n.text\nmov BYTE PTR [rip + .Lc], 0x61\ncall foo\n",
        Target::Linux,
    );
    assert!(e.is_empty());
    let m = m.unwrap();
    let t = &m.sections[0];
    assert_eq!(t.relocs.len(), 2);
    assert_eq!((t.relocs[0].offset, t.relocs[0].kind, t.relocs[0].addend), (2, RelocKind::Pc32, -5));
    assert_eq!((t.relocs[1].offset, t.relocs[1].kind, t.relocs[1].addend), (8, RelocKind::Branch32, -4));
    assert_eq!(m.symbols[t.relocs[1].symbol].name, "foo");
}

#[test]
fn relaxation() {
    let mut src = String::from("1: xor rax, rax\n");
    for _ in 0..40 {
        src.push_str("mov rax, 1\n");
    }
    src.push_str("jmp 1b\njz 1b\n");
    let t = text_of(&src);
    // 3 + 40*7 = 283 bytes before the jmp: rel32 needed.
    assert_eq!(&t[283..288], &hex("e9 e0 fe ff ff")[..]);
    assert_eq!(&t[288..290], &hex("0f 84")[..]);
}

#[test]
fn errors() {
    assert_eq!(err_of("add [rax], 1"), "ambiguous operand size for `add'");
    assert_eq!(err_of("mov ah, sil"), "can't encode register 'ah' in an instruction requiring REX/REX2 prefix");
    assert_eq!(err_of("x:\nnop\nx:"), "symbol `x' is already defined");
    assert_eq!(err_of("jmp 3b"), "backward ref to unknown label \"3:\"");
    assert_eq!(err_of("xor r13 r13"), "junk `r13' after expression");
}

#[test]
fn nop_padding() {
    let t = text_of("nop\n.align 16");
    assert_eq!(t, hex("90 0f1f4000 66662e0f1f840000000000"));
    let t = text_of("ret\n.align 256");
    assert_eq!(&t[..6], &hex("c3 e9 fa 00 00 00")[..]);
    assert_eq!(t.len(), 256);
}

#[test]
fn raw_label_text() {
    // The frontend passes label text exactly as it prints it with -S.
    let mut em = Emitter::new(Target::Linux);
    em.label(".L'a'Wins_0").unwrap();
    em.jump(Some(Cond::G), BranchTarget::Symbol(".L97Wins_0".into())).unwrap();
    em.jump(None, BranchTarget::Symbol(".L' 'Wins_0".into())).unwrap();
    em.label(".L32Wins_0").unwrap();
    em.label(".L' 'Wins_0").unwrap(); // same position: tolerated like gas
    em.insn(Op::Nop, &[]).unwrap();
    assert_eq!(em.label(".L32Wins_0").unwrap_err(), "symbol `.L32Wins_0' is already defined");
    assert_eq!(
        em.jump(Some(Cond::L), BranchTarget::Symbol(".LBYTE PTR [rax]Wins_0".into())).unwrap_err(),
        "junk `PTR [rax]Wins_0' after expression"
    );
    assert_eq!(
        em.jump(None, BranchTarget::Symbol(".L-5Wins_0".into())).unwrap_err(),
        "junk `Wins_0' after expression"
    );
    let m = em.finish().unwrap();
    assert_eq!(m.sections[0].data, hex("7f fe eb 00 90"));
}

#[test]
fn shifts_and_sizes() {
    assert_eq!(text_of("shl al, 255"), hex("c0 e0 ff"));
    assert_eq!(text_of("shl al, 0x10000"), hex("c0 e0 00"));
    assert_eq!(text_of("shl rax, 255"), hex("48 c1 e0 ff"));
    assert_eq!(text_of("shl rax, cl"), hex("48 d3 e0"));
    assert_eq!(text_of("shr BYTE PTR [rax], 1"), hex("d0 28"));
    assert_eq!(err_of("shl eax, -1"), "operand type mismatch for `shl'");
    assert_eq!(err_of("shl [rax], cl"), "ambiguous operand size for `shl'");
    assert_eq!(err_of("shl [rax], 256"), "operand type mismatch for `shl'");
    assert_eq!(err_of("imul al, 5"), "operand size mismatch for `imul'");
    assert_eq!(err_of("imul rax, al"), "operand type mismatch for `imul'");
    assert_eq!(err_of("imul rax, WORD PTR [rbx]"), "register type mismatch for `imul'");
    assert_eq!(err_of("add rax, WORD PTR [rbx]"), "register type mismatch for `add'");
    assert_eq!(err_of("test WORD PTR [rbx], rax"), "operand type mismatch for `test'");
    assert_eq!(err_of("mov BYTE PTR [rax], DWORD PTR [ax]"), "conflicting operand size modifiers");
    assert_eq!(err_of("mov [rax], [eax + 8]"), "`[eax+8]' is not a valid base/index expression");
    assert_eq!(err_of("movups WORD PTR [rax], WORD PTR [rax]"), "operand size mismatch for `movups'");
}

/// `(offset, kind, symbol, addend)` of the `.text` relocations.
fn text_relocs(m: &ObjModule) -> Vec<(u64, RelocKind, String, i64)> {
    m.sections[0].relocs.iter().map(|r| (r.offset, r.kind, m.symbols[r.symbol].name.clone(), r.addend)).collect()
}

const RELOC_SRC: &str = ".global gf\ngf: ret\nlf: ret\nstart:\ncall gf\ncall lf\njmp gf\nlea rax, [rip + gf]\n\
                         lea rax, [rip + lf]\njmp start\njmp .Lx\n.Lx: jz Lx\nLx: ret\n";

#[test]
fn relocations_elf() {
    // gas: calls and RIP references to global symbols keep their relocation (interposition);
    // `jmp`/`jcc` to them and references to local symbols are resolved.
    let (e, m) = assemble_text(RELOC_SRC, Target::Linux);
    assert!(e.is_empty(), "{e:?}");
    let m = m.unwrap();
    assert_eq!(
        m.sections[0].data,
        hex("c3 c3 e8 00000000 e8 f5ffffff eb f2 488d05 00000000 488d05 e5ffffff eb e4 eb 00 74 00 c3")
    );
    assert_eq!(
        text_relocs(&m),
        [(3, RelocKind::Branch32, "gf".into(), -4), (17, RelocKind::Pc32, "gf".into(), -4)]
    );
}

#[test]
fn relocations_coff() {
    let (e, m) = assemble_text(RELOC_SRC, Target::Windows);
    assert!(e.is_empty(), "{e:?}");
    let m = m.unwrap();
    assert_eq!(
        &m.sections[0].data[..35],
        &hex("c3 c3 e8 f9ffffff e8 f5ffffff eb f2 488d05 ebffffff 488d05 e5ffffff eb e4 eb 00 74 00 c3")[..]
    );
    assert!(m.sections[0].relocs.is_empty());
    // `L` labels are temporary for COFF as well.
    assert!(m.symbols.iter().any(|s| s.name == "Lx" && s.temporary));
}

#[test]
fn relocations_macho() {
    // LLVM: only references within one atom (from a non-temporary symbol to the next) are
    // resolved; `.L` labels are ordinary symbols, `L` labels are temporary. Expected bytes and
    // relocations from llvm-mc -triple x86_64-apple-macos.
    let (e, m) = assemble_text(RELOC_SRC, Target::MacOS);
    assert!(e.is_empty(), "{e:?}");
    let m = m.unwrap();
    assert_eq!(
        m.sections[0].data,
        hex("c3 c3 e8 00000000 e8 00000000 e9 00000000 488d05 00000000 488d05 00000000 eb e1 e9 00000000 \
             74 00 c3")
    );
    assert_eq!(
        text_relocs(&m),
        [
            (3, RelocKind::Branch32, "gf".into(), -4),
            (8, RelocKind::Branch32, "lf".into(), -4),
            (13, RelocKind::Branch32, "gf".into(), -4),
            (20, RelocKind::Pc32, "gf".into(), -4),
            (27, RelocKind::Pc32, "lf".into(), -4),
            (34, RelocKind::Branch32, ".Lx".into(), -4),
        ]
    );
    assert!(m.symbols.iter().any(|s| s.name == ".Lx" && !s.temporary));
    assert!(m.symbols.iter().any(|s| s.name == "Lx" && s.temporary));
}

#[test]
fn macho_atom_boundaries() {
    // A temporary label defined right before a non-temporary one at the same position belongs
    // to the previous atom; the instructions after them to the new one. (The writer refers to
    // `Ltmp` relative to its atom: llvm-mc emits `e9 01000000` with a BRANCH relocation against `a`.)
    let (e, m) = assemble_text("a: nop\nLtmp:\nb: jmp Ltmp\njmp b\n", Target::MacOS);
    assert!(e.is_empty(), "{e:?}");
    let m = m.unwrap();
    assert_eq!(m.sections[0].data, hex("90 e9 00000000 eb f9"));
    assert_eq!(text_relocs(&m), [(2, RelocKind::Branch32, "Ltmp".into(), -4)]);
}

#[test]
fn macho_align() {
    // Apple's `as` reads `.align n` as 2**n and pads code with LLVM's NOPs (at most 10 bytes).
    let (e, m) = assemble_text("nop\n.align 5\nret\n", Target::MacOS);
    assert!(e.is_empty(), "{e:?}");
    let m = m.unwrap();
    let ten = hex("66 2e 0f 1f 84 00 00 00 00 00");
    let mut want = hex("90");
    for _ in 0..3 {
        want.extend_from_slice(&ten);
    }
    want.extend(hex("90 c3"));
    assert_eq!(m.sections[0].data, want);
    assert_eq!(m.sections[0].align, 32);
    let mut em = Emitter::new(Target::MacOS);
    assert_eq!(em.align(536_870_912).unwrap_err(), "invalid alignment value");
    assert_eq!(em.align(32).unwrap_err(), "invalid alignment value");
}

#[test]
fn debug_line_rows() {
    // Like gas: every `.loc` keeps its row (consecutive ones at the same address, a pending one
    // flushed at the current address by the next), rows only start at instructions, line 0 and
    // a trailing `.loc` produce none. Paths are raw bytes.
    let mut em = Emitter::new(Target::Linux);
    em.set_debug_meta(b"/w\xff", "p");
    let f = em.debug_file(b"/x/\xfe.memeasm");
    em.debug_line(f, 1);
    em.label("a").unwrap();
    em.debug_line(f, 2);
    em.insn(Op::Nop, &[]).unwrap();
    em.debug_line(f, 0);
    em.insn(Op::Ret, &[]).unwrap();
    em.debug_line(f, 7);
    em.bytes(&[0x90]);
    em.debug_line(f, 8);
    em.bytes(&[0x90]);
    em.insn(Op::Nop, &[]).unwrap();
    em.debug_line(f, 9);
    let m = em.finish().unwrap();
    let d = m.debug.unwrap();
    let rows: Vec<(u64, u32)> = d.lines.iter().map(|r| (r.offset, r.line)).collect();
    assert_eq!(rows, [(0, 1), (0, 2), (3, 7), (4, 8)]);
    assert_eq!(d.files, [b"/x/\xfe.memeasm".to_vec()]);
    assert_eq!(d.comp_dir, b"/w\xff");
}
