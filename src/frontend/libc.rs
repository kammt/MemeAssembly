//! Emulation of the C library behaviour the original compiler relied on.
//!
//! These are *host* properties in the C build (they come from the libc / ABI of the machine the
//! compiler runs on, not from the `-D LINUX/MACOS/WINDOWS` target macro), so they are selected
//! with `cfg!` on the host here as well: `char` signedness, the `rand()` algorithm, the width of
//! `long` and `strerror()` texts. Everything else that the C code decided with `#ifdef` is a
//! runtime [`crate::target::Target`] decision.

use std::time::{SystemTime, UNIX_EPOCH};

/// Value of a byte read through a plain C `char` (signed on x86 and Apple ARM, unsigned on
/// Linux ARM), widened the way C's usual arithmetic conversions widen it into a `uint64_t`.
#[allow(clippy::unnecessary_cast)]
pub fn c_char_value(byte: u8) -> u64 {
    (byte as std::ffi::c_char) as i64 as u64
}

/// Number of bits of a C `long` on the host (32 on Windows, 64 elsewhere).
pub fn c_long_bits() -> u32 {
    std::mem::size_of::<std::ffi::c_long>() as u32 * 8
}

/// C `isspace()` in the "C" locale.
fn is_c_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// Result of `strtol`/`strtoll` with base 10.
pub struct StrToL {
    pub value: i64,
    /// Index of the first unconverted character (`endptr`); 0 if no conversion happened.
    pub end: usize,
    /// Whether `errno` would have been set to `ERANGE`.
    pub overflow: bool,
}

/// Base-10 `strtol` for an integer type with `bits` bits. `s` is treated as a C string
/// (it ends at the first NUL byte, if any).
pub fn strtol_bits(s: &[u8], bits: u32) -> StrToL {
    let s = cstr(s);
    let max: i128 = (1i128 << (bits - 1)) - 1;
    let min: i128 = -(1i128 << (bits - 1));
    let mut i = 0;
    while i < s.len() && is_c_space(s[i]) {
        i += 1;
    }
    let mut negative = false;
    if i < s.len() && (s[i] == b'+' || s[i] == b'-') {
        negative = s[i] == b'-';
        i += 1;
    }
    let digits_start = i;
    let mut acc: i128 = 0;
    let mut overflow = false;
    while i < s.len() && s[i].is_ascii_digit() {
        if !overflow {
            acc = acc * 10 + (s[i] - b'0') as i128;
            if acc > max + 1 {
                overflow = true;
            }
        }
        i += 1;
    }
    if i == digits_start {
        return StrToL { value: 0, end: 0, overflow: false };
    }
    let value = if negative { -acc } else { acc };
    let (value, overflow) = if overflow || value > max {
        (if negative { min } else { max }, true)
    } else if value < min {
        (min, true)
    } else {
        (value, false)
    };
    StrToL { value: value as i64, end: i, overflow }
}

/// `strtoll(s, &end, 10)`.
pub fn strtoll(s: &[u8]) -> StrToL {
    strtol_bits(s, 64)
}

/// `strtol(s, &end, 10)` with the host's `long`.
pub fn strtol(s: &[u8]) -> StrToL {
    strtol_bits(s, c_long_bits())
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

/// `time(NULL)`.
pub fn time_now() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    }
}

/// The `rand()` implementation of the host C library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RandAlgorithm {
    /// glibc `random_r` TYPE_3 additive feedback generator.
    Glibc,
    /// Microsoft CRT linear congruential generator.
    Msvcrt,
    /// macOS / FreeBSD libc Park–Miller "minimal standard" generator.
    Bsd,
}

impl RandAlgorithm {
    pub fn host() -> RandAlgorithm {
        if cfg!(windows) {
            RandAlgorithm::Msvcrt
        } else if cfg!(any(target_os = "macos", target_os = "ios", target_os = "freebsd")) {
            RandAlgorithm::Bsd
        } else {
            RandAlgorithm::Glibc
        }
    }
}

