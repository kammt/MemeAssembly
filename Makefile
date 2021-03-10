all: 
	gcc -o memeasm compiler/memeasm.c compiler/compiler.c

debug: 
	gcc -g -o memeasm compiler/memeasm.c compiler/compiler.c

clean: 
	  $(RM) memeasm	

uninstall: 
	  $(RM) /bin/memeasm	  

install: 
	gcc -o /bin/memeasm compiler/memeasm.c compiler/compiler.c	  