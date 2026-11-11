.intel_syntax noprefix
.global toupper_str

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

.stabs "/tmp/memeasm-fixture/toupper.memeasm", 100, 0, 0, .Ltext0
toupper_str:
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_4:
	mov eax, 0x61
	.stabn 68, 0, 4, .Lcmd_4
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_7:
	.LUpgradeMarker_0:
	.stabn 68, 0, 7, .Lcmd_7
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_8:
	push rdi
	.stabn 68, 0, 8, .Lcmd_8
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_11:
	.LBananaMarker_0:
	.stabn 68, 0, 11, .Lcmd_11
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_12:
	xor edx, edx
	.stabn 68, 0, 12, .Lcmd_12
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_13:
	mov dl, [rdi]
	.stabn 68, 0, 13, .Lcmd_13
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_14:
	cmp edx, 0x1
	jg .LedxWins_0
	jl .L0x1Wins_0
	.stabn 68, 0, 14, .Lcmd_14
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_17:
	.LedxWins_0:
	.stabn 68, 0, 17, .Lcmd_17
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_18:
	cmp dl, al
	jg .LdlWins_0
	jl .LalWins_0
	.stabn 68, 0, 18, .Lcmd_18
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_22:
	and dl, 0xDF
	.stabn 68, 0, 22, .Lcmd_22
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_23:
	mov [rdi], dl
	.stabn 68, 0, 23, .Lcmd_23
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_25:
	.LdlWins_0:
	.stabn 68, 0, 25, .Lcmd_25
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_26:
	.LalWins_0:
	.stabn 68, 0, 26, .Lcmd_26
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_28:
	add rdi, 1
	.stabn 68, 0, 28, .Lcmd_28
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_29:
	jmp .LBananaMarker_0
	.stabn 68, 0, 29, .Lcmd_29
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_32:
	.L0x1Wins_0:
	.stabn 68, 0, 32, .Lcmd_32
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_33:
	pop rdi
	.stabn 68, 0, 33, .Lcmd_33
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.LConfusedStonks_0: 
	.Lcmd_35:
	add eax, 1
	.stabn 68, 0, 35, .Lcmd_35
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_36:
	cmp eax, 0x7B
	je .LSamePicture_0
	.stabn 68, 0, 36, .Lcmd_36
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_38:
	jmp .LUpgradeMarker_0
	.stabn 68, 0, 38, .Lcmd_38
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_40:
	.LSamePicture_0:
	.stabn 68, 0, 40, .Lcmd_40
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str
	.Lcmd_41:
	ret
	.Lret_toupper_str:
	.stabn 68, 0, 41, .Lcmd_41
.stabs "toupper_str:F1", 36, 0, 0, toupper_str
.stabn 192, 0, 0, toupper_str
.stabn 224, 0, 0, .Lret_toupper_str


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

.LEOF:
.stabs "", 100, 0, 0, .LEOF
