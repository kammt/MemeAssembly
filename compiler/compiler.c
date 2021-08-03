#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

#include "commands.h"

#include "parser/parser.h"
#include "analyzer/parameters.h"
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
            .usedParameters = 1,
            .allowedParamTypes = {0b10000000},
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
            .usedParameters = 1,
            .allowedParamTypes = {0b11001},
            .translationPattern = "push 0"
        },
        {
            .pattern = "not stonks p",
            .usedParameters = 1,
            .allowedParamTypes = {0b1},
            .translationPattern = "pop 0"
        },

        ///Logical Operations
        {
            .pattern = "bitconneeeeeeect p p",
            .usedParameters = 2,
            .allowedParamTypes = {0b111, 0b11111}
        },

        ///Register Manipulation
        {
            .pattern = "sneak 100 p",
            .usedParameters = 1,
            .allowedParamTypes = {0b111},
            .translationPattern = "xor 0, 0"
        },
        {
            .pattern = "p is brilliant, but I like p",
            .usedParameters = 2,
            .allowedParamTypes = {0b111, 0b11111},
            .translationPattern = "mov 0, 1"
        },

        ///Arithmetic operations
        {
            .pattern = "upvote p",
            .usedParameters = 1,
            .allowedParamTypes = {0b111},
            .translationPattern = "add 0, 1"
        },
        {
            .pattern = "downvote p",
            .usedParameters = 1,
            .allowedParamTypes = {0b111},
            .translationPattern = "sub 0, 1"
        },
        {
            .pattern = "parry p you filthy casual p",
            .usedParameters = 2,
            .allowedParamTypes = {0b111111, 0b111},
            .translationPattern = "sub 1, 0"
        },
        {
            .pattern = "p units are ready, with p more well on the way",
            .usedParameters = 2,
            .allowedParamTypes = {0b111, 0b111111},
            .translationPattern = "sub 0, 1"
        },
        {
            .pattern = "upgrades, people. Upgrades p",
            .usedParameters = 1,
            .allowedParamTypes = {0b111},
            .translationPattern = "shl 0, 1"
        },
        {
            .pattern = "they had us in the first half, not gonna lie p",
            .usedParameters = 1,
            .allowedParamTypes = {0b111},
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
            .usedParameters = 1,
            .allowedParamTypes = {0b1000000},
            .translationPattern = ".L0:"
        },
        {
            .pattern = "return to monke p",
            .usedParameters = 1,
            .allowedParamTypes = {0b1000000},
            .translationPattern = "jmp .L0:"
        },
        {
            .pattern = "who would win? p or p",
            .usedParameters = 2,
            .allowedParamTypes = {0b111, 0b11111},
            .translationPattern = "cmp 0, 1\n\tjg .L0Wins\n\tjmp .L1Wins"
        },
        {
            .pattern = "p wins",
            .usedParameters = 1,
            .allowedParamTypes = {0b11111},
            .translationPattern = ".L0Wins:"
        },
        {
            .pattern = "they're the same picture",
            .usedParameters = 0,
            .translationPattern = ".LSamePicture:"
        },
        {
            .pattern = "corporate needs you to find the difference between p and p",
            .usedParameters = 2,
            .allowedParamTypes = {0b111, 0b11111},
            .translationPattern = "cmp 0, 1\n\tje .LSamePicture"
        },

        ///IO-Operations
        {
            .pattern = "what can I say except p",
            .usedParameters = 1,
            .allowedParamTypes = {0b10100}
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
    for (int i = 0; i < commands.size; ++i) {
        printf("Parameter Check returned %d\n", hasValidParameters(&commands.arrayPointer[i]));
    }
    /*
    printInfoMessage("Starting Assembly-Translation...");
    startTranslation(file, numberOfLines, opcodes, destPTR);
    
    printInfoMessage("Starting semantic analysis...");
    startSemanticAnalysis(opcodes, numberOfLines);

    printSuccessMessage("File compiled successfully!");
     */
    for(int i = 0; i < commands.size; i++) {
        struct parsedCommand parsedCommand = *(commands.arrayPointer + i);
        for(int j = 0; j < commandList[parsedCommand.opcode].usedParameters; j++) {
            free(parsedCommand.parameters[j]);
        }
    }
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