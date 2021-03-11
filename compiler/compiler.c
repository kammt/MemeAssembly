#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

#include "translate.h" //Translation to Assembly
#include "analyse.h" //Semantic analysis

#include <sys/types.h>
#include <unistd.h>

#define RED   "\x1B[31m"
#define GRN   "\x1B[32m"
#define YEL   "\x1B[33m"
#define BLU   "\x1B[34m"
#define MAG   "\x1B[35m"
#define CYN   "\x1B[36m"
#define WHT   "\x1B[37m"
#define RESET "\x1B[0m"

/**
 * Attempts to convert the source file to an x86 Assembly file
 * @param srcPTR a pointer to the source file to be compiled
 * @param destPTR a pointer to the destination file. If nonexistent, it will be created
 */
void compile(FILE *srcPTR, FILE *destPTR) {
    startTranslation(srcPTR, destPTR);
    startSemanticAnalysis();

    //The file pointer now points at the end of the source file. We now reset it
    rewind(srcPTR);

    printf(GRN "File compiled successfully!" RESET);
    printf("\n");
}

/**
 * Compiles, links and runs the specified memeasm-file
 * @param srcPTR a pointer to the source file to be compiled
 */
void compileAndRun(FILE *srcPTR) {
    FILE *destPTR = fopen("tmp.asm","w");
    compile(srcPTR, destPTR);

    printf(YEL "Calling nasm...\n" RESET);
    fflush( stdout );
    system("nasm -f elf32 -o tmp.o tmp.asm");

    printf(YEL "Linking...\n" RESET);
    fflush( stdout );
    system("gcc tmp.o -g -o tmp -m32 -fno-pie -no-pie");
    
    printf(YEL "Running file...\n" RESET);
    fflush( stdout );
    execv ("tmp", NULL);
    //TODO delete temporary files
}