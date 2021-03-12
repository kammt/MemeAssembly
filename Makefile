all: 
	gcc -o memeasm compiler/memeasm.c compiler/compiler.c compiler/analyser.c compiler/translator.c compiler/log.c compiler/preprocessor.c

debug: 
	gcc -g -o memeasm compiler/memeasm.c compiler/compiler.c compiler/analyser.c compiler/translator.c compiler/log.c compiler/preprocessor.c

clean: 
	  $(RM) memeasm	

uninstall: 
	  $(RM) /usr/bin/memeasm	  

install: 
	gcc -o /usr/bin/memeasm compiler/memeasm.c compiler/compiler.c compiler/analyser.c compiler/translator.c compiler/log.c compiler/preprocessor.c