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

.stabs "/tmp/memeasm-fixture/is_nice.memeasm", 100, 0, 0, .Ltext0
main:
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
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

	.Lcmd_5:
	mov BYTE PTR [rip + .LCharacter], 'E'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 5, .Lcmd_5
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_6:
	mov BYTE PTR [rip + .LCharacter], 'n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 6, .Lcmd_6
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_7:
	mov BYTE PTR [rip + .LCharacter], 't'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 7, .Lcmd_7
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_8:
	mov BYTE PTR [rip + .LCharacter], 'e'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 8, .Lcmd_8
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_9:
	mov BYTE PTR [rip + .LCharacter], 'r'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 9, .Lcmd_9
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_10:
	mov BYTE PTR [rip + .LCharacter], ' '
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 10, .Lcmd_10
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_11:
	mov BYTE PTR [rip + .LCharacter], 'y'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 11, .Lcmd_11
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_12:
	mov BYTE PTR [rip + .LCharacter], 'o'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 12, .Lcmd_12
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_13:
	mov BYTE PTR [rip + .LCharacter], 'u'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 13, .Lcmd_13
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_14:
	mov BYTE PTR [rip + .LCharacter], 'r'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 14, .Lcmd_14
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_15:
	mov BYTE PTR [rip + .LCharacter], ' '
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 15, .Lcmd_15
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_16:
	mov BYTE PTR [rip + .LCharacter], 'n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 16, .Lcmd_16
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_17:
	mov BYTE PTR [rip + .LCharacter], 'u'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 17, .Lcmd_17
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_18:
	mov BYTE PTR [rip + .LCharacter], 'm'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 18, .Lcmd_18
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_19:
	mov BYTE PTR [rip + .LCharacter], 'b'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 19, .Lcmd_19
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_20:
	mov BYTE PTR [rip + .LCharacter], 'e'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 20, .Lcmd_20
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_21:
	mov BYTE PTR [rip + .LCharacter], 'r'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 21, .Lcmd_21
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_22:
	mov BYTE PTR [rip + .LCharacter], ':'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 22, .Lcmd_22
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_23:
	mov BYTE PTR [rip + .LCharacter], ' '
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 23, .Lcmd_23
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_25:
	xor r8, r8
	.stabn 68, 0, 25, .Lcmd_25
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_27:
	.LUpgradeMarker_0:
	.stabn 68, 0, 27, .Lcmd_27
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_28:
	xor rax, rax
	.stabn 68, 0, 28, .Lcmd_28
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_29:
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call readchar
	add rsp, 8
	jmp 2f
	1: call readchar
	2:
	mov al, BYTE PTR [rip + .LCharacter]
	
	.stabn 68, 0, 29, .Lcmd_29
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_31:
	cmp al, 0xA
	jg .LalWins_0
	jl .L0xAWins_0
	.stabn 68, 0, 31, .Lcmd_31
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_34:
	jmp .LBananaMarker_0
	.stabn 68, 0, 34, .Lcmd_34
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_36:
	.LalWins_0:
	.stabn 68, 0, 36, .Lcmd_36
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_37:
	.L0xAWins_0:
	.stabn 68, 0, 37, .Lcmd_37
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_38:
	sub al, 0x30
	.stabn 68, 0, 38, .Lcmd_38
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_40:
	add r8, 1
	.stabn 68, 0, 40, .Lcmd_40
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_41:
	push rax
	.stabn 68, 0, 41, .Lcmd_41
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_42:
	jmp .LUpgradeMarker_0
	.stabn 68, 0, 42, .Lcmd_42
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_44:
	.LBananaMarker_0:
	.stabn 68, 0, 44, .Lcmd_44
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_46:
	xor rdx, rdx
	.stabn 68, 0, 46, .Lcmd_46
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_47:
	mov rcx, 0x1
	.stabn 68, 0, 47, .Lcmd_47
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_49:
	.Luuaauauaua:
	.stabn 68, 0, 49, .Lcmd_49
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_50:
	cmp r8, 0x0
	jg .Lr8Wins_0
	jl .L0x0Wins_0
	.stabn 68, 0, 50, .Lcmd_50
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_52:
	jmp .Luauauauauaaa
	.stabn 68, 0, 52, .Lcmd_52
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_54:
	.Lr8Wins_0:
	.stabn 68, 0, 54, .Lcmd_54
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_55:
	.L0x0Wins_0:
	.stabn 68, 0, 55, .Lcmd_55
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_56:
	pop rax
	.stabn 68, 0, 56, .Lcmd_56
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_58:
	imul rax, rcx
	.stabn 68, 0, 58, .Lcmd_58
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_59:
	add rdx, rax
	.stabn 68, 0, 59, .Lcmd_59
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_61:
	imul rcx, 0xA
	.stabn 68, 0, 61, .Lcmd_61
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_63:
	sub r8, 1
	.stabn 68, 0, 63, .Lcmd_63
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_64:
	jmp .Luuaauauaua
	.stabn 68, 0, 64, .Lcmd_64
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_67:
	.Luauauauauaaa:
	.stabn 68, 0, 67, .Lcmd_67
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_68:
	cmp rdx, 0x45
	je .LSamePicture_0
	.stabn 68, 0, 68, .Lcmd_68
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_69:
	cmp rdx, 0x1A4
	je .LSamePicture_0
	.stabn 68, 0, 69, .Lcmd_69
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_71:
	mov BYTE PTR [rip + .LCharacter], 'n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 71, .Lcmd_71
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.LConfusedStonks_0: 
	.Lcmd_72:
	mov BYTE PTR [rip + .LCharacter], 'o'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 72, .Lcmd_72
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_73:
	mov BYTE PTR [rip + .LCharacter], 't'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 73, .Lcmd_73
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_74:
	mov BYTE PTR [rip + .LCharacter], ' '
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 74, .Lcmd_74
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_76:
	.LSamePicture_0:
	.stabn 68, 0, 76, .Lcmd_76
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_77:
	mov BYTE PTR [rip + .LCharacter], 'n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 77, .Lcmd_77
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_78:
	mov BYTE PTR [rip + .LCharacter], 'i'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 78, .Lcmd_78
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_79:
	mov BYTE PTR [rip + .LCharacter], 'c'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 79, .Lcmd_79
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_80:
	mov BYTE PTR [rip + .LCharacter], 'e'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 80, .Lcmd_80
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_81:
	mov BYTE PTR [rip + .LCharacter], '!'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 81, .Lcmd_81
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_82:
	mov BYTE PTR [rip + .LCharacter], '\n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 82, .Lcmd_82
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_84:
	xor rax, rax
	ret
	.Lret_main:
	.stabn 68, 0, 84, .Lcmd_84
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main


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
