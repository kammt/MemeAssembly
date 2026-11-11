//! Hand-built modules (encoded with fadec), equivalent to what the translator emits for a
//! tiny "Hi\n" program that returns 7. Used by the tool-independent tests.
use crate::backend::fadec::*;
use crate::backend::module::*;

pub struct Builder {
    pub m: ObjModule,
    fixups: Vec<(usize, String)>,
}

impl Builder {
    pub fn new() -> Builder {
        let mut m = ObjModule::default();
        for kind in [SectionKind::Text, SectionKind::Data] {
            m.sections.push(Section { kind, data: Vec::new(), relocs: Vec::new(), align: 1 });
        }
        Builder { m, fixups: Vec::new() }
    }

    fn sym_id(&mut self, name: &str) -> SymbolId {
        if let Some(id) = self.m.symbol_by_name(name) {
            return id;
        }
        self.m.symbols.push(Symbol {
            name: name.into(),
            binding: Binding::Local,
            definition: None,
            temporary: name.starts_with(".L"),
        });
        self.m.symbols.len() - 1
    }

    pub fn global(&mut self, name: &str) {
        let id = self.sym_id(name);
        self.m.symbols[id].binding = Binding::Global;
    }

    pub fn label(&mut self, sec: usize, name: &str) {
        let id = self.sym_id(name);
        self.m.symbols[id].definition = Some((sec, self.m.sections[sec].data.len() as u64));
    }

    pub fn data(&mut self, bytes: &[u8]) {
        self.m.sections[1].data.extend_from_slice(bytes);
    }

    pub fn ins(&mut self, mnem: u64, ops: &[FeOp]) {
        let b = encode(mnem, ops).unwrap();
        self.m.sections[0].data.extend_from_slice(&b);
    }

    /// Instruction with a `[rip + sym]` operand at `mem_index`, followed by `trailing`
    /// immediate bytes.
    pub fn rip(&mut self, mnem: u64, ops: &[FeOp], mem_index: usize, trailing: usize, sym: &str) {
        let mut o = ops.to_vec();
        o[mem_index] = mem(FE_IP, 0, FE_NOREG, 0);
        let mut b = encode(mnem, &o).unwrap();
        let field = b.len() - trailing - 4;
        b[field..field + 4].fill(0);
        let text = &mut self.m.sections[0];
        let offset = (text.data.len() + field) as u64;
        text.data.extend_from_slice(&b);
        let symbol = self.sym_id(sym);
        self.m.sections[0].relocs.push(Reloc { offset, kind: RelocKind::Pc32, symbol, addend: -4 - trailing as i64 });
    }

    /// `call sym`: resolved here if `sym` is a label of this module, else a relocation.
    pub fn call(&mut self, sym: &str, external: bool) {
        let at = self.m.sections[0].data.len();
        let b = encode_branch(FE_CALL | FE_JMPL, at as u64, at as u64).unwrap();
        self.m.sections[0].data.extend_from_slice(&b);
        let field = self.m.sections[0].data.len() - 4;
        if external {
            self.m.sections[0].data[field..].fill(0);
            let symbol = self.sym_id(sym);
            self.m.sections[0].relocs.push(Reloc { offset: field as u64, kind: RelocKind::Branch32, symbol, addend: -4 });
        } else {
            self.fixups.push((field, sym.into()));
        }
    }

    pub fn finish(mut self) -> ObjModule {
        for (field, sym) in std::mem::take(&mut self.fixups) {
            let (sec, off) = self.m.symbols[self.m.symbol_by_name(&sym).unwrap()].definition.unwrap();
            assert_eq!(sec, 0);
            let rel = off as i64 - (field as i64 + 4);
            self.m.sections[0].data[field..field + 4].copy_from_slice(&(rel as i32).to_le_bytes());
        }
        self.m
    }
}

/// Common shape: `main` prints "Hi\n" through `writechar` and returns 7.
/// With `global_calls`, `writechar` is global and called through `Branch32` relocations, as the
/// emitter does for calls to global functions on ELF/Mach-O.
fn hello(main: &str, global_calls: bool, writechar: impl Fn(&mut Builder)) -> ObjModule {
    let mut b = Builder::new();
    b.global(main);
    if global_calls {
        b.global("writechar");
    }
    b.label(1, ".LCharacter");
    b.data(b"a");
    b.label(1, ".Ltmp64");
    b.data(&[0; 8]);
    b.label(0, main);
    b.ins(FE_SUB64ri, &[FE_SP, 8]); // keep the stack 16-byte aligned for the Win64 calls
    for c in *b"Hi\n" {
        b.rip(FE_MOV8mi, &[0, c as FeOp], 0, 1, ".LCharacter");
        b.call("writechar", global_calls);
    }
    b.ins(FE_ADD64ri, &[FE_SP, 8]);
    b.ins(FE_MOV32ri, &[FE_AX, 7]);
    b.ins(FE_RET, &[]);
    b.label(0, "writechar");
    writechar(&mut b);
    b.finish()
}

