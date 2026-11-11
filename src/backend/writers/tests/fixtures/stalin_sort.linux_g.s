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

.stabs "/tmp/memeasm-fixture/stalin-sort.memeasm", 100, 0, 0, .Ltext0
stalinSort:
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_15:
	xor r8, r8
	.stabn 68, 0, 15, .Lcmd_15
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_16:
	xor r9, r9
	.stabn 68, 0, 16, .Lcmd_16
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_19:
	mov ecx, [rdi]
	.stabn 68, 0, 19, .Lcmd_19
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_20:
	mov rax, rdi
	.stabn 68, 0, 20, .Lcmd_20
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_23:
	add rdi, 0x4
	.stabn 68, 0, 23, .Lcmd_23
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_24:
	add rax, 0x4
	.stabn 68, 0, 24, .Lcmd_24
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_27:
	add r8, 1
	.stabn 68, 0, 27, .Lcmd_27
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_28:
	add r9, 1
	.stabn 68, 0, 28, .Lcmd_28
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.LConfusedStonks_0: 
	.Lcmd_30:
	cmp r8, [rsi]
	je .LSamePicture_0
	.stabn 68, 0, 30, .Lcmd_30
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_32:
	.LUpgradeMarker_0:
	.stabn 68, 0, 32, .Lcmd_32
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_34:
	mov r10d, [rdi]
	.stabn 68, 0, 34, .Lcmd_34
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_35:
	cmp r10d, ecx
	jg .Lr10dWins_0
	jl .LecxWins_0
	.stabn 68, 0, 35, .Lcmd_35
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_38:
	.Lr10dWins_0:
	.stabn 68, 0, 38, .Lcmd_38
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_39:
	mov [rax], r10d
	.stabn 68, 0, 39, .Lcmd_39
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_41:
	mov ecx, r10d
	.stabn 68, 0, 41, .Lcmd_41
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_42:
	add rax, 0x4
	.stabn 68, 0, 42, .Lcmd_42
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_43:
	add r9, 1
	.stabn 68, 0, 43, .Lcmd_43
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_45:
	.LecxWins_0:
	.stabn 68, 0, 45, .Lcmd_45
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_46:
	add rdi, 0x4
	.stabn 68, 0, 46, .Lcmd_46
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_47:
	add r8, 1
	.stabn 68, 0, 47, .Lcmd_47
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_49:
	cmp r8, [rsi]
	je .LSamePicture_0
	.stabn 68, 0, 49, .Lcmd_49
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_50:
	jmp .LUpgradeMarker_0
	.stabn 68, 0, 50, .Lcmd_50
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_53:
	.LSamePicture_0:
	.stabn 68, 0, 53, .Lcmd_53
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_56:
	mov [rsi], r9
	.stabn 68, 0, 56, .Lcmd_56
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort
	.Lcmd_58:
	ret
	.Lret_stalinSort:
	.stabn 68, 0, 58, .Lcmd_58
.stabs "stalinSort:F1", 36, 0, 0, stalinSort
.stabn 192, 0, 0, stalinSort
.stabn 224, 0, 0, .Lret_stalinSort


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
