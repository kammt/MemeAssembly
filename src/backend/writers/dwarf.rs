//! DWARF 5 debug sections for `-g` (ELF only): `.debug_info`, `.debug_abbrev` and
//! `.debug_line`. Strings are stored inline (`DW_FORM_string`), so there are no string sections.
//!
//! The sections are produced format-neutrally together with the places that need relocation:
//! addresses inside `.text` (64-bit) and offsets into other debug sections (32-bit, DWARF32).
//! Relocatable output turns them into relocations against section symbols (like gas);
//! executables write the final values.
use super::super::module::DebugInfo;
use super::{uleb, Le};

/// What a relocated field refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelTarget {
    /// `.text` + addend (64-bit address).
    Text,
    /// Offset into the named debug section + addend (32-bit section offset).
    Section(&'static str),
}

#[derive(Debug, Clone)]
pub struct DebugReloc {
    pub offset: u64,
    pub target: RelTarget,
    pub addend: i64,
}

#[derive(Debug, Clone)]
pub struct DebugSection {
    pub name: &'static str,
    /// Relocated fields are left zero.
    pub data: Vec<u8>,
    pub relocs: Vec<DebugReloc>,
}

impl DebugSection {
    /// Contents with every relocation resolved, given the final address of `.text`.
    pub fn resolved(&self, text_addr: u64) -> Vec<u8> {
        let mut d = self.data.clone();
        for r in &self.relocs {
            let o = r.offset as usize;
            match r.target {
                RelTarget::Text => {
                    d[o..o + 8].copy_from_slice(&(text_addr as i64 + r.addend).to_le_bytes())
                }
                RelTarget::Section(_) => d[o..o + 4].copy_from_slice(&(r.addend as u32).to_le_bytes()),
            }
        }
        d
    }
}

const DW_TAG_COMPILE_UNIT: u64 = 0x11;
const DW_TAG_SUBPROGRAM: u64 = 0x2e;
const DW_AT_NAME: u64 = 0x03;
const DW_AT_STMT_LIST: u64 = 0x10;
const DW_AT_LOW_PC: u64 = 0x11;
const DW_AT_HIGH_PC: u64 = 0x12;
const DW_AT_LANGUAGE: u64 = 0x13;
const DW_AT_COMP_DIR: u64 = 0x1b;
const DW_AT_PRODUCER: u64 = 0x25;
const DW_AT_DECL_FILE: u64 = 0x3a;
const DW_AT_DECL_LINE: u64 = 0x3b;
const DW_AT_EXTERNAL: u64 = 0x3f;
const DW_FORM_ADDR: u64 = 0x01;
const DW_FORM_DATA2: u64 = 0x05;
const DW_FORM_DATA8: u64 = 0x07;
const DW_FORM_STRING: u64 = 0x08;
const DW_FORM_UDATA: u64 = 0x0f;
const DW_FORM_SEC_OFFSET: u64 = 0x17;
const DW_FORM_FLAG_PRESENT: u64 = 0x19;
const DW_LANG_MIPS_ASSEMBLER: u16 = 0x8001;
const DW_UT_COMPILE: u8 = 1;
const DW_LNCT_PATH: u64 = 1;
const DW_LNCT_DIRECTORY_INDEX: u64 = 2;
const DW_LNS_COPY: u8 = 1;
const DW_LNS_ADVANCE_PC: u8 = 2;
const DW_LNS_ADVANCE_LINE: u8 = 3;
const DW_LNS_SET_FILE: u8 = 4;
const DW_LNE_END_SEQUENCE: u8 = 1;
const DW_LNE_SET_ADDRESS: u8 = 2;

fn sleb(b: &mut Vec<u8>, mut v: i64) {
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        let done = (v == 0 && byte & 0x40 == 0) || (v == -1 && byte & 0x40 != 0);
        b.push(if done { byte } else { byte | 0x80 });
        if done {
            return;
        }
    }
}

/// A NUL-terminated inline string (`DW_FORM_string`).
fn string(b: &mut Vec<u8>, s: &[u8]) {
    b.extend_from_slice(s);
    b.push(0);
}