/// State of a C `srand()`/`rand()` pair.
pub struct CRand {
    algorithm: RandAlgorithm,
    glibc: [i32; 34],
    fptr: usize,
    rptr: usize,
    seed: u64,
}

impl CRand {
    /// `srand(seed)`.
    pub fn new(algorithm: RandAlgorithm, seed: u32) -> CRand {
        let mut r = CRand { algorithm, glibc: [0; 34], fptr: 3, rptr: 0, seed: seed as u64 };
        if algorithm == RandAlgorithm::Glibc {
            let seed = if seed == 0 { 1 } else { seed };
            r.glibc[0] = seed as i32;
            let mut word = seed as i64;
            for i in 1..31 {
                let hi = word / 127773;
                let lo = word % 127773;
                word = 16807 * lo - 2836 * hi;
                if word < 0 {
                    word += 2147483647;
                }
                r.glibc[i] = word as i32;
            }
            for _ in 0..310 {
                r.next_glibc();
            }
        }
        r
    }

    fn next_glibc(&mut self) -> i32 {
        const DEG: usize = 31;
        let val = (self.glibc[self.fptr] as u32).wrapping_add(self.glibc[self.rptr] as u32);
        self.glibc[self.fptr] = val as i32;
        self.fptr += 1;
        if self.fptr >= DEG {
            self.fptr = 0;
            self.rptr += 1;
        } else {
            self.rptr += 1;
            if self.rptr >= DEG {
                self.rptr = 0;
            }
        }
        (val >> 1) as i32
    }

    /// `rand()`.
    pub fn rand(&mut self) -> i32 {
        match self.algorithm {
            RandAlgorithm::Glibc => self.next_glibc(),
            RandAlgorithm::Msvcrt => {
                self.seed = (self.seed as u32).wrapping_mul(214013).wrapping_add(2531011) as u64;
                ((self.seed >> 16) & 0x7fff) as i32
            }
            RandAlgorithm::Bsd => {
                if self.seed == 0 {
                    self.seed = 123459876;
                }
                let ctx = self.seed as i64;
                let hi = ctx / 127773;
                let lo = ctx % 127773;
                let mut x = 16807 * lo - 2836 * hi;
                if x < 0 {
                    x += 0x7fffffff;
                }
                self.seed = x as u64;
                (x % 0x8000_0000) as i32
            }
        }
    }
}

/// `strerror(ERANGE)` of the host C library.
pub fn erange_message() -> String {
    if cfg!(windows) {
        "Result too large".to_string()
    } else {
        strip_os_error(&std::io::Error::from_raw_os_error(34).to_string())
    }
}

/// The text `perror()` would print for an I/O error.
pub fn io_error_message(e: &std::io::Error) -> String {
    if cfg!(windows) {
        // Rust reports Win32 messages; the C runtime reports errno texts.
        use std::io::ErrorKind;
        match e.kind() {
            ErrorKind::NotFound => return "No such file or directory".to_string(),
            ErrorKind::PermissionDenied => return "Permission denied".to_string(),
            ErrorKind::AlreadyExists => return "File exists".to_string(),
            ErrorKind::InvalidInput => return "Invalid argument".to_string(),
            _ => {}
        }
    }
    strip_os_error(&e.to_string())
}

fn strip_os_error(s: &str) -> String {
    match s.rfind(" (os error ") {
        Some(p) => s[..p].to_string(),
        None => s.to_string(),
    }
}

/// The C library's `struct tm`: nine `int`s, followed by `tm_gmtoff`/`tm_zone` on glibc and
/// macOS (reserved here so `localtime_r` can write them).
#[repr(C)]
struct Tm {
    tm_sec: i32,
    tm_min: i32,
    tm_hour: i32,
    tm_mday: i32,
    tm_mon: i32,
    tm_year: i32,
    tm_wday: i32,
    tm_yday: i32,
    tm_isdst: i32,
    _extra: [usize; 2],
}

