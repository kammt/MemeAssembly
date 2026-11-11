.intel_syntax noprefix
.global main

.extern GetStdHandle
.extern WriteFile
.extern ReadFile

.data
	.LCharacter: .ascii "a"
	.Ltmp64: .byte 0, 0, 0, 0, 0, 0, 0, 0


.text
	

.Ltext0:
main:
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
	push rax
	push rdx
	push r8
	push r9
	sub rsp, 32
	mov rcx, -11
	call GetStdHandle
	mov rcx, rax
	lea rdx, [rip + .LCharacter]
	mov r8, 1
	lea r9, [rip + .Ltmp64]
	mov QWORD PTR [rsp + 32], 0
	call WriteFile
	add rsp, 32
	pop r9
	pop r8
	pop rdx
	pop rax
	pop rcx
	ret


readchar:
	push rcx
	push rax
	push rdx
	push r8
	push r9
	sub rsp, 32
	mov rcx, -10
	call GetStdHandle
	mov rcx, rax
	lea rdx, [rip + .LCharacter]
	mov r8, 1
	lea r9, [rip + .Ltmp64]
	mov QWORD PTR [rsp + 32], 0
	call ReadFile
	add rsp, 32
	pop r9
	pop r8
	pop rdx
	pop rax
	pop rcx
	ret
