//! Helpers for the C-string handling the original compiler relied on, decimal parsing and
//! I/O error texts.

use std::num::IntErrorKind;

/// A decimal number parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decimal {
    /// The value, saturated to `i64::MIN`/`i64::MAX` if it does not fit into 64 bits.
    pub value: i64,
    /// Whether the number does not fit into 64 bits.
    pub overflow: bool,
}

/// Parses a decimal number: optional leading ASCII whitespace, an optional sign and digits,
/// nothing else. The empty string counts as 0 (bully mode can shorten a number to nothing).
pub fn parse_decimal(s: &[u8]) -> Option<Decimal> {
    if s.is_empty() {
        return Some(Decimal { value: 0, overflow: false });
    }
    let s = s.trim_ascii_start();
    let digits = s.strip_prefix(b"-").or_else(|| s.strip_prefix(b"+")).unwrap_or(s);
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let text = std::str::from_utf8(s).ok()?;
    match text.parse::<i64>() {
        Ok(value) => Some(Decimal { value, overflow: false }),
        Err(e) => match e.kind() {
            IntErrorKind::PosOverflow => Some(Decimal { value: i64::MAX, overflow: true }),
            IntErrorKind::NegOverflow => Some(Decimal { value: i64::MIN, overflow: true }),
            _ => None,
        },
    }
}

/// The value of a decimal number parameter, 0 if `s` is not one (e.g. only a sign is left
/// after bully mode shortened it).
pub fn decimal_value(s: &[u8]) -> i64 {
    parse_decimal(s).map_or(0, |d| d.value)
}

/// The part of `s` before the first NUL byte (what C string functions see).
pub fn cstr(s: &[u8]) -> &[u8] {
    match s.iter().position(|&b| b == 0) {
        Some(p) => &s[..p],
        None => s,
    }
}

/// `strncmp(a, b, n) == 0` for C strings.
pub fn strncmp_eq(a: &[u8], b: &[u8], n: usize) -> bool {
    for i in 0..n {
        let ca = a.get(i).copied().unwrap_or(0);
        let cb = b.get(i).copied().unwrap_or(0);
        if ca != cb {
            return false;
        }
        if ca == 0 {
            return true;
        }
    }
    true
}

/// `strtok_r` over an owned copy of a C string. Tokens are returned as index ranges; the save
/// pointer is exposed because the original parser peeks at (and advances) it directly.
pub struct StrTok {
    buf: Vec<u8>,
    pub save: usize,
}

impl StrTok {
    pub fn new(s: &[u8]) -> StrTok {
        StrTok { buf: cstr(s).to_vec(), save: 0 }
    }

    pub fn next_token(&mut self, delims: &[u8]) -> Option<&[u8]> {
        let len = self.buf.len();
        let mut s = self.save.min(len);
        while s < len && delims.contains(&self.buf[s]) {
            s += 1;
        }
        if s == len {
            self.save = len;
            return None;
        }
        let mut e = s;
        while e < len && !delims.contains(&self.buf[e]) {
            e += 1;
        }
        self.save = if e == len { len } else { e + 1 };
        Some(&self.buf[s..e])
    }

    /// The C string the save pointer currently points to.
    pub fn rest(&self) -> &[u8] {
        &self.buf[self.save.min(self.buf.len())..]
    }
}

/// The text of an I/O error without Rust's ` (os error N)` suffix.
pub fn io_error_message(e: &std::io::Error) -> String {
    strip_os_error(&e.to_string())
}

/// Removes Rust's ` (os error N)` suffix, leaving a perror-style `context: message`.
pub(super) fn strip_os_error(s: &str) -> String {
    match s.rfind(" (os error ") {
        Some(p) => s[..p].to_string(),
        None => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimals() {
        let d = |s: &[u8]| parse_decimal(s).map(|d| (d.value, d.overflow));
        assert_eq!(d(b"42"), Some((42, false)));
        assert_eq!(d(b" \t-12"), Some((-12, false)));
        assert_eq!(d(b"+7"), Some((7, false)));
        assert_eq!(d(b""), Some((0, false)));
        assert_eq!(d(b"-9223372036854775808"), Some((i64::MIN, false)));
        assert_eq!(d(b"99999999999999999999"), Some((i64::MAX, true)));
        assert_eq!(d(b"-99999999999999999999"), Some((i64::MIN, true)));
        for s in [&b"-"[..], b"+", b" ", b"12x", b"99999999999999999999x", b"1 2", b"0x10", b"--1", b"1\0"] {
            assert_eq!(d(s), None, "{:?}", String::from_utf8_lossy(s));
        }
        assert_eq!(decimal_value(b"-"), 0);
        assert_eq!(decimal_value(b"123"), 123);
    }
}
