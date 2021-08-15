section .text
global main
main:
	line_1: 
	mov esp, 0

	ret

writechar:
	push ebx
	push ecx
	push edx
	push eax
	mov edx, 1
	mov ecx, esp
	mov ebx, 1
	mov eax, 4
	int 128
	pop eax
	pop edx
	pop ecx
	pop ebx
	
	ret