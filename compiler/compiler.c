#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

#include "preprocessor.h" //Semantic analysis
#include "translator.h" //Translation to Assembly
#include "analyser.h" //Semantic analysis
#include "log.h" //Writing to the command line with log levels

#include <sys/types.h>
#include <unistd.h>

/**
 * compiler.c:
 * This file simply provides the functions compile and compileAndRun. The main functionality of these functions is implemented in translate.c and analyse.c
 */


/**
 * Attempts to convert the source file to an x86 Assembly file
 * @param srcPTR a pointer to the source file to be compiled
 * @param destPTR a pointer to the destination file. If nonexistent, it will be created
 */
void compile(FILE *srcPTR, FILE *destPTR) {
    //First, we create an array with the size of our number of lines. For that, we first count the number of lines
    char line[128];
    int numberOfLines = 0;
    while(fgets(line, sizeof(line), srcPTR) != NULL) {
        numberOfLines++;
    }
    //Done, now we create the array
    char file[numberOfLines][128];
    int opcodes[numberOfLines];
    
    printDebugMessage("Array created, rewinding source pointer...", "");
    rewind(srcPTR);

    printInfoMessage("Starting pre-translation processing...");
    preprocess(file, srcPTR);

    printInfoMessage("Starting Assembly-Translation...");
    startTranslation(file, numberOfLines, opcodes, destPTR);
    
    printInfoMessage("Starting semantic analysis...");
    //startSemanticAnalysis();

    printSuccessMessage("File compiled successfully!");
}

/**
 * Compiles, links and runs the specified memeasm-file
 * @param srcPTR a pointer to the source file to be compiled
 */
void compileAndRun(FILE *srcPTR) {
    FILE *destPTR = fopen("tmp.asm","w");
    compile(srcPTR, destPTR);

    printStatusMessage("Calling nasm...");
    system("nasm -f elf32 -o tmp.o tmp.asm");

    printStatusMessage("Linking...");
    system("gcc tmp.o -g -o tmp -m32 -fno-pie -no-pie");
    
    printStatusMessage("Running file...");
    execv ("tmp", NULL);
    //TODO delete temporary files
}