#[cfg(unix)]
extern "C" {
    fn localtime_r(timep: *const i64, result: *mut Tm) -> *mut Tm;
}

#[cfg(windows)]
extern "C" {
    fn _localtime64_s(result: *mut Tm, timep: *const i64) -> i32;
}

/// `localtime()` of `t` as (year, month 1-12, day, hour, minute, second, weekday 0 = Sunday),
/// or `None` if the C library can't convert it.
fn local_time(t: i64) -> Option<(i64, i64, i64, i64, i64, i64, i64)> {
    let mut tm = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        _extra: [0; 2],
    };
    // SAFETY: `tm` is a valid, writable `struct tm` (with room for the glibc/BSD extension
    // fields) and `t` a valid `time_t` (64 bit on all supported targets).
    #[cfg(unix)]
    let ok = unsafe { !localtime_r(&t, &mut tm).is_null() };
    #[cfg(windows)]
    let ok = unsafe { _localtime64_s(&mut tm, &t) == 0 };
    #[cfg(not(any(unix, windows)))]
    let ok = false;
    ok.then(|| {
        (
            tm.tm_year as i64 + 1900,
            tm.tm_mon as i64 + 1,
            tm.tm_mday as i64,
            tm.tm_hour as i64,
            tm.tm_min as i64,
            tm.tm_sec as i64,
            tm.tm_wday as i64,
        )
    })
}

/// `asctime(localtime())` of the current time, including the trailing newline (UTC if the C
/// library can't convert the time).
pub fn asctime_now() -> String {
    let t = time_now();
    if let Some((year, month, d, h, m, s, wday)) = local_time(t) {
        return format_asctime(year, month, d, h, m, s, wday as usize);
    }
    let days = t.div_euclid(86400);
    let secs = t.rem_euclid(86400);
    let (h, m, s) = (secs / 3600, secs % 3600 / 60, secs % 60);
    let wday = (days + 4).rem_euclid(7) as usize;
    // Civil-from-days (Howard Hinnant).
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    format_asctime(year, month, d, h, m, s, wday)
}

fn format_asctime(year: i64, month: i64, d: i64, h: i64, m: i64, s: i64, wday: usize) -> String {
    const WDAY: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MON: [&str; 12] =
        ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    format!(
        "{} {}{:3} {:02}:{:02}:{:02} {}\n",
        WDAY[wday % 7],
        MON[((month - 1).rem_euclid(12)) as usize],
        d,
        h,
        m,
        s,
        year
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rand_vectors() {
        let mut g = CRand::new(RandAlgorithm::Glibc, 1);
        assert_eq!([g.rand(), g.rand(), g.rand()], [1804289383, 846930886, 1681692777]);
        let mut m = CRand::new(RandAlgorithm::Msvcrt, 1);
        assert_eq!([m.rand(), m.rand(), m.rand()], [41, 18467, 6334]);
        let mut b = CRand::new(RandAlgorithm::Bsd, 1);
        assert_eq!([b.rand(), b.rand(), b.rand()], [16807, 282475249, 1622650073]);
    }

    #[test]
    fn strtol_behaviour() {
        let r = strtoll(b" \t-12x");
        assert_eq!((r.value, r.end, r.overflow), (-12, 5, false));
        let r = strtoll(b"99999999999999999999");
        assert_eq!((r.value, r.overflow), (i64::MAX, true));
        let r = strtoll(b"-9223372036854775808");
        assert_eq!((r.value, r.overflow), (i64::MIN, false));
        let r = strtol_bits(b"2147483648", 32);
        assert_eq!((r.value, r.overflow), (i32::MAX as i64, true));
        assert_eq!(strtoll(b"-").end, 0);
        assert_eq!(strtoll(b"").end, 0);
    }
}
