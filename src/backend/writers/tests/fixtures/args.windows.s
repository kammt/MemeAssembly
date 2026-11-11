# Entry stub check for PE executables: writes argv[2] to stdout and returns
# argc + 40 + (rsp & 15 at entry); with two arguments and a correctly aligned call that is 51.
.intel_syntax noprefix
.extern GetStdHandle
.extern WriteFile
.text
.global main
main:
    push rbx
    push rsi
    push rdi
    push r12
    sub rsp, 40
    lea r12, [rsp + 72]
    and r12d, 15
    mov ebx, ecx
    add ebx, r12d
    mov rsi, [rdx + 16]
    xor edi, edi
1:  cmp byte ptr [rsi + rdi], 0
    je 2f
    inc rdi
    jmp 1b
2:  mov ecx, -11
    call GetStdHandle
    mov rcx, rax
    mov rdx, rsi
    mov r8, rdi
    lea r9, [rip + written]
    mov qword ptr [rsp + 32], 0
    call WriteFile
    lea eax, [rbx + 40]
    add rsp, 40
    pop r12
    pop rdi
    pop rsi
    pop rbx
    ret
.data
written: .quad 0
