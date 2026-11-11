.intel_syntax noprefix
.global rot13

.extern GetStdHandle
.extern WriteFile
.extern ReadFile

.data
	.LCharacter: .ascii "a"
	.Ltmp64: .byte 0, 0, 0, 0, 0, 0, 0, 0


.text
	

.Ltext0:
rot13:
	push rdi
	.LUpgradeMarker_0:
	xor edx, edx
	mov dl, [rdi]
	cmp edx, 0x1
	jg .LedxWins_0
	jl .L0x1Wins_0
	.LedxWins_0:
	mov eax, 0x61
	.LBananaMarker_0:
	.LConfusedStonks_0: 
	cmp dl, al
	je .LSamePicture_0
	add eax, 1
	cmp eax, 0x7A
	jg .LeaxWins_0
	jl .L0x7AWins_0
	.L0x7AWins_0:
	jmp .LBananaMarker_0
	.LSamePicture_0:
	add dx, 0xD
	cmp dx, 0x7B
	jg .LdxWins_0
	jl .L0x7BWins_0
	.LdxWins_0:
	.Luauauaa:
	sub dx, 0x1A
	.L0x7BWins_0:
	.L0x5BWins_0:
	mov [rdi], dl
	add rdi, 1
	jmp .LUpgradeMarker_0
	.LeaxWins_0:
	mov ecx, 0x41
	.Luaaua:
	cmp dl, cl
	jg .LdlWins_0
	jl .LclWins_0
	add dx, 0xD
	cmp dx, 0x5B
	jg .LdxWins_0
	jl .L0x5BWins_0
	jmp .Luauauaa
	.LdlWins_0:
	.LclWins_0:
	add ecx, 1
	cmp ecx, 0x5A
	jg .LecxWins_0
	jl .L0x5AWins_0
	.L0x5AWins_0:
	jmp .Luaaua
	.LecxWins_0:
	add rdi, 1
	jmp .LUpgradeMarker_0
	.L0x1Wins_0:
	pop rdi
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
