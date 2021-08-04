#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

#include "commands.h"

#include "parser/parser.h"
#include "analyzer/parameters.h"
#include "analyzer/functions.h"
#include "analyzer/jumpLabels.h"
#include "analyzer/comparisons.h"
#include "analyzer/randomCommands.h"
#include "translator/translator.h"
#include "logger/log.h"

/**
 * compiler.c:
 * This file simply provides the functions compile and compileAndRun. The main functionality of these functions is implemented in translate.c and analyse.c
 */

int compileMode = 1; //1 = Create Executable, 0 = Compile only

struct command commandList[NUMBER_OF_COMMANDS] = {
        ///Functions
        {
            .pattern = "I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun p",
            .usedParameters = 1,
            .allowedParamTypes = {0b10000000},
            .analysisFunction = &checkFunctionValidity,
            .translationPattern = "0:"
        },
        {
            .pattern = "right back at ya, buckaroo",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "ret"
        },
        {
            .pattern = "no, I don't think I will",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "mov rax, 1\n\tret"
        },
        {
            .pattern = "I see this as an absolute win",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "mov rax, 0\n\tret"
        },

        ///Stack operations
        {
            .pattern = "stonks p",
            .usedParameters = 1,
            .allowedParamTypes = {0b11001},
            .analysisFunction = NULL,
            .translationPattern = "push 0"
        },
        {
            .pattern = "not stonks p",
            .usedParameters = 1,
            .allowedParamTypes = {0b1},
            .analysisFunction = NULL,
            .translationPattern = "pop 0"
        },

        ///Logical Operations
        {
            .pattern = "bitconneeeeeeect p p",
            .usedParameters = 2,
            .analysisFunction = NULL,
            .allowedParamTypes = {0b111, 0b11111},
            .translationPattern = "and 0, 1"
        },

        ///Register Manipulation
        {
            .pattern = "sneak 100 p",
            .usedParameters = 1,
            .allowedParamTypes = {0b111},
            .analysisFunction = NULL,
            .translationPattern = "xor 0, 0"
        },
        {
            .pattern = "p is brilliant, but I like p",
            .usedParameters = 2,
            .allowedParamTypes = {0b111, 0b11111},
            .analysisFunction = NULL,
            .translationPattern = "mov 0, 1"
        },

        ///Arithmetic operations
        {
            .pattern = "upvote p",
            .usedParameters = 1,
            .allowedParamTypes = {0b111},
            .analysisFunction = NULL,
            .translationPattern = "add 0, 1"
        },
        {
            .pattern = "downvote p",
            .usedParameters = 1,
            .allowedParamTypes = {0b111},
            .analysisFunction = NULL,
            .translationPattern = "sub 0, 1"
        },
        {
            .pattern = "parry p you filthy casual p",
            .usedParameters = 2,
            .allowedParamTypes = {0b111111, 0b111},
            .analysisFunction = NULL,
            .translationPattern = "sub 1, 0"
        },
        {
            .pattern = "p units are ready, with p more well on the way",
            .usedParameters = 2,
            .allowedParamTypes = {0b111, 0b111111},
            .analysisFunction = NULL,
            .translationPattern = "add 0, 1"
        },
        {
            .pattern = "upgrades, people. Upgrades p",
            .usedParameters = 1,
            .allowedParamTypes = {0b111},
            .analysisFunction = NULL,
            .translationPattern = "shl 0, 1"
        },
        {
            .pattern = "they had us in the first half, not gonna lie p",
            .usedParameters = 1,
            .allowedParamTypes = {0b111},
            .analysisFunction = NULL,
            .translationPattern = "shr 0, 1"
        },


        ///Jumps and Jump Markers
        {
            .pattern = "upgrade",
            .usedParameters = 0,
            .analysisFunction = &checkJumpLabelValidity,
            .translationPattern = ".LUpgradeMarker:"
        },
        {
            .pattern = "fuck go back",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "jmp .LUpgradeMarker"
        },
        {
            .pattern = "banana",
            .usedParameters = 0,
            .analysisFunction = &checkJumpLabelValidity,
            .translationPattern = ".LBananaMarker:"
        },
        {
            .pattern = "where banana",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "jmp .LBananaMarker"
        },
        {
            .pattern = "monke p",
            .usedParameters = 1,
            .allowedParamTypes = {0b1000000},
            .analysisFunction = &checkMonkeJumpLabelValidity,
            .translationPattern = ".L0:"
        },
        {
            .pattern = "return to monke p",
            .usedParameters = 1,
            .allowedParamTypes = {0b1000000},
            .analysisFunction = NULL,
            .translationPattern = "jmp .L0:"
        },
        {
            .pattern = "who would win? p or p",
            .usedParameters = 2,
            .allowedParamTypes = {0b111, 0b11111},
            .analysisFunction = &checkWhoWouldWinValidity,
            .translationPattern = "cmp 0, 1\n\tjg .L0Wins\n\tjmp .L1Wins"
        },
        {
            .pattern = "p wins",
            .usedParameters = 1,
            .allowedParamTypes = {0b11111},
            .analysisFunction = NULL,
            .translationPattern = ".L0Wins:"
        },
        {
            .pattern = "corporate needs you to find the difference between p and p",
            .usedParameters = 2,
            .allowedParamTypes = {0b111, 0b11111},
            .analysisFunction = &checkTheyreTheSamePictureValidity,
            .translationPattern = "cmp 0, 1\n\tje .LSamePicture"
        },
        {
            .pattern = "they're the same picture",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = ".LSamePicture:"
        },