fn unix_writechar(b: &mut Builder, syscall_write: i64) {
    for r in [FE_CX, gp(11), FE_AX, FE_DI, FE_SI, FE_DX] {
        b.ins(FE_PUSHr, &[r]);
    }
    b.ins(FE_MOV64ri, &[FE_DX, 1]);
    b.rip(FE_LEA64rm, &[FE_SI, 0], 1, 0, ".LCharacter");
    b.ins(FE_MOV64ri, &[FE_DI, 1]);
    b.ins(FE_MOV64ri, &[FE_AX, syscall_write]);
    b.ins(FE_SYSCALL, &[]);
    for r in [FE_DX, FE_SI, FE_DI, FE_AX, gp(11), FE_CX] {
        b.ins(FE_POPr, &[r]);
    }
    b.ins(FE_RET, &[]);
}

pub fn hello_world_linux() -> ObjModule {
    hello("main", false, |b| unix_writechar(b, 1))
}

/// [`hello_world_linux`] with `-g` information: `main` on line 1 (one row per instruction,
/// lines 1..), `writechar` on line 7.
pub fn hello_world_linux_debug() -> ObjModule {
    let mut m = hello_world_linux();
    let at = |m: &ObjModule, s: &str| m.symbols[m.symbol_by_name(s).unwrap()].definition.unwrap().1;
    let (main, wc, end) = (at(&m, "main"), at(&m, "writechar"), m.sections[0].data.len() as u64);
    let mut lines: Vec<LineRow> = Vec::new();
    for (i, off) in [main, main + 4, main + 11].into_iter().enumerate() {
        lines.push(LineRow { offset: off, file: 0, line: 1 + i as u32 });
    }
    lines.push(LineRow { offset: wc, file: 0, line: 7 });
    m.debug = Some(DebugInfo {
        files: vec!["/src/hello.memeasm".into()],
        comp_dir: "/src".into(),
        producer: "hand-built".into(),
        lines,
        functions: vec![
            DebugFunction { name: "main".into(), file: 0, line: 1, low: main, high: wc },
            DebugFunction { name: "writechar".into(), file: 0, line: 7, low: wc, high: end },
        ],
    });
    m
}

pub fn hello_world_macos() -> ObjModule {
    hello("_main", false, |b| unix_writechar(b, 0x2000004))
}

pub fn hello_world_windows() -> ObjModule {
    hello("main", false, |b| {
        for r in [FE_CX, FE_AX, FE_DX, gp(8), gp(9)] {
            b.ins(FE_PUSHr, &[r]);
        }
        b.ins(FE_SUB64ri, &[FE_SP, 32]);
        b.ins(FE_MOV64ri, &[FE_CX, -11]);
        b.call("GetStdHandle", true);
        b.ins(FE_MOV64rr, &[FE_CX, FE_AX]);
        b.rip(FE_LEA64rm, &[FE_DX, 0], 1, 0, ".LCharacter");
        b.ins(FE_MOV64ri, &[gp(8), 1]);
        b.rip(FE_LEA64rm, &[gp(9), 0], 1, 0, ".Ltmp64");
        b.ins(FE_MOV64mi, &[mem(FE_SP, 0, FE_NOREG, 32), 0]);
        b.call("WriteFile", true);
        b.ins(FE_ADD64ri, &[FE_SP, 32]);
        for r in [gp(9), gp(8), FE_DX, FE_AX, FE_CX] {
            b.ins(FE_POPr, &[r]);
        }
        b.ins(FE_RET, &[]);
    })
}

/// [`hello_world_linux`] calling a global `writechar` through relocations.
pub fn hello_world_linux_global_calls() -> ObjModule {
    hello("main", true, |b| unix_writechar(b, 1))
}

/// [`hello_world_macos`] calling a global `writechar` through relocations.
pub fn hello_world_macos_global_calls() -> ObjModule {
    hello("_main", true, |b| unix_writechar(b, 0x2000004))
}
