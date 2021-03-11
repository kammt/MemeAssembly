all: 
	gcc -o memeasm compiler/memeasm.c compiler/compiler.c compiler/analyse.c compiler/translate.c compiler/log.c

debug: 
	gcc -g -o memeasm compiler/memeasm.c compiler/compiler.c compiler/analyse.c compiler/translate.c compiler/log.c

clean: 
	  $(RM) memeasm	

uninstall: 
	  $(RM) /bin/memeasm	  

install: 
	gcc -o /bin/memeasm compiler/memeasm.c compiler/compiler.c compiler/analyse.c compiler/translate.c compiler/log.c	   