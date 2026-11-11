.intel_syntax noprefix
.global stalinSort

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

stalinSort:
	xor r8, r8
	xor r9, r9
	mov ecx, [rdi]
	mov rax, rdi
	add rdi, 0x4
	add rax, 0x4
	add r8, 1
	add r9, 1
	.LConfusedStonks_0: 
	cmp r8, [rsi]
	je .LSamePicture_0
	.LUpgradeMarker_0:
	mov r10d, [rdi]
	cmp r10d, ecx
	jg .Lr10dWins_0
	jl .LecxWins_0
	.Lr10dWins_0:
	mov [rax], r10d
	mov ecx, r10d
	add rax, 0x4
	add r9, 1
	.LecxWins_0:
	add rdi, 0x4
	add r8, 1
	cmp r8, [rsi]
	je .LSamePicture_0
	jmp .LUpgradeMarker_0
	.LSamePicture_0:
	mov [rsi], r9
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
