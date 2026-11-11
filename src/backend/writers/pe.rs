//! PE32+ console executables, linked directly from the module (no mingw crt).
//!
//! Layout: headers, `.text` (module code, entry stub, import thunks), `.data`, `.idata`,
//! `.reloc`. Like mingw's output the image is relocatable (ASLR: `DYNAMIC_BASE`,
//! `HIGH_ENTROPY_VA`): every absolute 64-bit address gets a `DIR64` base relocation.
//!
//! Entry stub: like mingw's crt it obtains `argc`/`argv`/`envp` from msvcrt's
//! `__getmainargs` (msvcrt.dll is imported by every mingw-built program, including the
//! original compiler's output), calls `main(argc, argv, envp)` with the stack 16-byte aligned
//! and 32 bytes of shadow space, then `ExitProcess(main's return value)`.
//!
//! Undefined symbols may only name the kernel32 functions MemeAssembly programs use: a call
//! to `X` goes through a `jmp [rip + __imp_X]` thunk, `__imp_X` is the IAT slot itself.
use super::super::fadec::*;
use super::super::module::ObjModule;
use super::coff::{
    IMAGE_FILE_MACHINE_AMD64, IMAGE_SCN_CNT_CODE, IMAGE_SCN_CNT_INITIALIZED_DATA, IMAGE_SCN_MEM_EXECUTE,
    IMAGE_SCN_MEM_READ, IMAGE_SCN_MEM_WRITE,
};
use super::{align_up, link, put_u32, Asm, Le};

const IMAGE_BASE: u64 = 0x1_4000_0000;
const SECTION_ALIGN: u64 = 0x1000;
const FILE_ALIGN: u64 = 0x200;

const IMAGE_FILE_EXECUTABLE_IMAGE: u16 = 0x0002;
const IMAGE_FILE_LARGE_ADDRESS_AWARE: u16 = 0x0020;
const IMAGE_DLLCHARACTERISTICS_HIGH_ENTROPY_VA: u16 = 0x0020;
const IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE: u16 = 0x0040;
const IMAGE_DLLCHARACTERISTICS_NX_COMPAT: u16 = 0x0100;
const IMAGE_SCN_MEM_DISCARDABLE: u32 = 0x0200_0000;
const IMAGE_REL_BASED_ABSOLUTE: u16 = 0;
const IMAGE_REL_BASED_DIR64: u16 = 10;
const IMAGE_SUBSYSTEM_WINDOWS_CUI: u16 = 3;
const IMAGE_DIRECTORY_ENTRY_IMPORT: usize = 1;
const IMAGE_DIRECTORY_ENTRY_BASERELOC: usize = 5;
const IMAGE_DIRECTORY_ENTRY_IAT: usize = 12;

const KERNEL32: &str = "KERNEL32.dll";
const MSVCRT: &str = "msvcrt.dll";
/// Functions a module may import (MemeAssembly's Windows I/O) and the ones the stub needs.
const KERNEL32_FUNCTIONS: [&str; 4] = ["ExitProcess", "GetStdHandle", "ReadFile", "WriteFile"];

const DOS_HEADER_SIZE: u64 = 64;
const OPTIONAL_HEADER_SIZE: u64 = 240;

struct Import {
    dll: &'static str,
    name: String,
    /// RVA of the IAT slot.
    slot: u64,
    /// Offset of the `jmp [rip + slot]` thunk in `.text` when the module calls it directly.
    thunk: Option<u64>,
}

/// The entry stub; returns (code, offset of `call main`'s rel32, offsets of the rip-relative
/// fields for `__getmainargs` and `ExitProcess`).
fn entry_stub() -> (Vec<u8>, usize, usize, usize) {
    let mut a = Asm::new();
    // On entry rsp = 8 (mod 16); 72 bytes realign it and hold the shadow space (32), the 5th
    // argument (8), argc/argv/envp (24) and _startupinfo (4).
    a.ins(FE_SUB64ri, &[FE_SP, 72]);
    a.ins(FE_LEA64rm, &[FE_CX, mem(FE_SP, 0, FE_NOREG, 40)]); // &argc
    a.ins(FE_LEA64rm, &[FE_DX, mem(FE_SP, 0, FE_NOREG, 48)]); // &argv
    a.ins(FE_LEA64rm, &[FE_R8, mem(FE_SP, 0, FE_NOREG, 56)]); // &envp
    a.ins(FE_XOR32rr, &[FE_R9, FE_R9]); // no wildcard expansion
    a.ins(FE_MOV32mi, &[mem(FE_SP, 0, FE_NOREG, 64), 0]); // _startupinfo.newmode = 0
    a.ins(FE_LEA64rm, &[FE_AX, mem(FE_SP, 0, FE_NOREG, 64)]);
    a.ins(FE_MOV64mr, &[mem(FE_SP, 0, FE_NOREG, 32), FE_AX]);
    let getmainargs = a.rip_placeholder(FE_CALLm, &[0], 0);
    a.ins(FE_MOV32rm, &[FE_CX, mem(FE_SP, 0, FE_NOREG, 40)]);
    a.ins(FE_MOV64rm, &[FE_DX, mem(FE_SP, 0, FE_NOREG, 48)]);
    a.ins(FE_MOV64rm, &[FE_R8, mem(FE_SP, 0, FE_NOREG, 56)]);
    let main = a.branch_placeholder(FE_CALL);
    a.ins(FE_MOV32rr, &[FE_CX, FE_AX]);
    let exit = a.rip_placeholder(FE_CALLm, &[0], 0);
    a.ins(FE_INT3, &[]);
    (a.code, main, getmainargs, exit)
}

