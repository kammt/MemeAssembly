//! Hand-built modules (encoded with the emitter's x86 encoder, but laid out here), equivalent
//! to what the translator emits for a tiny "Hi\n" program that returns 7. Used by the
//! tool-independent tests.
use crate::backend::emitter::x86::{self, BranchKind};
use crate::backend::emitter::{Op, Operand, Size};
use crate::backend::module::*;
use crate::backend::writers::{mem, reg, rip};

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

    /// `op ops`; a `[rip + sym]` operand becomes a `Pc32` relocation against `sym`.
    pub fn ins(&mut self, op: Op, ops: &[Operand]) {
        let e = x86::encode(op, ops).unwrap();
        let text = &mut self.m.sections[0];
        let at = text.data.len();
        text.data.extend_from_slice(&e.bytes);
        if let Some((field, sym, disp)) = e.rip {
            let trailing = e.bytes.len() - field - 4;
            let symbol = self.sym_id(&sym);
            let reloc = Reloc { offset: (at + field) as u64, kind: RelocKind::Pc32, symbol, addend: disp - 4 - trailing as i64 };
            self.m.sections[0].relocs.push(reloc);
        }
    }

    /// Raw code bytes.
    pub fn code(&mut self, bytes: &[u8]) {
        self.m.sections[0].data.extend_from_slice(bytes);
    }

    /// `call sym`: resolved here if `sym` is a label of this module, else a relocation.
    pub fn call(&mut self, sym: &str, external: bool) {
        self.m.sections[0].data.extend_from_slice(&x86::branch(BranchKind::Call, true, 0));
        let field = self.m.sections[0].data.len() - 4;
        if external {
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
    b.ins(Op::Sub, &[reg("rsp"), Operand::Imm(8)]); // keep the stack 16-byte aligned for the Win64 calls
    for c in *b"Hi\n" {
        b.ins(Op::Mov, &[rip(Some(Size::Byte), ".LCharacter"), Operand::Imm(c.into())]);
        b.call("writechar", global_calls);
    }
    b.ins(Op::Add, &[reg("rsp"), Operand::Imm(8)]);
    b.ins(Op::Mov, &[reg("eax"), Operand::Imm(7)]);
    b.ins(Op::Ret, &[]);
    b.label(0, "writechar");
    writechar(&mut b);
    b.finish()
}

fn unix_writechar(b: &mut Builder, syscall_write: i64) {
    for r in ["rcx", "r11", "rax", "rdi", "rsi", "rdx"] {
        b.ins(Op::Push, &[reg(r)]);
    }
    b.ins(Op::Mov, &[reg("rdx"), Operand::Imm(1)]);
    b.ins(Op::Lea, &[reg("rsi"), rip(None, ".LCharacter")]);
    b.ins(Op::Mov, &[reg("rdi"), Operand::Imm(1)]);
    b.ins(Op::Mov, &[reg("rax"), Operand::Imm(syscall_write)]);
    b.ins(Op::Syscall, &[]);
    for r in ["rdx", "rsi", "rdi", "rax", "r11", "rcx"] {
        b.ins(Op::Pop, &[reg(r)]);
    }
    b.ins(Op::Ret, &[]);
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
        for r in ["rcx", "rax", "rdx", "r8", "r9"] {
            b.ins(Op::Push, &[reg(r)]);
        }
        b.ins(Op::Sub, &[reg("rsp"), Operand::Imm(32)]);
        b.ins(Op::Mov, &[reg("rcx"), Operand::Imm(-11)]);
        b.call("GetStdHandle", true);
        b.ins(Op::Mov, &[reg("rcx"), reg("rax")]);
        b.ins(Op::Lea, &[reg("rdx"), rip(None, ".LCharacter")]);
        b.ins(Op::Mov, &[reg("r8"), Operand::Imm(1)]);
        b.ins(Op::Lea, &[reg("r9"), rip(None, ".Ltmp64")]);
        b.ins(Op::Mov, &[mem(Some(Size::Qword), "rsp", 32), Operand::Imm(0)]);
        b.call("WriteFile", true);
        b.ins(Op::Add, &[reg("rsp"), Operand::Imm(32)]);
        for r in ["r9", "r8", "rdx", "rax", "rcx"] {
            b.ins(Op::Pop, &[reg(r)]);
        }
        b.ins(Op::Ret, &[]);
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
