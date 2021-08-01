#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

#include "commands.h"

#include "parser.h" //String functions
#include "translator.h" //Translation to Assembly
#include "analyser.h" //Semantic analysis
#include "logger/log.h" //Writing to the command line with log levels

/**
 * compiler.c:
 * This file simply provides the functions compile and compileAndRun. The main functionality of these functions is implemented in translate.c and analyse.c
 */

struct command commandList[NUMBER_OF_COMMANDS] = {
        ///Functions
        {
            .pattern = "I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun p",
            .usesPointer = 1,
            .translationPattern = "0:"
        },
        {
            .pattern = "right back at ya, buckaroo",
            .usedParameters = 0,
            .translationPattern = "ret"
        },
        {
            .pattern = "no, I don't think I will",
            .usedParameters = 0,
            .translationPattern = "mov rax, 1\n\tret"
        },
        {
            .pattern = "I see this as an absolute win",
            .usedParameters = 0,
            .translationPattern = "mov rax, 0\n\tret"
        },
        //TODO "ah shit, here we go again",

        ///Stack operations
        {
            .pattern = "stonks p",
            .usesPointer = 0,
            .usedParameters = 1,
            .allowedTypesParam1 = 0b11001,
            .translationPattern = "push 0"
        },
        {
            .pattern = "not stonks p",
            .usesPointer = 0,
            .usedParameters = 1,
            .allowedTypesParam1 = 0b1,
            .translationPattern = "pop 0"
        },

        ///Logical Operations
        {
            .pattern = "bitconneeeeeeect p p",
            .usesPointer = 0,
            .usedParameters = 2,
            .allowedTypesParam1 = 0b111,
            .allowedTypesParam2 = 0b11111,
        },

        ///Register Manipulation
        {
            .pattern = "sneak 100 p",
            .usesPointer = 0,
            .usedParameters = 1,
            .allowedTypesParam1 = 0b111,
            .translationPattern = "xor 0, 0"
        },
        {
            .pattern = "p is brilliant, but I like p",
            .usesPointer = 0,
            .usedParameters = 2,
            .allowedTypesParam1 = 0b111,
            .allowedTypesParam2 = 0b11111,
            .translationPattern = "mov 0, 1"
        },

        ///Arithmetic operations
        {
            .pattern = "upvote p",
            .usesPointer = 0,
            .usedParameters = 1,
            .allowedTypesParam1 = 0b111,
            .translationPattern = "add 0, 1"
        },
        {
            .pattern = "downvote p",
            .usesPointer = 0,
            .usedParameters = 1,
            .allowedTypesParam1 = 0b111,
            .translationPattern = "sub 0, 1"
        },
        {
            .pattern = "parry p you filthy casual p",
            .usedParameters = 2,
            .usesPointer = 0,
            .allowedTypesParam1 = 0b111111,
            .allowedTypesParam2 = 0b111,
            .translationPattern = "sub 1, 0"
        },
        {
            .pattern = "p units are ready, with p more well on the way",
            .usedParameters = 2,
            .usesPointer = 0,
            .allowedTypesParam1 = 0b111,
            .allowedTypesParam2 = 0b111111,
            .translationPattern = "sub 0, 1"
        },
        {
            .pattern = "upgrades, people. Upgrades p",
            .usesPointer = 0,
            .usedParameters = 1,
            .allowedTypesParam1 = 0b111,
            .translationPattern = "shl 0, 1"
        },
        {
            .pattern = "they had us in the first half, not gonna lie p",
            .usesPointer = 0,
            .usedParameters = 1,
            .allowedTypesParam1 = 0b111,
            .translationPattern = "shr 0, 1"
        },


