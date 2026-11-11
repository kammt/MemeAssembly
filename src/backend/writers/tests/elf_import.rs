//! Test helper: turns a GNU as ELF relocatable object back into an [`ObjModule`], so the
//! writers can be checked against gas (and other assemblers) without the real assembler.
use crate::backend::module::*;

fn u16_at(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes(b[o..o + 2].try_into().unwrap())
}
fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn u64_at(b: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(b[o..o + 8].try_into().unwrap())
}
fn cstr(b: &[u8], o: usize) -> String {
    let end = b[o..].iter().position(|&c| c == 0).unwrap() + o;
    String::from_utf8(b[o..end].to_vec()).unwrap()
}

struct Sh {
    name: String,
    ty: u32,
    off: usize,
    size: usize,
    info: u32,
    align: u64,
}

pub fn import(b: &[u8]) -> ObjModule {
    assert_eq!(&b[..4], b"\x7fELF");
    assert_eq!(u16_at(b, 16), 1, "not a relocatable object");
    let shoff = u64_at(b, 0x28) as usize;
    let shnum = u16_at(b, 0x3c) as usize;
    let shstrndx = u16_at(b, 0x3e) as usize;
    let raw: Vec<_> = (0..shnum)
        .map(|i| {
            let h = shoff + 64 * i;
            (u32_at(b, h), u32_at(b, h + 4), u64_at(b, h + 0x18) as usize, u64_at(b, h + 0x20) as usize, u32_at(b, h + 0x2c), u64_at(b, h + 0x30))
        })
        .collect();
    let shstr_off = raw[shstrndx].2;
    let shs: Vec<Sh> = raw
        .iter()
        .map(|&(n, ty, off, size, info, align)| Sh { name: cstr(b, shstr_off + n as usize), ty, off, size, info, align })
        .collect();
    let find = |n: &str| shs.iter().position(|s| s.name == n);
    let text_i = find(".text").unwrap();
    let data_i = find(".data").unwrap();
    let to_mod = |shndx: usize| -> Option<usize> {
        if shndx == text_i {
            Some(0)
        } else if shndx == data_i {
            Some(1)
        } else {
            None
        }
    };
    let mut m = ObjModule::default();
    for (i, kind) in [(text_i, SectionKind::Text), (data_i, SectionKind::Data)] {
        let s = &shs[i];
        m.sections.push(Section { kind, data: b[s.off..s.off + s.size].to_vec(), relocs: Vec::new(), align: s.align.max(1) });
    }

    // Symbols. ELF index -> module symbol (None for the null/section symbols).
    let symtab = &shs[find(".symtab").unwrap()];
    let strtab = &shs[find(".strtab").unwrap()];
    let nsyms = symtab.size / 24;
    let mut map: Vec<Option<SymbolId>> = vec![None; nsyms];
    let mut section_of_sym: Vec<Option<usize>> = vec![None; nsyms];
    for i in 1..nsyms {
        let e = symtab.off + 24 * i;
        let name = cstr(b, strtab.off + u32_at(b, e) as usize);
        let info = b[e + 4];
        let shndx = u16_at(b, e + 6) as usize;
        let value = u64_at(b, e + 8);
        if info & 0xf == 3 {
            section_of_sym[i] = to_mod(shndx);
            continue;
        }
        let binding = if info >> 4 == 0 { Binding::Local } else { Binding::Global };
        let definition = if shndx == 0 { None } else { Some((to_mod(shndx).expect("symbol in unknown section"), value)) };
        map[i] = Some(m.symbols.len());
        m.symbols.push(Symbol { name, binding, definition, temporary: false });
    }
    // Temporary labels standing in for "section symbol + offset".
    let temp = |m: &mut ObjModule, sec: usize, off: u64| -> SymbolId {
        let name = format!(".Ltmp_{sec}_{off}");
        if let Some(id) = m.symbol_by_name(&name) {
            return id;
        }
        m.symbols.push(Symbol { name, binding: Binding::Local, definition: Some((sec, off)), temporary: true });
        m.symbols.len() - 1
    };

    let relocs = |name: &str| -> Vec<(u64, u32, usize, i64)> {
        match find(name) {
            None => Vec::new(),
            Some(i) => {
                let s = &shs[i];
                assert_eq!(s.ty, 4);
                (0..s.size / 24)
                    .map(|k| {
                        let e = s.off + 24 * k;
                        let info = u64_at(b, e + 8);
                        (u64_at(b, e), info as u32, (info >> 32) as usize, u64_at(b, e + 16) as i64)
                    })
                    .collect()
            }
        }
    };
    for (si, name) in [(0usize, ".rela.text"), (1, ".rela.data")] {
        if let Some(i) = find(name) {
            assert_eq!(shs[i].info as usize, if si == 0 { text_i } else { data_i });
        }
        for (off, ty, symi, addend) in relocs(name) {
            let kind = match ty {
                1 => RelocKind::Abs64,
                2 => RelocKind::Pc32,
                4 => RelocKind::Branch32,
                10 => RelocKind::Abs32,
                _ => panic!("unsupported relocation type {ty}"),
            };
            let (symbol, addend) = match map[symi] {
                Some(id) => (id, addend),
                None => {
                    let sec = section_of_sym[symi].expect("relocation against unknown section");
                    let len = m.sections[sec].data.len() as i64;
                    // Guess the label the way source code usually references it.
                    let guess = if super::super::is_pcrel(kind) { addend + 4 } else { addend }.clamp(0, len);
                    (temp(&mut m, sec, guess as u64), addend - guess)
                }
            };
            m.sections[si].relocs.push(Reloc { offset: off, kind, symbol, addend });
        }
    }

    if let (Some(stab_i), Some(stabstr_i)) = (find(".stab"), find(".stabstr")) {
        let stab = &shs[stab_i];
        let stabstr = &shs[stabstr_i];
        let rel: std::collections::HashMap<u64, (usize, i64)> =
            relocs(".rela.stab").into_iter().map(|(off, ty, symi, a)| {
                assert_eq!(ty, 10);
                (off, (symi, a))
            }).collect();
        let n = u16_at(b, stab.off + 6) as usize;
        assert_eq!(stab.size, 12 * (n + 1));
        for k in 1..=n {
            let e = stab.off + 12 * k;
            let strx = u32_at(b, e) as usize;
            let value = match rel.get(&(12 * k as u64 + 8)) {
                None => StabValue::Const(u32_at(b, e + 8)),
                Some(&(symi, a)) => match map[symi] {
                    Some(id) => {
                        assert_eq!(a, 0);
                        StabValue::Symbol(id)
                    }
                    None => StabValue::Symbol(temp(&mut m, section_of_sym[symi].unwrap(), a as u64)),
                },
            };
            m.stabs.push(Stab {
                string: if strx == 0 { if b[e + 4] == 100 { Some(String::new()) } else { None } } else { Some(cstr(b, stabstr.off + strx)) },
                n_type: b[e + 4],
                n_other: b[e + 5],
                n_desc: u16_at(b, e + 6),
                value,
            });
        }
    }
    m
}
