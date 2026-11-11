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

.stabs "/tmp/memeasm-fixture/reverse-string.memeasm", 100, 0, 0, .Ltext0
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

	.Lcmd_3:
	xor rax, rax
	.stabn 68, 0, 3, .Lcmd_3
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_6:
	xor rcx, rcx
	.stabn 68, 0, 6, .Lcmd_6
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_8:
	.LUpgradeMarker_0:
	.stabn 68, 0, 8, .Lcmd_8
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_11:
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call readchar
	add rsp, 8
	jmp 2f
	1: call readchar
	2:
	mov al, BYTE PTR [rip + .LCharacter]
	
	.stabn 68, 0, 11, .Lcmd_11
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_14:
	cmp al, '\n'
	je .LSamePicture_0
	.stabn 68, 0, 14, .Lcmd_14
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_17:
	push rax
	.stabn 68, 0, 17, .Lcmd_17
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_18:
	add rcx, 1
	.stabn 68, 0, 18, .Lcmd_18
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_20:
	jmp .LUpgradeMarker_0
	.stabn 68, 0, 20, .Lcmd_20
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.LConfusedStonks_0: 
	.Lcmd_22:
	.LSamePicture_0:
	.stabn 68, 0, 22, .Lcmd_22
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_26:
	.LBananaMarker_0:
	.stabn 68, 0, 26, .Lcmd_26
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_29:
	cmp rcx, 0x0
	jg .LrcxWins_0
	jl .L0x0Wins_0
	.stabn 68, 0, 29, .Lcmd_29
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_31:
	.L0x0Wins_0:
	.stabn 68, 0, 31, .Lcmd_31
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_33:
	jmp .Luaaaaaua
	.stabn 68, 0, 33, .Lcmd_33
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_35:
	.LrcxWins_0:
	.stabn 68, 0, 35, .Lcmd_35
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_38:
	pop rax
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
	.Lcmd_41:
	sub rcx, 1
	.stabn 68, 0, 41, .Lcmd_41
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_43:
	jmp .LBananaMarker_0
	.stabn 68, 0, 43, .Lcmd_43
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_45:
	.Luaaaaaua:
	.stabn 68, 0, 45, .Lcmd_45
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_47:
	mov BYTE PTR [rip + .LCharacter], '\n'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	.stabn 68, 0, 47, .Lcmd_47
.stabs "main:F1", 36, 0, 0, main
.stabn 192, 0, 0, main
.stabn 224, 0, 0, .Lret_main
	.Lcmd_49:
	xor rax, rax
	ret
	.Lret_main:
	.stabn 68, 0, 49, .Lcmd_49
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