/// Splits an absolute path into (directory, file name).
fn split_path(p: &[u8]) -> (&[u8], &[u8]) {
    match p.iter().rposition(|&c| c == b'/') {
        Some(0) => (b"/", &p[1..]),
        Some(i) => (&p[..i], &p[i + 1..]),
        None => (b"", p),
    }
}

/// Builds the DWARF sections describing a `.text` of `text_size` bytes.
pub fn build(d: &DebugInfo, text_size: u64) -> Vec<DebugSection> {
    // ---- .debug_line ----
    // Directory 0 is the compilation directory; file 0 is the primary source file and is
    // repeated as file 1 (like gcc/gas), so `DebugInfo::files[i]` is DWARF file `i + 1`.
    let mut dirs: Vec<Vec<u8>> = vec![d.comp_dir.clone()];
    let mut file_entries: Vec<(Vec<u8>, u64)> = Vec::new();
    let primary = d.files.first().cloned().unwrap_or_default();
    for f in std::iter::once(&primary).chain(d.files.iter()) {
        let (dir, name) = split_path(f);
        let di = match dirs.iter().position(|x| x == dir) {
            Some(i) => i,
            None => {
                dirs.push(dir.to_vec());
                dirs.len() - 1
            }
        };
        file_entries.push((name.to_vec(), di as u64));
    }

    let mut line = Vec::new();
    let mut line_relocs = Vec::new();
    line.u32(0); // unit_length, patched
    line.u16(5);
    line.u8(8); // address_size
    line.u8(0); // segment_selector_size
    let header_length_at = line.len();
    line.u32(0); // header_length, patched
    line.u8(1); // minimum_instruction_length
    line.u8(1); // maximum_operations_per_instruction
    line.u8(1); // default_is_stmt
    line.u8((-5i8) as u8); // line_base
    line.u8(14); // line_range
    line.u8(13); // opcode_base
    line.extend_from_slice(&[0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 1]);
    line.u8(1); // directory_entry_format_count
    uleb(&mut line, DW_LNCT_PATH);
    uleb(&mut line, DW_FORM_STRING);
    uleb(&mut line, dirs.len() as u64);
    for dir in &dirs {
        string(&mut line, dir);
    }
    line.u8(2); // file_name_entry_format_count
    uleb(&mut line, DW_LNCT_PATH);
    uleb(&mut line, DW_FORM_STRING);
    uleb(&mut line, DW_LNCT_DIRECTORY_INDEX);
    uleb(&mut line, DW_FORM_UDATA);
    uleb(&mut line, file_entries.len() as u64);
    for (name, di) in &file_entries {
        string(&mut line, name);
        uleb(&mut line, *di);
    }
    let header_length = line.len() - header_length_at - 4;
    line[header_length_at..header_length_at + 4].copy_from_slice(&(header_length as u32).to_le_bytes());

    // Line number program: one sequence from the first row to the end of .text.
    if let Some(first) = d.lines.first() {
        line.u8(0);
        uleb(&mut line, 9);
        line.u8(DW_LNE_SET_ADDRESS);
        line_relocs.push(DebugReloc { offset: line.len() as u64, target: RelTarget::Text, addend: first.offset as i64 });
        line.u64(0);
        let (mut addr, mut file, mut lineno) = (first.offset, 1u64, 1i64);
        for row in &d.lines {
            let f = row.file as u64 + 1;
            if f != file {
                line.u8(DW_LNS_SET_FILE);
                uleb(&mut line, f);
                file = f;
            }
            if row.line as i64 != lineno {
                line.u8(DW_LNS_ADVANCE_LINE);
                sleb(&mut line, row.line as i64 - lineno);
                lineno = row.line as i64;
            }
            if row.offset > addr {
                line.u8(DW_LNS_ADVANCE_PC);
                uleb(&mut line, row.offset - addr);
                addr = row.offset;
            }
            line.u8(DW_LNS_COPY);
        }
        if text_size > addr {
            line.u8(DW_LNS_ADVANCE_PC);
            uleb(&mut line, text_size - addr);
        }
        line.u8(0);
        uleb(&mut line, 1);
        line.u8(DW_LNE_END_SEQUENCE);
    }
    let unit_length = line.len() as u32 - 4;
    line[..4].copy_from_slice(&unit_length.to_le_bytes());

    // ---- .debug_abbrev ----
    let mut abbrev = Vec::new();
    let mut decl = |code: u64, tag: u64, children: bool, attrs: &[(u64, u64)]| {
        uleb(&mut abbrev, code);
        uleb(&mut abbrev, tag);
        abbrev.u8(children as u8);
        for &(a, f) in attrs {
            uleb(&mut abbrev, a);
            uleb(&mut abbrev, f);
        }
        abbrev.extend_from_slice(&[0, 0]);
    };
    let has_functions = !d.functions.is_empty();
    decl(
        1,
        DW_TAG_COMPILE_UNIT,
        has_functions,
        &[
            (DW_AT_STMT_LIST, DW_FORM_SEC_OFFSET),
            (DW_AT_LOW_PC, DW_FORM_ADDR),
            (DW_AT_HIGH_PC, DW_FORM_DATA8),
            (DW_AT_NAME, DW_FORM_STRING),
            (DW_AT_COMP_DIR, DW_FORM_STRING),
            (DW_AT_PRODUCER, DW_FORM_STRING),
            (DW_AT_LANGUAGE, DW_FORM_DATA2),
        ],
    );
    decl(
        2,
        DW_TAG_SUBPROGRAM,
        false,
        &[
            (DW_AT_EXTERNAL, DW_FORM_FLAG_PRESENT),
            (DW_AT_NAME, DW_FORM_STRING),
            (DW_AT_DECL_FILE, DW_FORM_UDATA),
            (DW_AT_DECL_LINE, DW_FORM_UDATA),
            (DW_AT_LOW_PC, DW_FORM_ADDR),
            (DW_AT_HIGH_PC, DW_FORM_DATA8),
        ],
    );
    abbrev.u8(0);

    // ---- .debug_info ----
    let mut info = Vec::new();
    let mut info_relocs = Vec::new();
    // Offset 0 of another debug section.
    let sec = |info: &mut Vec<u8>, relocs: &mut Vec<DebugReloc>, name: &'static str| {
        relocs.push(DebugReloc { offset: info.len() as u64, target: RelTarget::Section(name), addend: 0 });
        info.u32(0);
    };
    info.u32(0); // unit_length, patched
    info.u16(5);
    info.u8(DW_UT_COMPILE);
    info.u8(8);
    sec(&mut info, &mut info_relocs, ".debug_abbrev");
    uleb(&mut info, 1);
    sec(&mut info, &mut info_relocs, ".debug_line");
    info_relocs.push(DebugReloc { offset: info.len() as u64, target: RelTarget::Text, addend: 0 });
    info.u64(0);
    info.u64(text_size);
    string(&mut info, &primary);
    string(&mut info, &d.comp_dir);
    string(&mut info, d.producer.as_bytes());
    info.u16(DW_LANG_MIPS_ASSEMBLER);
    for f in &d.functions {
        uleb(&mut info, 2);
        string(&mut info, f.name.as_bytes());
        uleb(&mut info, f.file as u64 + 1);
        uleb(&mut info, f.line as u64);
        info_relocs.push(DebugReloc { offset: info.len() as u64, target: RelTarget::Text, addend: f.low as i64 });
        info.u64(0);
        info.u64(f.high.saturating_sub(f.low));
    }
    if has_functions {
        info.u8(0);
    }
    let unit_length = info.len() as u32 - 4;
    info[..4].copy_from_slice(&unit_length.to_le_bytes());

    vec![
        DebugSection { name: ".debug_info", data: info, relocs: info_relocs },
        DebugSection { name: ".debug_abbrev", data: abbrev, relocs: Vec::new() },
        DebugSection { name: ".debug_line", data: line, relocs: line_relocs },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leb128() {
        let mut b = Vec::new();
        uleb(&mut b, 624485);
        assert_eq!(b, [0xe5, 0x8e, 0x26]);
        b.clear();
        sleb(&mut b, -123456);
        assert_eq!(b, [0xc0, 0xbb, 0x78]);
        b.clear();
        sleb(&mut b, 63);
        sleb(&mut b, 64);
        sleb(&mut b, -64);
        sleb(&mut b, -65);
        assert_eq!(b, [0x3f, 0xc0, 0x00, 0x40, 0xbf, 0x7f]);
    }
}