        ///IO-Operations
        {
            .pattern = "what can I say except p",
            .usedParameters = 1,
            .analysisFunction = NULL,
            .allowedParamTypes = {0b110100},
            .translationPattern = "mov BYTE PTR [rip + .LCharacter], 0\n\tcall writechar"
        },


        ///Random commands
        {
            .pattern = "guess I'll die",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "mov rax, [69]"
        },
        {
            .pattern = "confused stonks",
            .usedParameters = 0,
            .analysisFunction = &setConfusedStonksJumpLabel,
            .translationPattern = "jmp .LConfusedStonks:"
        },
        {
            .pattern = "perfectly balanced as all things should be",
            .usedParameters = 0,
            .analysisFunction = &chooseLinesToBeDeleted,
            .translationPattern = ""
        },
        //Insert commands above this one
        {
            .pattern = "or draw 25",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "add eax, 25"
        },
        {
            .pattern = "",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "0"
        }
};

/**
 * Attempts to convert the source file to an x86 Assembly file
 * @param srcPTR a pointer to the source file to be compiled
 * @param destPTR a pointer to the destination file. If nonexistent, it will be created
 * @return 0 on success, 1 otherwise
 */
int compile(FILE *srcPTR, FILE *destPTR) {
    printStatusMessage("Parsing input file");
    struct commandsArray commands = parseCommands(srcPTR);

    printStatusMessage("Starting parameter check");
    for (int i = 0; i < commands.size; ++i) {
        checkParameters(&commands.arrayPointer[i]);
    }

    printStatusMessage("Analyzing commands");
    for(int opcode = 0; opcode < NUMBER_OF_COMMANDS - 2; opcode++) {
        if(commandList[opcode].analysisFunction != NULL) {
            commandList[opcode].analysisFunction(&commands, opcode);
        }
    }

    /*
    printInfoMessage("Starting Assembly-Translation...");
    startTranslation(file, numberOfLines, opcodes, destPTR);
    
    printInfoMessage("Starting semantic analysis...");
    startSemanticAnalysis(opcodes, numberOfLines);

    printSuccessMessage("File compiled successfully!");
     */
    if(getNumberOfCompilationErrors() > 0) {
        printErrorASCII();
        fprintf(stderr, "Compilation failed with %d error(s), please check your code and try again.\n", getNumberOfCompilationErrors());
    } else {
        writeToFile(&commands, destPTR);
    }

    printDebugMessage("Freeing memory", "");
    for(int i = 0; i < commands.size; i++) {
        struct parsedCommand parsedCommand = *(commands.arrayPointer + i);
        for(int j = 0; j < commandList[parsedCommand.opcode].usedParameters; j++) {
            free(parsedCommand.parameters[j]);
        }
    }
    free(commands.arrayPointer);

    printDebugMessage("All memory freed, compilation done", "");

    if(getNumberOfCompilationErrors() > 0) {
        return 1;
    }
    return 0;
}

/**
 * Compiles, links and runs the specified memeasm-file
 * @param srcPTR a pointer to the source file to be compiled
 */
void createExecutable(FILE *srcPTR, char *destFile) {
    FILE *tmpPTR = fopen("tmp.S","w");
    int result = compile(srcPTR, tmpPTR);

    if(result == 0) {
        printStatusMessage("Calling gcc");
        char commandPrefix[] = "gcc tmp.S -o ";
        size_t strLen = strlen(commandPrefix) + strlen(destFile);
        char command[strLen];
        command[0] = '\0';

        strncat(command, commandPrefix, strLen);
        strncat(command, destFile, strLen);
        system(command);

        printDebugMessage("Removing temporary file", "");
        system("rm tmp.S");
        exit(EXIT_SUCCESS);
    } else {
        exit(EXIT_FAILURE);
    }
}