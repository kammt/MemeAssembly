.intel_syntax noprefix
.global _main

.data
	.LCharacter: .ascii "a"
	.Ltmp64: .byte 0, 0, 0, 0, 0, 0, 0, 0
	.LsigStruct:
		.Lsa_handler: .quad 0
		.Lsa_handler_2: .quad 0
		.quad 0, 0



.text
	

.Ltext0:
killParent:
    mov rax, 0x2000027
    syscall

    mov rdi, rax
    mov rsi, 9
    mov rax, 0x2000025
    syscall

    mov rdi, 0
    mov rax, 60
    syscall
    ret

_main:
push rax
    push rdi
    push rsi
    push rdx
    push r10
    push rcx
    push r11
    
    lea rax, [rip + killParent]
    mov [rip + .Lsa_handler], rax
    mov [rip + .Lsa_handler_2], rax

    mov rax, 0x200002E
    mov rdi, 2
    lea rsi, [rip + .LsigStruct]
    xor rdx, rdx
    mov r10, 8
    syscall
    
    pop r11
    pop rcx
    pop r10
    pop rdx
    pop rsi
    pop rdi
    pop rax

	mov rdi, rsi
	add rdi, 0x8
	mov rdi, [rdi]
	push rbp
	mov rbp, rsp
	sub rsp, 0x2710
	mov rdx, rsp
	xor rax, rax
	xor r8b, r8b
	.Luuuuuuuuauuuuuauauaa:
	cmp rax, 0x2710
	jg .LraxWins_0
	jl .L0x2710Wins_0
	jmp .Laaaaaaaaaaauaaaaaaaaaaaauu
	.LraxWins_0:
	.L0x2710Wins_0:
	mov [rdx], r8b
	add rdx, 1
	add rax, 1
	jmp .Luuuuuuuuauuuuuauauaa
	.Laaaaaaaaaaauaaaaaaaaaaaauu:
	mov rdx, rsp
	push r12
	push r13
	push r14
	push r15
	push rbx
	.Luauauauauaaaa:
	mov al, [rdi]
	cmp al, 0x0
	je .LSamePicture_0
	cmp al, '+'
	jg .LalWins_0
	jl .L'+'Wins_0
	mov r8b, [rdx]
	add r8b, 1
	mov [rdx], r8b
	add rdi, 1
	jmp .Laaaaaaaauauauaaa
	.L'+'Wins_0:
	.LalWins_0:
	mov r8b, al
	cmp r8b, '-'
	jg .Lr8bWins_0
	jl .L'-'Wins_0
	mov r8b, [rdx]
	sub r8b, 1
	mov [rdx], r8b
	add rdi, 1
	jmp .Laaaaaaaauauauaaa
	.L'-'Wins_0:
	.Lr8bWins_0:
	mov r9b, r8b
	cmp r9b, '>'
	jg .Lr9bWins_0
	jl .L'>'Wins_0
	add rdx, 1
	add rdi, 1
	jmp .Laaaaaaaauauauaaa
	.L'>'Wins_0:
	.Lr9bWins_0:
	mov r10b, r9b
	cmp r10b, '<'
	jg .Lr10bWins_0
	jl .L'<'Wins_0
	sub rdx, 1
	add rdi, 1
	jmp .Laaaaaaaauauauaaa
	.Lr10bWins_0:
	.L'<'Wins_0:
	mov r11b, r10b
	cmp r11b, ','
	jg .Lr11bWins_0
	jl .L','Wins_0
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call readchar
	add rsp, 8
	jmp 2f
	1: call readchar
	2:
	mov r8b, BYTE PTR [rip + .LCharacter]
	
	mov [rdx], r8b
	add rdi, 1
	jmp .Laaaaaaaauauauaaa
	.Lr11bWins_0:
	.L','Wins_0:
	mov r12b, r11b
	cmp r12b, '.'
	jg .Lr12bWins_0
	jl .L'.'Wins_0
	mov r8b, [rdx]
	mov BYTE PTR [rip + .LCharacter], r8b
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	add rdi, 1
	jmp .Laaaaaaaauauauaaa
	.Lr12bWins_0:
	.L'.'Wins_0:
	mov r13b, r12b
	cmp r13b, '['
	jg .Lr13bWins_0
	jl .L'['Wins_0
	xor r9, r9
	mov r9b, [rdx]
	cmp r9, 0x0
	jg .Lr9Wins_0
	jl .L0x0Wins_0
	mov r8, 0x1
	.LUpgradeMarker_0:
	add rdi, 1
	xor r10, r10
	mov r10b, [rdi]
	add r10, 1
	cmp r10, 0x5C
	jg .Lr10Wins_0
	jl .L0x5CWins_0
	add r8, 1
	jmp .LUpgradeMarker_0
	.Lr10Wins_0:
	.L0x5CWins_0:
	mov r11, r10
	cmp r11, 0x5E
	jg .Lr11Wins_0
	jl .L0x5EWins_0
	cmp r8, 0x1
	jg .Lr8Wins_0
	jl .L0x1Wins_0
	add rdi, 1
	jmp .Laaaaaaaauauauaaa
	.Lr8Wins_0:
	.L0x1Wins_0:
	sub r8, 1
	.Lr11Wins_0:
	.L0x5EWins_0:
	jmp .LUpgradeMarker_0
	.Lr9Wins_0:
	.L0x0Wins_0:
	add rdi, 1
	jmp .Laaaaaaaauauauaaa
	.Lr13bWins_0:
	.L'['Wins_0:
	mov r14b, r13b
	cmp r14b, ']'
	jg .Lr14bWins_0
	jl .L']'Wins_0
	xor r12, r12
	mov r12b, [rdx]
	xor r13, r13
	cmp r12, r13
	jg .Lr12Wins_0
	jl .Lr13Wins_0
	add rdi, 1
	jmp .Laaaaaaaauauauaaa
	.Lr13Wins_0:
	.Lr12Wins_0:
	.LConfusedStonks_0: 
	mov rbx, 0x1
	.LBananaMarker_0:
	sub rdi, 1
	xor r9, r9
	mov r9b, [rdi]
	sub r9, 1
	cmp r9w, 0x5A
	jg .Lr9wWins_0
	jl .L0x5AWins_0
	add rbx, 1
	cmp rbx, 0x2
	jg .LrbxWins_0
	jl .L0x2Wins_0
	add rdi, 1
	jmp .Laaaaaaaauauauaaa
	.L0x2Wins_0:
	.LrbxWins_0:
	sub rbx, 1
	sub rbx, 1
	jmp .LBananaMarker_0
	.Lr9wWins_0:
	.L0x5AWins_0:
	add r9, 1
	add r9, 1
	add r9, 1
	mov r11w, r9w
	cmp r11w, 0x5F
	jg .Lr11wWins_0
	jl .L0x5FWins_0
	add rbx, 1
	.Lr11wWins_0:
	.L0x5FWins_0:
	jmp .LBananaMarker_0
	jmp .Laaaaaaaauauauaaa
	.Lr14bWins_0:
	.L']'Wins_0:
	add rdi, 1
	.Laaaaaaaauauauaaa:
	jmp .Luauauauauaaaa
	.LSamePicture_0:
	pop rbx
	pop r15
	pop r14
	pop r13
	pop r12
	mov rsp, rbp
	pop rbp
	mov BYTE PTR [rip + .LCharacter], '\n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	xor rax, rax
	ret


writechar:
	push rcx
	push r11
	push rax
	push rdi
	push rsi
	push rdx
	mov rdx, 1
	lea rsi, [rip + .LCharacter]
	mov rdi, 1
	mov rax, 0x2000004
	syscall
	pop rdx
	pop rsi
	pop rdi
	pop rax
	pop r11
	pop rcx
	
	ret


readchar:
	push rcx
	push r11
	push rax
	push rdi
	push rsi
	push rdx

	mov rdx, 1
	lea rsi, [rip + .LCharacter]
	mov rdi, 0
	mov rax, 0x2000003
	syscall

	pop rdx
	pop rsi
	pop rdi
	pop rax
	pop r11
	pop rcx
	ret
