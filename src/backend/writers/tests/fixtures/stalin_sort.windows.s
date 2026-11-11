.intel_syntax noprefix
.global stalinSort

.extern GetStdHandle
.extern WriteFile
.extern ReadFile

.data
	.LCharacter: .ascii "a"
	.Ltmp64: .byte 0, 0, 0, 0, 0, 0, 0, 0


.text
	

.Ltext0:
stalinSort:
	xor r8, r8
	xor r9, r9
	mov ecx, [rdi]
	mov rax, rdi
	add rdi, 0x4
	add rax, 0x4
	add r8, 1
	add r9, 1
	.LConfusedStonks_0: 
	cmp r8, [rsi]
	je .LSamePicture_0
	.LUpgradeMarker_0:
	mov r10d, [rdi]
	cmp r10d, ecx
	jg .Lr10dWins_0
	jl .LecxWins_0
	.Lr10dWins_0:
	mov [rax], r10d
	mov ecx, r10d
	add rax, 0x4
	add r9, 1
	.LecxWins_0:
	add rdi, 0x4
	add r8, 1
	cmp r8, [rsi]
	je .LSamePicture_0
	jmp .LUpgradeMarker_0
	.LSamePicture_0:
	mov [rsi], r9
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
