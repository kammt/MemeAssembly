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

.stabs "/tmp/memeasm-fixture/the-link.memeasm", 100, 0, 0, .Ltext0
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

	.Lcmd_2:
	mov BYTE PTR [rip + .LCharacter], 'h'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 2, .Lcmd_2
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_4:
	xor ecx, ecx
	.stabn 68, 0, 4, .Lcmd_4
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_5:
	.LUpgradeMarker_0:
	.stabn 68, 0, 5, .Lcmd_5
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_6:
	mov eax, 0x74
	.stabn 68, 0, 6, .Lcmd_6
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_7:
	mov BYTE PTR [rip + .LCharacter], al
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
	add ecx, 1
	.stabn 68, 0, 8, .Lcmd_8
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_9:
	cmp ecx, 0x2
	je .LSamePicture_0
	.stabn 68, 0, 9, .Lcmd_9
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.LConfusedStonks_0: 
	.Lcmd_10:
	jmp .LUpgradeMarker_0
	.stabn 68, 0, 10, .Lcmd_10
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_12:
	.LSamePicture_0:
	.stabn 68, 0, 12, .Lcmd_12
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_14:
	sub eax, 0x4
	.stabn 68, 0, 14, .Lcmd_14
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_15:
	mov BYTE PTR [rip + .LCharacter], al
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
	.Lcmd_17:
	mov BYTE PTR [rip + .LCharacter], 's'
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
	.Lcmd_19:
	mov BYTE PTR [rip + .LCharacter], 58
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
	.Lcmd_21:
	sub eax, 0x41
	.stabn 68, 0, 21, .Lcmd_21
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_22:
	mov BYTE PTR [rip + .LCharacter], al
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
	mov BYTE PTR [rip + .LCharacter], al
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
	.Lcmd_24:
	push rax
	.stabn 68, 0, 24, .Lcmd_24
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_26:
	xor eax, eax
	.stabn 68, 0, 26, .Lcmd_26
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_27:
	sub eax, 0x87
	.stabn 68, 0, 27, .Lcmd_27
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_28:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 28, .Lcmd_28
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_29:
	sub eax, 0xA
	.stabn 68, 0, 29, .Lcmd_29
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_30:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 30, .Lcmd_30
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_31:
	push rax
	.stabn 68, 0, 31, .Lcmd_31
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_33:
	add eax, 1
	.stabn 68, 0, 33, .Lcmd_33
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_34:
	add eax, 1
	.stabn 68, 0, 34, .Lcmd_34
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_35:
	add eax, 1
	.stabn 68, 0, 35, .Lcmd_35
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_36:
	add eax, 1
	.stabn 68, 0, 36, .Lcmd_36
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_37:
	add eax, 1
	.stabn 68, 0, 37, .Lcmd_37
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_38:
	add eax, 1
	.stabn 68, 0, 38, .Lcmd_38
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_39:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 39, .Lcmd_39
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_40:
	push rax
	.stabn 68, 0, 40, .Lcmd_40
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_41:
	sub eax, 1
	.stabn 68, 0, 41, .Lcmd_41
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_42:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 42, .Lcmd_42
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_43:
	pop rax
	.stabn 68, 0, 43, .Lcmd_43
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_44:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 44, .Lcmd_44
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_46:
	mov eax, 0x65
	.stabn 68, 0, 46, .Lcmd_46
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_47:
	push rax
	.stabn 68, 0, 47, .Lcmd_47
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_48:
	add eax, 1
	.stabn 68, 0, 48, .Lcmd_48
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_49:
	and eax, 0xFFFFFFFA
	.stabn 68, 0, 49, .Lcmd_49
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_50:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 50, .Lcmd_50
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_51:
	pop rax
	.stabn 68, 0, 51, .Lcmd_51
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_52:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 52, .Lcmd_52
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_54:
	mov BYTE PTR [rip + .LCharacter], '.'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 54, .Lcmd_54
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_56:
	mov BYTE PTR [rip + .LCharacter], 'c'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 56, .Lcmd_56
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_57:
	pop rax
	.stabn 68, 0, 57, .Lcmd_57
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_58:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 58, .Lcmd_58
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_60:
	and eax, 0xFD
	.stabn 68, 0, 60, .Lcmd_60
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_61:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 61, .Lcmd_61
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_63:
	pop rax
	.stabn 68, 0, 63, .Lcmd_63
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_64:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 64, .Lcmd_64
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_66:
	mov BYTE PTR [rip + .LCharacter], 'w'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 66, .Lcmd_66
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_67:
	mov BYTE PTR [rip + .LCharacter], 97
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 67, .Lcmd_67
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_69:
	xor ebx, ebx
	.stabn 68, 0, 69, .Lcmd_69
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_70:
	sub ebx, 0x28C
	.stabn 68, 0, 70, .Lcmd_70
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_71:
	mov BYTE PTR [rip + .LCharacter], bl
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
	.Lcmd_73:
	mov edx, ebx
	.stabn 68, 0, 73, .Lcmd_73
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_74:
	sub edx, 0x11
	.stabn 68, 0, 74, .Lcmd_74
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_75:
	mov BYTE PTR [rip + .LCharacter], dl
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 75, .Lcmd_75
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_77:
	sub ebx, 0xC
	.stabn 68, 0, 77, .Lcmd_77
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_78:
	mov BYTE PTR [rip + .LCharacter], bl
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
	.Lcmd_80:
	mov BYTE PTR [rip + .LCharacter], '?'
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
	.Lcmd_82:
	mov BYTE PTR [rip + .LCharacter], 118
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
	sub eax, 0xF2
	.stabn 68, 0, 84, .Lcmd_84
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_85:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 85, .Lcmd_85
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_87:
	mov eax, 0x64
	.stabn 68, 0, 87, .Lcmd_87
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_88:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 88, .Lcmd_88
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_89:
	sub eax, 0x13
	.stabn 68, 0, 89, .Lcmd_89
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_90:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 90, .Lcmd_90
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_91:
	push rax
	.stabn 68, 0, 91, .Lcmd_91
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_93:
	add eax, 25
	.stabn 68, 0, 93, .Lcmd_93
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_94:
	add eax, 25
	.stabn 68, 0, 94, .Lcmd_94
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_95:
	sub eax, 0xC
	.stabn 68, 0, 95, .Lcmd_95
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_96:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 96, .Lcmd_96
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_98:
	push rax
	.stabn 68, 0, 98, .Lcmd_98
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_99:
	mov BYTE PTR [rip + .LCharacter], '4'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 99, .Lcmd_99
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_100:
	pop rax
	.stabn 68, 0, 100, .Lcmd_100
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_101:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 101, .Lcmd_101
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_102:
	mov BYTE PTR [rip + .LCharacter], '9'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 102, .Lcmd_102
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_104:
	pop rax
	.stabn 68, 0, 104, .Lcmd_104
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_105:
	push rax
	.stabn 68, 0, 105, .Lcmd_105
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_107:
	add eax, 1
	.stabn 68, 0, 107, .Lcmd_107
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_108:
	add eax, 1
	.stabn 68, 0, 108, .Lcmd_108
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_109:
	add eax, 1
	.stabn 68, 0, 109, .Lcmd_109
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_110:
	add eax, 1
	.stabn 68, 0, 110, .Lcmd_110
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_111:
	add eax, 1
	.stabn 68, 0, 111, .Lcmd_111
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_112:
	add eax, 1
	.stabn 68, 0, 112, .Lcmd_112
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_113:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 113, .Lcmd_113
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_115:
	sub ebx, 1
	.stabn 68, 0, 115, .Lcmd_115
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_116:
	mov BYTE PTR [rip + .LCharacter], bl
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 116, .Lcmd_116
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_120:
	sub ebx, 0xF
	.stabn 68, 0, 120, .Lcmd_120
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_121:
	mov BYTE PTR [rip + .LCharacter], bl
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 121, .Lcmd_121
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_123:
	mov BYTE PTR [rip + .LCharacter], 'c'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 123, .Lcmd_123
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_125:
	pop rax
	.stabn 68, 0, 125, .Lcmd_125
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_126:
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 126, .Lcmd_126
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_128:
	mov BYTE PTR [rip + .LCharacter], '\n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 128, .Lcmd_128
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_130:
	xor rax, rax
	ret
	.Lret_main:
	.stabn 68, 0, 130, .Lcmd_130
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
