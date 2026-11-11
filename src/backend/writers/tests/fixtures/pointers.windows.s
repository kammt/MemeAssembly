.intel_syntax noprefix
# Absolute pointers in .data (base relocations) next to RIP-relative references.
# Returns 42 when every pointer points where it should, wherever the image was loaded.
.global main

.data
.Ltable: .quad .Lone, .Ltwo, main
.Lone: .byte 40
.Ltwo: .byte 2
.p2align 4
aligned: .quad aligned
.p2align 12
nextpage: .quad .Lone

.text
main:
  mov rcx, [rip + .Ltable]
  movzx eax, byte ptr [rcx]
  mov rcx, [rip + .Ltable + 8]
  movzx edx, byte ptr [rcx]
  add eax, edx
  lea rcx, [rip + main]
  cmp rcx, [rip + .Ltable + 16]
  jne 1f
  lea rcx, [rip + aligned]
  cmp rcx, [rip + aligned]
  jne 1f
  lea rcx, [rip + .Lone]
  cmp rcx, [rip + nextpage]
  jne 1f
  movabs rcx, offset .Lone
  cmp byte ptr [rcx], 40
  jne 1f
  ret
1:
  mov eax, 1
  ret
