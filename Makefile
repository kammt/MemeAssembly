all: 
	gcc -o memeasm compiler/memeasm.c compiler/compiler.c

debug: 
	gcc -g -o memeasm compiler/memeasm.c compiler/compiler.c