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
	mov BYTE PTR [rip + .LCharacter], 'h'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	xor ecx, ecx
	.LUpgradeMarker_0:
	mov eax, 0x74
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	add ecx, 1
	cmp ecx, 0x2
	je .LSamePicture_0
	.LConfusedStonks_0: 
	jmp .LUpgradeMarker_0
	.LSamePicture_0:
	sub eax, 0x4
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 's'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 58
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	sub eax, 0x41
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	push rax
	xor eax, eax
	sub eax, 0x87
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	sub eax, 0xA
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	push rax
	add eax, 1
	add eax, 1
	add eax, 1
	add eax, 1
	add eax, 1
	add eax, 1
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	push rax
	sub eax, 1
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	pop rax
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov eax, 0x65
	push rax
	add eax, 1
	and eax, 0xFFFFFFFA
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	pop rax
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], '.'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'c'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	pop rax
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	and eax, 0xFD
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	pop rax
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'w'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 97
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	xor ebx, ebx
	sub ebx, 0x28C
	mov BYTE PTR [rip + .LCharacter], bl
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov edx, ebx
	sub edx, 0x11
	mov BYTE PTR [rip + .LCharacter], dl
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	sub ebx, 0xC
	mov BYTE PTR [rip + .LCharacter], bl
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], '?'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 118
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	sub eax, 0xF2
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov eax, 0x64
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	sub eax, 0x13
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	push rax
	add eax, 25
	add eax, 25
	sub eax, 0xC
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	push rax
	mov BYTE PTR [rip + .LCharacter], '4'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	pop rax
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], '9'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	pop rax
	push rax
	add eax, 1
	add eax, 1
	add eax, 1
	add eax, 1
	add eax, 1
	add eax, 1
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	sub ebx, 1
	mov BYTE PTR [rip + .LCharacter], bl
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	sub ebx, 0xF
	mov BYTE PTR [rip + .LCharacter], bl
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	mov BYTE PTR [rip + .LCharacter], 'c'
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
	pop rax
	mov BYTE PTR [rip + .LCharacter], al
	test rsp, 0xF
	jz 1f
	sub rsp, 8
	call writechar
	add rsp, 8
	jmp 2f
	1: call writechar
	2:
	
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
