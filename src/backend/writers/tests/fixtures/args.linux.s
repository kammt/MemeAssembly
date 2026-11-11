.intel_syntax noprefix
# Prints argv[argc-1] and envp[0] (each followed by a newline) and returns argc + 40.
# Returns 99 if the stack is not aligned like crt does (rsp % 16 == 8 on entry to main).
.global main

.data
.Lnl: .ascii "\n"
.Lnlptr: .quad .Lnl

.text
main:
  push rbx
  push r12
  push r13
  mov rbx, rdi
  test rsp, 15
  jnz .Lbad
  mov r12, rsi
  mov r13, rdx
  mov rdi, [r12 + rbx*8 - 8]
  call puts_
  mov rdi, [r13]
  call puts_
  lea rax, [rbx + 40]
  pop r13
  pop r12
  pop rbx
  ret
.Lbad:
  mov eax, 99
  pop r13
  pop r12
  pop rbx
  ret

puts_:
  mov rsi, rdi
  xor edx, edx
1: cmp byte ptr [rsi + rdx], 0
  je 2f
  inc rdx
  jmp 1b
2: mov edi, 1
  mov eax, 1
  syscall
  mov rsi, [rip + .Lnlptr]
  mov edx, 1
  mov edi, 1
  mov eax, 1
  syscall
  ret