/// `jmp`/`call` targets are 32-bit displacements from the end of the field.
fn rel32(from_field: u64, to: u64) -> Result<[u8; 4], String> {
    let d = to as i128 - (from_field + 4) as i128;
    i32::try_from(d).map(|v| v.to_le_bytes()).map_err(|_| "relocation truncated to fit in entry stub".to_string())
}

/// `.reloc` contents: one block per 4 KiB page with a `DIR64` entry per address (sorted RVAs),
/// each block padded to 4 bytes with `ABSOLUTE` entries. Without addresses there is a single
/// empty block for the first code page, so the section and directory still exist (as with ld).
fn base_relocations(sites: &[u64], first_page: u64) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    loop {
        let page = sites.get(i).map_or(first_page, |&rva| rva & !0xfff);
        let block = out.len();
        out.u32(page as u32);
        out.u32(0); // SizeOfBlock, below
        while let Some(&rva) = sites.get(i).filter(|&&rva| rva & !0xfff == page) {
            out.u16(IMAGE_REL_BASED_DIR64 << 12 | (rva & 0xfff) as u16);
            i += 1;
        }
        if out.len() % 4 != 0 {
            out.u16(IMAGE_REL_BASED_ABSOLUTE << 12);
        }
        let size = (out.len() - block) as u32;
        put_u32(&mut out, block + 4, size);
        if i >= sites.len() {
            return out;
        }
    }
}