        ///Jumps and Jump Markers
        {
            .pattern = "upgrade",
            .usedParameters = 0,
            .translationPattern = ".LUpgradeMarker:"
        },
        {
            .pattern = "fuck go back",
            .usedParameters = 0,
            .translationPattern = "jmp .LUpgradeMarker"
        },
        {
            .pattern = "banana",
            .usedParameters = 0,
            .translationPattern = ".LBananaMarker:"
        },
        {
            .pattern = "where banana",
            .usedParameters = 0,
            .translationPattern = "jmp .LBananaMarker"
        },
        {
            .pattern = "monke p",
            .usesPointer = 1,
            .translationPattern = ".L0:"
        },
        {
            .pattern = "return to monke p",
            .usesPointer = 1,
            .translationPattern = "jmp .L0:"
        },
        {
            .pattern = "who would win? p or p",
            .usesPointer = 0,
            .usedParameters = 2,
            .allowedTypesParam1 = 0b111,
            .allowedTypesParam2 = 0b11111,
            .translationPattern = "cmp 0, 1\n\tjg .L0Wins\n\tjmp .L1Wins"
        },
        {
            .pattern = "they're the same picture",
            .usedParameters = 0,
            .translationPattern = ".LSamePicture:"
        },
        {
            .pattern = "corporate needs you to find the difference between p and p",
            .usedParameters = 2,
            .usesPointer = 0,
            .allowedTypesParam1 = 0b111,
            .allowedTypesParam2 = 0b11111,
            .translationPattern = "cmp 0, 1\n\tje .LSamePicture"
        },

        ///IO-Operations
        {
            .pattern = "what can I say except p",
            .usesPointer = 0,
            .usedParameters = 1,
            .allowedTypesParam1 = 0b10100
        },


        ///Random commands
        {
            .pattern = "guess I'll die",
            .usedParameters = 0,
            .translationPattern = "mov rax, [69]"
        },
        {
            .pattern = "confused stonks",
            .usedParameters = 0,
            .translationPattern = "jmp .LConfusedStonks:"
        },
        {
            .pattern = "perfectly balanced as all things should be",
            .usedParameters = 0,
            .translationPattern = ""
        },
        //Insert commands above this one
        {
            .pattern = "or draw 25",
            .usedParameters = 0,
            .translationPattern = "add eax, 25"
        }
};

/**
 * Attempts to convert the source file to an x86 Assembly file
 * @param srcPTR a pointer to the source file to be compiled
 * @param destPTR a pointer to the destination file. If nonexistent, it will be created
 */
void compile(FILE *srcPTR, FILE *destPTR) {
    struct commandsArray commands = parseCommands(srcPTR);

    for(int i = 0; i < commands.size; i++) {
        struct parsedCommand parsedCommand = *(commands.arrayPointer + i);
        printf("---------------\n\topcode = %d\n\tparameter1 = %s\n\tparameter2 = %s\n\tparameter%d is a pointer\n\tcommand was found in line %d\n", parsedCommand.opcode, parsedCommand.parameters.paramsArray.params[0], parsedCommand.parameters.paramsArray.params[1], parsedCommand.isPointer, parsedCommand.lineNum);
    }
    /*
    printInfoMessage("Starting Assembly-Translation...");
    startTranslation(file, numberOfLines, opcodes, destPTR);
    
    printInfoMessage("Starting semantic analysis...");
    startSemanticAnalysis(opcodes, numberOfLines);

    printSuccessMessage("File compiled successfully!");
     */
    free(commands.arrayPointer);
}

/**
 * Compiles, links and runs the specified memeasm-file
 * @param srcPTR a pointer to the source file to be compiled
 */
void createExecutable(FILE *srcPTR) {
    FILE *destPTR = fopen("tmp.asm","w");
    compile(srcPTR, destPTR);

    printStatusMessage("Calling nasm...");
    system("nasm -f elf32 -o tmp.o tmp.asm");

    printStatusMessage("Linking...");
    system("gcc tmp.o -g -o tmp -m32 -fno-pie -no-pie");

    //TODO delete temporary files
    exit(EXIT_SUCCESS);
}