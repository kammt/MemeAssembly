.intel_syntax noprefix
.global _toupper_str

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

_toupper_str:
	mov eax, 0x61
	.LUpgradeMarker_0:
	push rdi
	.LBananaMarker_0:
	xor edx, edx
	mov dl, [rdi]
	cmp edx, 0x1
	jg .LedxWins_0
	jl .L0x1Wins_0
	.LedxWins_0:
	cmp dl, al
	jg .LdlWins_0
	jl .LalWins_0
	and dl, 0xDF
	mov [rdi], dl
	.LdlWins_0:
	.LalWins_0:
	add rdi, 1
	jmp .LBananaMarker_0
	.L0x1Wins_0:
	pop rdi
	.LConfusedStonks_0: 
	add eax, 1
	cmp eax, 0x7B
	je .LSamePicture_0
	jmp .LUpgradeMarker_0
	.LSamePicture_0:
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
