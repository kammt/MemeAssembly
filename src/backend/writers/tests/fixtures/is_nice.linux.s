.intel_syntax noprefix
.global main

.data
	.LCharacter: .ascii "a"
	.Ltmp64: .byte 0, 0, 0, 0, 0, 0, 0, 0
	.LsigStruct:
		.Lsa_handler: .quad 0
		.quad 0x04000000
		.quad 0, 0



.text
	

.Ltext0:
killParent:
    mov rax, 110
    syscall

    mov rdi, rax
    mov rsi, 9
    mov rax, 62
    syscall

    mov rdi, 0
    mov rax, 60
    syscall
    ret

main:
push rax
    push rdi
    push rsi
    push rdx
    push r10
    push rcx
    push r11
    
    lea rax, [rip + killParent]
    mov [rip + .Lsa_handler], rax

    mov rax, 13
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

	mov BYTE PTR [rip + .LCharacter], 'E'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 't'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'e'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'r'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], ' '
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'y'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'o'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'u'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'r'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], ' '
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'u'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'm'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'b'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'e'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'r'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], ':'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], ' '
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	xor r8, r8
	.LUpgradeMarker_0:
	xor rax, rax
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call readchar
	add rsp, 8
	jmp 2f
	1: call readchar
	2:
	mov al, BYTE PTR [rip + .LCharacter]
	
	cmp al, 0xA
	jg .LalWins_0
	jl .L0xAWins_0
	jmp .LBananaMarker_0
	.LalWins_0:
	.L0xAWins_0:
	sub al, 0x30
	add r8, 1
	push rax
	jmp .LUpgradeMarker_0
	.LBananaMarker_0:
	xor rdx, rdx
	mov rcx, 0x1
	.Luuaauauaua:
	cmp r8, 0x0
	jg .Lr8Wins_0
	jl .L0x0Wins_0
	jmp .Luauauauauaaa
	.Lr8Wins_0:
	.L0x0Wins_0:
	pop rax
	imul rax, rcx
	add rdx, rax
	imul rcx, 0xA
	sub r8, 1
	jmp .Luuaauauaua
	.Luauauauauaaa:
	cmp rdx, 0x45
	je .LSamePicture_0
	cmp rdx, 0x1A4
	je .LSamePicture_0
	mov BYTE PTR [rip + .LCharacter], 'n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.LConfusedStonks_0: 
	mov BYTE PTR [rip + .LCharacter], 'o'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 't'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], ' '
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.LSamePicture_0:
	mov BYTE PTR [rip + .LCharacter], 'n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'i'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'c'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'e'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], '!'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
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
	mov rax, 1
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
	mov rax, 0
	syscall

	pop rdx
	pop rsi
	pop rdi
	pop rax
	pop r11
	pop rcx
	ret
