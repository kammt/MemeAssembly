.intel_syntax noprefix
# Absolute pointers in __data (rebased by dyld) next to PC-relative references.
# Returns 42 when every pointer points where it should.
.global _main

.data
.Ltable: .quad .Lone, .Ltwo, _main
.Lone: .byte 40
.Ltwo: .byte 2
.p2align 4
aligned: .quad aligned

.text
_main:
  mov rcx, [rip + .Ltable]
  movzx eax, byte ptr [rcx]
  mov rcx, [rip + .Ltable + 8]
  movzx edx, byte ptr [rcx]
  add eax, edx
  lea rcx, [rip + _main]
  cmp rcx, [rip + .Ltable + 16]
  jne 1f
  lea rcx, [rip + aligned]
  cmp rcx, [rip + aligned]
  jne 1f
  ret
1:
  mov eax, 1
  ret
