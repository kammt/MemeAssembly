.intel_syntax noprefix
.global rot13

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

.stabs "/tmp/memeasm-fixture/rot13.memeasm", 100, 0, 0, .Ltext0
rot13:
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_6:
	push rdi
	.stabn 68, 0, 6, .Lcmd_6
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_9:
	.LUpgradeMarker_0:
	.stabn 68, 0, 9, .Lcmd_9
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_10:
	xor edx, edx
	.stabn 68, 0, 10, .Lcmd_10
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_11:
	mov dl, [rdi]
	.stabn 68, 0, 11, .Lcmd_11
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_14:
	cmp edx, 0x1
	jg .LedxWins_0
	jl .L0x1Wins_0
	.stabn 68, 0, 14, .Lcmd_14
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_15:
	.LedxWins_0:
	.stabn 68, 0, 15, .Lcmd_15
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_18:
	mov eax, 0x61
	.stabn 68, 0, 18, .Lcmd_18
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_19:
	.LBananaMarker_0:
	.stabn 68, 0, 19, .Lcmd_19
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.LConfusedStonks_0: 
	.Lcmd_20:
	cmp dl, al
	je .LSamePicture_0
	.stabn 68, 0, 20, .Lcmd_20
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_21:
	add eax, 1
	.stabn 68, 0, 21, .Lcmd_21
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_22:
	cmp eax, 0x7A
	jg .LeaxWins_0
	jl .L0x7AWins_0
	.stabn 68, 0, 22, .Lcmd_22
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_23:
	.L0x7AWins_0:
	.stabn 68, 0, 23, .Lcmd_23
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_24:
	jmp .LBananaMarker_0
	.stabn 68, 0, 24, .Lcmd_24
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_27:
	.LSamePicture_0:
	.stabn 68, 0, 27, .Lcmd_27
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_30:
	add dx, 0xD
	.stabn 68, 0, 30, .Lcmd_30
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_31:
	cmp dx, 0x7B
	jg .LdxWins_0
	jl .L0x7BWins_0
	.stabn 68, 0, 31, .Lcmd_31
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_32:
	.LdxWins_0:
	.stabn 68, 0, 32, .Lcmd_32
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_33:
	.Luauauaa:
	.stabn 68, 0, 33, .Lcmd_33
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_34:
	sub dx, 0x1A
	.stabn 68, 0, 34, .Lcmd_34
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_35:
	.L0x7BWins_0:
	.stabn 68, 0, 35, .Lcmd_35
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_36:
	.L0x5BWins_0:
	.stabn 68, 0, 36, .Lcmd_36
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_37:
	mov [rdi], dl
	.stabn 68, 0, 37, .Lcmd_37
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_39:
	add rdi, 1
	.stabn 68, 0, 39, .Lcmd_39
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_40:
	jmp .LUpgradeMarker_0
	.stabn 68, 0, 40, .Lcmd_40
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_43:
	.LeaxWins_0:
	.stabn 68, 0, 43, .Lcmd_43
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_46:
	mov ecx, 0x41
	.stabn 68, 0, 46, .Lcmd_46
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_47:
	.Luaaua:
	.stabn 68, 0, 47, .Lcmd_47
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_48:
	cmp dl, cl
	jg .LdlWins_0
	jl .LclWins_0
	.stabn 68, 0, 48, .Lcmd_48
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_51:
	add dx, 0xD
	.stabn 68, 0, 51, .Lcmd_51
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_52:
	cmp dx, 0x5B
	jg .LdxWins_0
	jl .L0x5BWins_0
	.stabn 68, 0, 52, .Lcmd_52
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_53:
	jmp .Luauauaa
	.stabn 68, 0, 53, .Lcmd_53
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_55:
	.LdlWins_0:
	.stabn 68, 0, 55, .Lcmd_55
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_56:
	.LclWins_0:
	.stabn 68, 0, 56, .Lcmd_56
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_57:
	add ecx, 1
	.stabn 68, 0, 57, .Lcmd_57
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_58:
	cmp ecx, 0x5A
	jg .LecxWins_0
	jl .L0x5AWins_0
	.stabn 68, 0, 58, .Lcmd_58
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_59:
	.L0x5AWins_0:
	.stabn 68, 0, 59, .Lcmd_59
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_60:
	jmp .Luaaua
	.stabn 68, 0, 60, .Lcmd_60
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_63:
	.LecxWins_0:
	.stabn 68, 0, 63, .Lcmd_63
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_64:
	add rdi, 1
	.stabn 68, 0, 64, .Lcmd_64
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_65:
	jmp .LUpgradeMarker_0
	.stabn 68, 0, 65, .Lcmd_65
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_67:
	.L0x1Wins_0:
	.stabn 68, 0, 67, .Lcmd_67
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_68:
	pop rdi
	.stabn 68, 0, 68, .Lcmd_68
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13
	.Lcmd_70:
	xor rax, rax
	ret
	.Lret_rot13:
	.stabn 68, 0, 70, .Lcmd_70
.stabs "rot13:F1", 36, 0, 0, rot13
.stabn 192, 0, 0, rot13
.stabn 224, 0, 0, .Lret_rot13


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
