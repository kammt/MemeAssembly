.intel_syntax noprefix
# Hand-written relocation edge cases (objects only; not a runnable program).
.global main
.global gdata
.global undefined_but_global

.data
.Lfoo: .quad 1
dlocal: .quad main
  .quad .Lfoo
  .quad dlocal
  .long dlocal
  .quad ext
  .quad gdata + 3
gdata: .byte 2
.Lend:

.text
main:
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
  mov eax, OFFSET dlocal
  call local_fn
  ret
local_fn:
  lea rax, [rip + local_fn]
  ret
