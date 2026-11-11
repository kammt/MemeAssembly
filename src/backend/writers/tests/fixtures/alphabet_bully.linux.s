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

	mov eax, 0x41
	.LUpgradeMarker_0:
	mov BYTE PTR [rip + .LCharacter], al
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
	
	add eax, 1
	cmp eax, 0x5B
	je .LSamePicture_0
	jmp .LUpgradeMarker_0
	.LSamePicture_0:
	.LConfusedStonks_0: 
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
