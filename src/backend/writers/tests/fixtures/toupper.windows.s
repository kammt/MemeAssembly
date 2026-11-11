.intel_syntax noprefix
.global toupper_str

.extern GetStdHandle
.extern WriteFile
.extern ReadFile

.data
	.LCharacter: .ascii "a"
	.Ltmp64: .byte 0, 0, 0, 0, 0, 0, 0, 0


.text
	

.Ltext0:
toupper_str:
	mov eax, 0x61
	.LUpgradeMarker_0:
	push rdi
	.LBananaMarker_0:
	xor edx, edx
	mov dl, [rdi]
	cmp edx, 0x1
	jg .LedxWins_0
	jl .L0x1Wins_0
	.LedxWins_0:
	cmp dl, al
	jg .LdlWins_0
	jl .LalWins_0
	and dl, 0xDF
	mov [rdi], dl
	.LdlWins_0:
	.LalWins_0:
	add rdi, 1
	jmp .LBananaMarker_0
	.L0x1Wins_0:
	pop rdi
	.LConfusedStonks_0: 
	add eax, 1
	cmp eax, 0x7B
	je .LSamePicture_0
	jmp .LUpgradeMarker_0
	.LSamePicture_0:
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
