.intel_syntax noprefix
# Relocation edge cases for Mach-O (objects only; not a runnable program). Like
# edge.linux.s minus the 32-bit absolute references, which x86-64 Mach-O can't express.
.global _main
.global gdata
.global undefined_but_global

.data
.Lfoo: .quad 1
dlocal: .quad _main
  .quad .Lfoo
  .quad dlocal
  .quad ext
  .quad ext + 5
  .quad gdata + 3
  .quad .Lfoo + 6
gdata: .byte 2
.Lend:

.text
_main:
  call ext
  call dlocal
  call gdata
  call .Lfoo
  lea rax, [rip + dlocal]
  lea rax, [rip + ext]
  lea rax, [rip + ext + 16]
  lea rax, [rip + gdata]
  lea rax, [rip + .Lfoo + 3]
  lea rax, [rip + .Lend]
  mov byte ptr [rip + .Lfoo], 3
  mov word ptr [rip + .Lfoo], 3
  mov dword ptr [rip + .Lfoo], 3
  mov qword ptr [rip + .Lfoo], 3
  mov byte ptr [rip + gdata], 3
  mov dword ptr [rip + ext], 3
  jmp ext
  jmp dlocal
  call local_fn
  ret
local_fn:
  lea rax, [rip + local_fn]
  ret