pub fn write_executable(m: &ObjModule) -> Result<Vec<u8>, String> {
    // Imports: everything the module references, plus the stub's.
    let mut imports: Vec<Import> = Vec::new();
    let mut direct: Vec<String> = Vec::new();
    for r in m.sections.iter().flat_map(|s| &s.relocs) {
        let s = &m.symbols[r.symbol];
        if s.definition.is_some() {
            continue;
        }
        let (name, is_direct) = match s.name.strip_prefix("__imp_") {
            Some(n) => (n, false),
            None => (s.name.as_str(), true),
        };
        if !KERNEL32_FUNCTIONS.contains(&name) {
            continue; // reported as undefined by the relocation pass
        }
        if !imports.iter().any(|i| i.name == name) {
            imports.push(Import { dll: KERNEL32, name: name.into(), slot: 0, thunk: None });
        }
        if is_direct && !direct.iter().any(|d| d == name) {
            direct.push(name.into());
        }
    }
    if !imports.iter().any(|i| i.name == "ExitProcess") {
        imports.push(Import { dll: KERNEL32, name: "ExitProcess".into(), slot: 0, thunk: None });
    }
    imports.sort_by(|a, b| a.name.cmp(&b.name));
    imports.push(Import { dll: MSVCRT, name: "__getmainargs".into(), slot: 0, thunk: None });
    let dlls = [KERNEL32, MSVCRT];

    // .text: module code, entry stub, thunks.
    let (mut stub, call_main, call_getmainargs, call_exit) = entry_stub();
    let text = &m.sections[0];
    let data = &m.sections[1];
    let stub_off = align_up(text.data.len() as u64, 16);
    let mut thunks = Asm::new();
    let mut thunk_fields = Vec::new();
    for imp in imports.iter_mut() {
        if direct.contains(&imp.name) {
            thunks.code.resize(align_up(thunks.code.len() as u64, 8) as usize, 0xcc);
            let at = thunks.code.len() as u64;
            let field = thunks.rip_placeholder(FE_JMPm, &[0], 0);
            imp.thunk = Some(stub_off + stub.len() as u64 + at);
            thunk_fields.push(field);
        }
    }
    let thunks_off = stub_off + stub.len() as u64;
    let text_size = thunks_off + thunks.code.len() as u64;

    // Section placement. Alignment beyond a page can't be honoured by the loader; the section
    // start is page aligned, which satisfies everything up to 4 KiB.
    let headers_size = align_up(DOS_HEADER_SIZE + 4 + 20 + OPTIONAL_HEADER_SIZE + 40 * 4, FILE_ALIGN);
    let has_data = !data.data.is_empty();
    let text_rva = align_up(headers_size, SECTION_ALIGN);
    let data_rva = align_up(text_rva + text_size, SECTION_ALIGN);
    let idata_rva = if has_data { align_up(data_rva + data.data.len() as u64, SECTION_ALIGN) } else { data_rva };

    // .idata: import directory, lookup tables, IATs (contiguous), hint/name entries, DLL names.
    let ndll = dlls.len() as u64;
    let idt_size = 20 * (ndll + 1);
    let thunk_table_size: u64 = dlls.iter().map(|d| 8 * (imports.iter().filter(|i| i.dll == *d).count() as u64 + 1)).sum();
    let ilt_off = align_up(idt_size, 8);
    let iat_off = ilt_off + thunk_table_size;
    let names_off = iat_off + thunk_table_size;
    let mut idata = vec![0u8; names_off as usize];
    let mut hint_names: Vec<u64> = Vec::new();
    for imp in &imports {
        hint_names.push(idata_rva + idata.len() as u64);
        idata.u16(0); // hint
        idata.extend_from_slice(imp.name.as_bytes());
        idata.u8(0);
        if idata.len() % 2 == 1 {
            idata.u8(0);
        }
    }
    let mut dll_names = Vec::new();
    for d in dlls {
        dll_names.push(idata_rva + idata.len() as u64);
        idata.extend_from_slice(d.as_bytes());
        idata.u8(0);
    }
    let mut table = 0u64; // offset within the lookup/IAT tables
    for (k, d) in dlls.iter().enumerate() {
        let first = table;
        for (i, imp) in imports.iter_mut().enumerate().filter(|(_, i)| i.dll == *d) {
            let hn = hint_names[i];
            for base in [ilt_off, iat_off] {
                let o = (base + table) as usize;
                idata[o..o + 8].copy_from_slice(&hn.to_le_bytes());
            }
            imp.slot = idata_rva + iat_off + table;
            table += 8;
        }
        table += 8; // null terminator
        let e = 20 * k;
        put_u32(&mut idata, e, (idata_rva + ilt_off + first) as u32); // OriginalFirstThunk
        put_u32(&mut idata, e + 12, dll_names[k] as u32); // Name
        put_u32(&mut idata, e + 16, (idata_rva + iat_off + first) as u32); // FirstThunk
    }

    // Link the module.
    let addrs = [IMAGE_BASE + text_rva, IMAGE_BASE + data_rva];
    let resolve = |name: &str| -> Option<u64> {
        let (n, via_iat) = match name.strip_prefix("__imp_") {
            Some(n) => (n, true),
            None => (name, false),
        };
        let imp = imports.iter().find(|i| i.dll == KERNEL32 && i.name == n)?;
        if via_iat {
            Some(IMAGE_BASE + imp.slot)
        } else {
            imp.thunk.map(|t| IMAGE_BASE + text_rva + t)
        }
    };
    // (32-bit absolute relocations fail here: the image base is above 4 GiB, as with ld.
    // They couldn't be rebased either.)
    let (linked, main_addr) = link::link_program(m, "main", link::NO_MAIN, &addrs, &resolve)?;

    // Patch the stub and thunks.
    let slot_of = |n: &str| imports.iter().find(|i| i.name == n).expect("stub import").slot;
    let stub_rva = text_rva + stub_off;
    let main_rva = main_addr - IMAGE_BASE;
    let patch = |code: &mut Vec<u8>, base_rva: u64, field: usize, target_rva: u64| -> Result<(), String> {
        let b = rel32(base_rva + field as u64, target_rva)?;
        code[field..field + 4].copy_from_slice(&b);
        Ok(())
    };
    patch(&mut stub, stub_rva, call_main, main_rva)?;
    patch(&mut stub, stub_rva, call_getmainargs, slot_of("__getmainargs"))?;
    patch(&mut stub, stub_rva, call_exit, slot_of("ExitProcess"))?;
    let thunks_rva = text_rva + thunks_off;
    let mut k = 0;
    for imp in &imports {
        if imp.thunk.is_some() {
            patch(&mut thunks.code, thunks_rva, thunk_fields[k], imp.slot)?;
            k += 1;
        }
    }
    let mut text_bytes = linked.sections[0].clone();
    text_bytes.resize(stub_off as usize, 0xcc);
    text_bytes.extend_from_slice(&stub);
    text_bytes.extend_from_slice(&thunks.code);

    // Sections: (name, rva, contents, characteristics).
    let mut sections: Vec<(&str, u64, Vec<u8>, u32)> =
        vec![(".text", text_rva, text_bytes, IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ)];
    let rw_data = IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_WRITE;
    if has_data {
        sections.push((".data", data_rva, linked.sections[1].clone(), rw_data));
    }
    let idata_size = idata.len() as u64;
    sections.push((".idata", idata_rva, idata, rw_data));
    let reloc_rva = align_up(idata_rva + idata_size, SECTION_ALIGN);
    let mut sites: Vec<u64> = linked.abs64_sites.iter().map(|&(s, off)| [text_rva, data_rva][s] + off).collect();
    sites.sort_unstable();
    let reloc = base_relocations(&sites, text_rva);
    let reloc_size = reloc.len() as u64;
    sections.push((".reloc", reloc_rva, reloc, IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ | IMAGE_SCN_MEM_DISCARDABLE));
    let image_size = align_up(reloc_rva + reloc_size, SECTION_ALIGN);

    // Headers.
    let mut out = Vec::new();
    out.extend_from_slice(b"MZ");
    out.pad_to(0x3c, 0);
    out.u32(DOS_HEADER_SIZE as u32); // e_lfanew: no DOS stub program
    out.extend_from_slice(b"PE\0\0");
    out.u16(IMAGE_FILE_MACHINE_AMD64);
    out.u16(sections.len() as u16);
    out.u32(0); // TimeDateStamp
    out.u32(0);
    out.u32(0);
    out.u16(OPTIONAL_HEADER_SIZE as u16);
    out.u16(IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_LARGE_ADDRESS_AWARE);
    let raw_size = |len: usize| align_up(len as u64, FILE_ALIGN);
    let code_size = raw_size(sections[0].2.len());
    let init_data: u64 = sections[1..].iter().map(|s| raw_size(s.2.len())).sum();
    out.u16(0x20b); // PE32+
    out.u8(0); // linker version
    out.u8(0);
    out.u32(code_size as u32);
    out.u32(init_data as u32);
    out.u32(0); // SizeOfUninitializedData
    out.u32(stub_rva as u32); // AddressOfEntryPoint
    out.u32(text_rva as u32); // BaseOfCode
    out.u64(IMAGE_BASE);
    out.u32(SECTION_ALIGN as u32);
    out.u32(FILE_ALIGN as u32);
    out.u16(4); // OS version
    out.u16(0);
    out.u16(0); // image version
    out.u16(0);
    out.u16(5); // subsystem version 5.2 (as mingw ld)
    out.u16(2);
    out.u32(0); // Win32VersionValue
    out.u32(image_size as u32);
    out.u32(headers_size as u32);
    out.u32(0); // CheckSum (only checked for drivers and system DLLs)
    out.u16(IMAGE_SUBSYSTEM_WINDOWS_CUI);
    out.u16(IMAGE_DLLCHARACTERISTICS_HIGH_ENTROPY_VA | IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE | IMAGE_DLLCHARACTERISTICS_NX_COMPAT);
    out.u64(0x20_0000); // stack reserve
    out.u64(0x1000); // stack commit
    out.u64(0x10_0000); // heap reserve
    out.u64(0x1000); // heap commit
    out.u32(0); // LoaderFlags
    out.u32(16); // NumberOfRvaAndSizes
    let mut dirs = [(0u32, 0u32); 16];
    dirs[IMAGE_DIRECTORY_ENTRY_IMPORT] = ((idata_rva) as u32, idt_size as u32);
    dirs[IMAGE_DIRECTORY_ENTRY_BASERELOC] = (reloc_rva as u32, reloc_size as u32);
    dirs[IMAGE_DIRECTORY_ENTRY_IAT] = ((idata_rva + iat_off) as u32, thunk_table_size as u32);
    for (rva, size) in dirs {
        out.u32(rva);
        out.u32(size);
    }
    let mut file_pos = headers_size;
    let mut placed = Vec::new();
    for (name, rva, bytes, chars) in &sections {
        out.name::<8>(name);
        out.u32(bytes.len() as u32); // VirtualSize
        out.u32(*rva as u32);
        let raw = raw_size(bytes.len());
        out.u32(raw as u32);
        out.u32(file_pos as u32);
        out.u32(0); // relocations
        out.u32(0); // line numbers
        out.u16(0);
        out.u16(0);
        out.u32(*chars);
        placed.push(file_pos);
        file_pos += raw;
    }
    for (i, (_, _, bytes, _)) in sections.iter().enumerate() {
        out.pad_to(placed[i], 0);
        out.extend_from_slice(bytes);
    }
    out.pad_to(file_pos, 0);
    Ok(out)
}
