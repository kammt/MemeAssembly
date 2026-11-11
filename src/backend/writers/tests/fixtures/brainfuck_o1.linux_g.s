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

.stabs "/tmp/memeasm-fixture/brainfuck.memeasm", 100, 0, 0, .Ltext0
main:
	nop
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

	.Lcmd_10:
	mov rdi, rsi
	nop
	.stabn 68, 0, 10, .Lcmd_10
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_12:
	add rdi, 0x8
	nop
	.stabn 68, 0, 12, .Lcmd_12
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_13:
	mov rdi, [rdi]
	nop
	.stabn 68, 0, 13, .Lcmd_13
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_16:
	push rbp
	nop
	.stabn 68, 0, 16, .Lcmd_16
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_17:
	mov rbp, rsp
	nop
	.stabn 68, 0, 17, .Lcmd_17
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_20:
	sub rsp, 0x2710
	nop
	.stabn 68, 0, 20, .Lcmd_20
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_22:
	mov rdx, rsp
	nop
	.stabn 68, 0, 22, .Lcmd_22
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_23:
	xor rax, rax
	nop
	.stabn 68, 0, 23, .Lcmd_23
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_24:
	xor r8b, r8b
	nop
	.stabn 68, 0, 24, .Lcmd_24
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_25:
	.Luuuuuuuuauuuuuauauaa:
	nop
	.stabn 68, 0, 25, .Lcmd_25
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_26:
	cmp rax, 0x2710
	jg .LraxWins_0
	jl .L0x2710Wins_0
	nop
	.stabn 68, 0, 26, .Lcmd_26
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_27:
	jmp .Laaaaaaaaaaauaaaaaaaaaaaauu
	nop
	.stabn 68, 0, 27, .Lcmd_27
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_28:
	.LraxWins_0:
	nop
	.stabn 68, 0, 28, .Lcmd_28
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_29:
	.L0x2710Wins_0:
	nop
	.stabn 68, 0, 29, .Lcmd_29
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_30:
	mov [rdx], r8b
	nop
	.stabn 68, 0, 30, .Lcmd_30
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_31:
	add rdx, 1
	nop
	.stabn 68, 0, 31, .Lcmd_31
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_32:
	add rax, 1
	nop
	.stabn 68, 0, 32, .Lcmd_32
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_33:
	jmp .Luuuuuuuuauuuuuauauaa
	nop
	.stabn 68, 0, 33, .Lcmd_33
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_34:
	.Laaaaaaaaaaauaaaaaaaaaaaauu:
	nop
	.stabn 68, 0, 34, .Lcmd_34
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_35:
	mov rdx, rsp
	nop
	.stabn 68, 0, 35, .Lcmd_35
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_40:
	push r12
	nop
	.stabn 68, 0, 40, .Lcmd_40
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_41:
	push r13
	nop
	.stabn 68, 0, 41, .Lcmd_41
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_42:
	push r14
	nop
	.stabn 68, 0, 42, .Lcmd_42
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_43:
	push r15
	nop
	.stabn 68, 0, 43, .Lcmd_43
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_44:
	push rbx
	nop
	.stabn 68, 0, 44, .Lcmd_44
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_48:
	.Luauauauauaaaa:
	nop
	.stabn 68, 0, 48, .Lcmd_48
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_49:
	mov al, [rdi]
	nop
	.stabn 68, 0, 49, .Lcmd_49
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_51:
	cmp al, 0x0
	je .LSamePicture_0
	nop
	.stabn 68, 0, 51, .Lcmd_51
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_53:
	cmp al, '+'
	jg .LalWins_0
	jl .L'+'Wins_0
	nop
	.stabn 68, 0, 53, .Lcmd_53
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_54:
	mov r8b, [rdx]
	nop
	.stabn 68, 0, 54, .Lcmd_54
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_55:
	add r8b, 1
	nop
	.stabn 68, 0, 55, .Lcmd_55
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_56:
	mov [rdx], r8b
	nop
	.stabn 68, 0, 56, .Lcmd_56
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_57:
	add rdi, 1
	nop
	.stabn 68, 0, 57, .Lcmd_57
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_58:
	jmp .Laaaaaaaauauauaaa
	nop
	.stabn 68, 0, 58, .Lcmd_58
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_60:
	.L'+'Wins_0:
	nop
	.stabn 68, 0, 60, .Lcmd_60
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_61:
	.LalWins_0:
	nop
	.stabn 68, 0, 61, .Lcmd_61
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_62:
	mov r8b, al
	nop
	.stabn 68, 0, 62, .Lcmd_62
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_64:
	cmp r8b, '-'
	jg .Lr8bWins_0
	jl .L'-'Wins_0
	nop
	.stabn 68, 0, 64, .Lcmd_64
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_66:
	mov r8b, [rdx]
	nop
	.stabn 68, 0, 66, .Lcmd_66
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_67:
	sub r8b, 1
	nop
	.stabn 68, 0, 67, .Lcmd_67
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_68:
	mov [rdx], r8b
	nop
	.stabn 68, 0, 68, .Lcmd_68
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_69:
	add rdi, 1
	nop
	.stabn 68, 0, 69, .Lcmd_69
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_70:
	jmp .Laaaaaaaauauauaaa
	nop
	.stabn 68, 0, 70, .Lcmd_70
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_72:
	.L'-'Wins_0:
	nop
	.stabn 68, 0, 72, .Lcmd_72
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_73:
	.Lr8bWins_0:
	nop
	.stabn 68, 0, 73, .Lcmd_73
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_74:
	mov r9b, r8b
	nop
	.stabn 68, 0, 74, .Lcmd_74
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_76:
	cmp r9b, '>'
	jg .Lr9bWins_0
	jl .L'>'Wins_0
	nop
	.stabn 68, 0, 76, .Lcmd_76
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_78:
	add rdx, 1
	nop
	.stabn 68, 0, 78, .Lcmd_78
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_79:
	add rdi, 1
	nop
	.stabn 68, 0, 79, .Lcmd_79
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_80:
	jmp .Laaaaaaaauauauaaa
	nop
	.stabn 68, 0, 80, .Lcmd_80
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_82:
	.L'>'Wins_0:
	nop
	.stabn 68, 0, 82, .Lcmd_82
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_83:
	.Lr9bWins_0:
	nop
	.stabn 68, 0, 83, .Lcmd_83
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_84:
	mov r10b, r9b
	nop
	.stabn 68, 0, 84, .Lcmd_84
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_86:
	cmp r10b, '<'
	jg .Lr10bWins_0
	jl .L'<'Wins_0
	nop
	.stabn 68, 0, 86, .Lcmd_86
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_87:
	sub rdx, 1
	nop
	.stabn 68, 0, 87, .Lcmd_87
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_88:
	add rdi, 1
	nop
	.stabn 68, 0, 88, .Lcmd_88
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_89:
	jmp .Laaaaaaaauauauaaa
	nop
	.stabn 68, 0, 89, .Lcmd_89
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_91:
	.Lr10bWins_0:
	nop
	.stabn 68, 0, 91, .Lcmd_91
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_92:
	.L'<'Wins_0:
	nop
	.stabn 68, 0, 92, .Lcmd_92
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_93:
	mov r11b, r10b
	nop
	.stabn 68, 0, 93, .Lcmd_93
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_95:
	cmp r11b, ','
	jg .Lr11bWins_0
	jl .L','Wins_0
	nop
	.stabn 68, 0, 95, .Lcmd_95
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_97:
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call readchar
	add rsp, 8
	jmp 2f
	1: call readchar
	2:
	mov r8b, BYTE PTR [rip + .LCharacter]
	
	nop
	.stabn 68, 0, 97, .Lcmd_97
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_98:
	mov [rdx], r8b
	nop
	.stabn 68, 0, 98, .Lcmd_98
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_99:
	add rdi, 1
	nop
	.stabn 68, 0, 99, .Lcmd_99
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_100:
	jmp .Laaaaaaaauauauaaa
	nop
	.stabn 68, 0, 100, .Lcmd_100
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_102:
	.Lr11bWins_0:
	nop
	.stabn 68, 0, 102, .Lcmd_102
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_103:
	.L','Wins_0:
	nop
	.stabn 68, 0, 103, .Lcmd_103
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_104:
	mov r12b, r11b
	nop
	.stabn 68, 0, 104, .Lcmd_104
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_106:
	cmp r12b, '.'
	jg .Lr12bWins_0
	jl .L'.'Wins_0
	nop
	.stabn 68, 0, 106, .Lcmd_106
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_108:
	mov r8b, [rdx]
	nop
	.stabn 68, 0, 108, .Lcmd_108
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_109:
	mov BYTE PTR [rip + .LCharacter], r8b
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	nop
	.stabn 68, 0, 109, .Lcmd_109
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_110:
	add rdi, 1
	nop
	.stabn 68, 0, 110, .Lcmd_110
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_111:
	jmp .Laaaaaaaauauauaaa
	nop
	.stabn 68, 0, 111, .Lcmd_111
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_113:
	.Lr12bWins_0:
	nop
	.stabn 68, 0, 113, .Lcmd_113
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_114:
	.L'.'Wins_0:
	nop
	.stabn 68, 0, 114, .Lcmd_114
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_115:
	mov r13b, r12b
	nop
	.stabn 68, 0, 115, .Lcmd_115
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_117:
	cmp r13b, '['
	jg .Lr13bWins_0
	jl .L'['Wins_0
	nop
	.stabn 68, 0, 117, .Lcmd_117
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_120:
	xor r9, r9
	nop
	.stabn 68, 0, 120, .Lcmd_120
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_121:
	mov r9b, [rdx]
	nop
	.stabn 68, 0, 121, .Lcmd_121
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_122:
	cmp r9, 0x0
	jg .Lr9Wins_0
	jl .L0x0Wins_0
	nop
	.stabn 68, 0, 122, .Lcmd_122
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_124:
	mov r8, 0x1
	nop
	.stabn 68, 0, 124, .Lcmd_124
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_125:
	.LUpgradeMarker_0:
	nop
	.stabn 68, 0, 125, .Lcmd_125
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_126:
	add rdi, 1
	nop
	.stabn 68, 0, 126, .Lcmd_126
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_127:
	xor r10, r10
	nop
	.stabn 68, 0, 127, .Lcmd_127
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_128:
	mov r10b, [rdi]
	nop
	.stabn 68, 0, 128, .Lcmd_128
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_129:
	add r10, 1
	nop
	.stabn 68, 0, 129, .Lcmd_129
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_130:
	cmp r10, 0x5C
	jg .Lr10Wins_0
	jl .L0x5CWins_0
	nop
	.stabn 68, 0, 130, .Lcmd_130
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_132:
	add r8, 1
	nop
	.stabn 68, 0, 132, .Lcmd_132
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_133:
	jmp .LUpgradeMarker_0
	nop
	.stabn 68, 0, 133, .Lcmd_133
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_135:
	.Lr10Wins_0:
	nop
	.stabn 68, 0, 135, .Lcmd_135
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_136:
	.L0x5CWins_0:
	nop
	.stabn 68, 0, 136, .Lcmd_136
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_137:
	mov r11, r10
	nop
	.stabn 68, 0, 137, .Lcmd_137
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_139:
	cmp r11, 0x5E
	jg .Lr11Wins_0
	jl .L0x5EWins_0
	nop
	.stabn 68, 0, 139, .Lcmd_139
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_143:
	cmp r8, 0x1
	jg .Lr8Wins_0
	jl .L0x1Wins_0
	nop
	.stabn 68, 0, 143, .Lcmd_143
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_144:
	add rdi, 1
	nop
	.stabn 68, 0, 144, .Lcmd_144
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_145:
	jmp .Laaaaaaaauauauaaa
	nop
	.stabn 68, 0, 145, .Lcmd_145
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_147:
	.Lr8Wins_0:
	nop
	.stabn 68, 0, 147, .Lcmd_147
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_148:
	.L0x1Wins_0:
	nop
	.stabn 68, 0, 148, .Lcmd_148
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_149:
	sub r8, 1
	nop
	.stabn 68, 0, 149, .Lcmd_149
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_150:
	.Lr11Wins_0:
	nop
	.stabn 68, 0, 150, .Lcmd_150
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_151:
	.L0x5EWins_0:
	nop
	.stabn 68, 0, 151, .Lcmd_151
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_152:
	jmp .LUpgradeMarker_0
	nop
	.stabn 68, 0, 152, .Lcmd_152
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_154:
	.Lr9Wins_0:
	nop
	.stabn 68, 0, 154, .Lcmd_154
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_155:
	.L0x0Wins_0:
	nop
	.stabn 68, 0, 155, .Lcmd_155
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_157:
	add rdi, 1
	nop
	.stabn 68, 0, 157, .Lcmd_157
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_158:
	jmp .Laaaaaaaauauauaaa
	nop
	.stabn 68, 0, 158, .Lcmd_158
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_160:
	.Lr13bWins_0:
	nop
	.stabn 68, 0, 160, .Lcmd_160
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_161:
	.L'['Wins_0:
	nop
	.stabn 68, 0, 161, .Lcmd_161
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_162:
	mov r14b, r13b
	nop
	.stabn 68, 0, 162, .Lcmd_162
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_165:
	cmp r14b, ']'
	jg .Lr14bWins_0
	jl .L']'Wins_0
	nop
	.stabn 68, 0, 165, .Lcmd_165
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_167:
	xor r12, r12
	nop
	.stabn 68, 0, 167, .Lcmd_167
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_168:
	mov r12b, [rdx]
	nop
	.stabn 68, 0, 168, .Lcmd_168
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_169:
	xor r13, r13
	nop
	.stabn 68, 0, 169, .Lcmd_169
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_170:
	cmp r12, r13
	jg .Lr12Wins_0
	jl .Lr13Wins_0
	nop
	.stabn 68, 0, 170, .Lcmd_170
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_172:
	add rdi, 1
	nop
	.stabn 68, 0, 172, .Lcmd_172
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_173:
	jmp .Laaaaaaaauauauaaa
	nop
	.stabn 68, 0, 173, .Lcmd_173
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_174:
	.Lr13Wins_0:
	nop
	.stabn 68, 0, 174, .Lcmd_174
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_175:
	.Lr12Wins_0:
	nop
	.stabn 68, 0, 175, .Lcmd_175
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.LConfusedStonks_0: 
	.Lcmd_177:
	mov rbx, 0x1
	nop
	.stabn 68, 0, 177, .Lcmd_177
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_178:
	.LBananaMarker_0:
	nop
	.stabn 68, 0, 178, .Lcmd_178
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_179:
	sub rdi, 1
	nop
	.stabn 68, 0, 179, .Lcmd_179
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_180:
	xor r9, r9
	nop
	.stabn 68, 0, 180, .Lcmd_180
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_181:
	mov r9b, [rdi]
	nop
	.stabn 68, 0, 181, .Lcmd_181
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_183:
	sub r9, 1
	nop
	.stabn 68, 0, 183, .Lcmd_183
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_184:
	cmp r9w, 0x5A
	jg .Lr9wWins_0
	jl .L0x5AWins_0
	nop
	.stabn 68, 0, 184, .Lcmd_184
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_186:
	add rbx, 1
	nop
	.stabn 68, 0, 186, .Lcmd_186
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_188:
	cmp rbx, 0x2
	jg .LrbxWins_0
	jl .L0x2Wins_0
	nop
	.stabn 68, 0, 188, .Lcmd_188
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_189:
	add rdi, 1
	nop
	.stabn 68, 0, 189, .Lcmd_189
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_190:
	jmp .Laaaaaaaauauauaaa
	nop
	.stabn 68, 0, 190, .Lcmd_190
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_192:
	.L0x2Wins_0:
	nop
	.stabn 68, 0, 192, .Lcmd_192
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_193:
	.LrbxWins_0:
	nop
	.stabn 68, 0, 193, .Lcmd_193
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_194:
	sub rbx, 1
	nop
	.stabn 68, 0, 194, .Lcmd_194
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_195:
	sub rbx, 1
	nop
	.stabn 68, 0, 195, .Lcmd_195
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_196:
	jmp .LBananaMarker_0
	nop
	.stabn 68, 0, 196, .Lcmd_196
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_198:
	.Lr9wWins_0:
	nop
	.stabn 68, 0, 198, .Lcmd_198
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_199:
	.L0x5AWins_0:
	nop
	.stabn 68, 0, 199, .Lcmd_199
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_200:
	add r9, 1
	nop
	.stabn 68, 0, 200, .Lcmd_200
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_201:
	add r9, 1
	nop
	.stabn 68, 0, 201, .Lcmd_201
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_202:
	add r9, 1
	nop
	.stabn 68, 0, 202, .Lcmd_202
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_203:
	mov r11w, r9w
	nop
	.stabn 68, 0, 203, .Lcmd_203
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_204:
	cmp r11w, 0x5F
	jg .Lr11wWins_0
	jl .L0x5FWins_0
	nop
	.stabn 68, 0, 204, .Lcmd_204
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_206:
	add rbx, 1
	nop
	.stabn 68, 0, 206, .Lcmd_206
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_208:
	.Lr11wWins_0:
	nop
	.stabn 68, 0, 208, .Lcmd_208
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_209:
	.L0x5FWins_0:
	nop
	.stabn 68, 0, 209, .Lcmd_209
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_210:
	jmp .LBananaMarker_0
	nop
	.stabn 68, 0, 210, .Lcmd_210
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_211:
	jmp .Laaaaaaaauauauaaa
	nop
	.stabn 68, 0, 211, .Lcmd_211
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_214:
	.Lr14bWins_0:
	nop
	.stabn 68, 0, 214, .Lcmd_214
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_215:
	.L']'Wins_0:
	nop
	.stabn 68, 0, 215, .Lcmd_215
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_216:
	add rdi, 1
	nop
	.stabn 68, 0, 216, .Lcmd_216
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_218:
	.Laaaaaaaauauauaaa:
	nop
	.stabn 68, 0, 218, .Lcmd_218
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_220:
	jmp .Luauauauauaaaa
	nop
	.stabn 68, 0, 220, .Lcmd_220
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_221:
	.LSamePicture_0:
	nop
	.stabn 68, 0, 221, .Lcmd_221
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_224:
	pop rbx
	nop
	.stabn 68, 0, 224, .Lcmd_224
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_225:
	pop r15
	nop
	.stabn 68, 0, 225, .Lcmd_225
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_226:
	pop r14
	nop
	.stabn 68, 0, 226, .Lcmd_226
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_227:
	pop r13
	nop
	.stabn 68, 0, 227, .Lcmd_227
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_228:
	pop r12
	nop
	.stabn 68, 0, 228, .Lcmd_228
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_229:
	mov rsp, rbp
	nop
	.stabn 68, 0, 229, .Lcmd_229
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_230:
	pop rbp
	nop
	.stabn 68, 0, 230, .Lcmd_230
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_231:
	mov BYTE PTR [rip + .LCharacter], '\n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	nop
	.stabn 68, 0, 231, .Lcmd_231
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_232:
	xor rax, rax
	ret
	nop
	.Lret_main:
	.stabn 68, 0, 232, .Lcmd_232
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
