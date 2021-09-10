/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021 Tobias Kamm and contributors

MemeAssembly is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

MemeAssembly is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with MemeAssembly. If not, see <https://www.gnu.org/licenses/>.
*/

#include "compiler.h"
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


int compileMode = 2; //2 = Create Executable, 1 = Create Object File, 0 = Compile only
extern int compilationErrors;

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
            .allowedParamTypes = {0b10001},
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
            .allowedParamTypes = {0b1111, 0b111111},
            .translationPattern = "and 0, 1"
        },
        {
            .pattern = "p \\s",
            .usedParameters = 1,
            .analysisFunction = NULL,
            .allowedParamTypes = {0b1111},
            .translationPattern = "not 0"
        },

        ///Register Manipulation
        {
            .pattern = "sneak 100 p",
            .usedParameters = 1,
            .allowedParamTypes = {0b1111},
            .analysisFunction = NULL,
            .translationPattern = "xor 0, 0"
        },
        {
            .pattern = "p is brilliant, but I like p",
            .usedParameters = 2,
            .allowedParamTypes = {0b1111, 0b111111},
            .analysisFunction = NULL,
            .translationPattern = "mov 0, 1"
        },

        ///Arithmetic operations
        {
            .pattern = "upvote p",
            .usedParameters = 1,
            .allowedParamTypes = {0b1111},
            .analysisFunction = NULL,
            .translationPattern = "add 0, 1"
        },
        {
            .pattern = "downvote p",
            .usedParameters = 1,
            .allowedParamTypes = {0b1111},
            .analysisFunction = NULL,
            .translationPattern = "sub 0, 1"
        },
        {
            .pattern = "parry p you filthy casual p",
            .usedParameters = 2,
            .allowedParamTypes = {0b111111, 0b1111},
            .analysisFunction = NULL,
            .translationPattern = "sub 1, 0"
        },
        {
            .pattern = "p units are ready, with p more well on the way",
            .usedParameters = 2,
            .allowedParamTypes = {0b1111, 0b111111},
            .analysisFunction = NULL,
            .translationPattern = "add 0, 1"
        },
        {
            .pattern = "upgrades, people. Upgrades p",
            .usedParameters = 1,
            .allowedParamTypes = {0b1111},
            .analysisFunction = NULL,
            .translationPattern = "shl 0, 1"
        },
        {
            .pattern = "they had us in the first half, not gonna lie p",
            .usedParameters = 1,
            .allowedParamTypes = {0b1111},
            .analysisFunction = NULL,
            .translationPattern = "shr 0, 1"
        },
        {
                .pattern = "p is getting out of hand, now there are p of them",
                .usedParameters = 2,
                .allowedParamTypes = {0b11, 0b10011},
                .analysisFunction = NULL,
                .translationPattern = "imul 0, 1"
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
            .translationPattern = "jmp .L0"
        },
        {
            .pattern = "who would win? p or p",
            .usedParameters = 2,
            .allowedParamTypes = {0b1111, 0b111111},
            .analysisFunction = &checkWhoWouldWinValidity,
            .translationPattern = "cmp 0, 1\n\tjg .L0Wins\n\tjl .L1Wins"
        },
        {
            .pattern = "p wins",
            .usedParameters = 1,
            .allowedParamTypes = {0b111111},
            .analysisFunction = NULL,
            .translationPattern = ".L0Wins:"
        },
        {
            .pattern = "corporate needs you to find the difference between p and p",
            .usedParameters = 2,
            .allowedParamTypes = {0b1111, 0b111111},
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
            .allowedParamTypes = {0b101000},
            .translationPattern = "mov BYTE PTR [rip + .LCharacter], 0\n\tcall writechar"
        },
        {
            .pattern = "let me in. LET ME IIIIIIIIN p",
            .usedParameters = 1,
            .analysisFunction = NULL,
            .allowedParamTypes = {0b1000},
            .translationPattern = "call readchar\n\tmov 0, BYTE PTR [rip + .LCharacter]"
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
        {
            .pattern = "wait, that's illegal",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "xor rbx, rbx\n\txor rbp, rbp\n\txor r12, r12\n\txor r13 r13"
        },

        ///Debug commands
        {
            .pattern = "it's a trap",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "int3"
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

void freeCommandsArray(struct commandsArray *commands) {
    printDebugMessage("Freeing memory", "");
    for(size_t i = 0; i < commands -> size; i++) {
        struct parsedCommand parsedCommand = *(commands -> arrayPointer + i);
        for(size_t j = 0; j < commandList[parsedCommand.opcode].usedParameters; j++) {
            free(parsedCommand.parameters[j]);
        }
    }
    free(commands -> arrayPointer);

    printDebugMessage("All memory freed, compilation done", "");
}

/**
 * Parses the provided source file, converts it into a commandsArray struct and runs all the required semantic analysis functions
 * @param srcPTR a pointer to the input file
 * @return the commandsArray struct. Note that this struct is also returned when a compilation error occurred.
 *          compilationErrors (defined in log.c) counts the number of compilation errors.
 */
struct commandsArray compile(FILE *srcPTR) {
    printStatusMessage("Parsing input file");
    struct commandsArray commands = parseCommands(srcPTR);

    if(commands.size > 0) {
        printStatusMessage("Starting parameter check");
        for (size_t i = 0; i < commands.size; ++i) {
            checkParameters(&commands.arrayPointer[i]);
        }

        printStatusMessage("Analyzing commands");
        for(int opcode = 0; opcode < NUMBER_OF_COMMANDS - 2; opcode++) {
            if(commandList[opcode].analysisFunction != NULL) {
                commandList[opcode].analysisFunction(&commands, opcode);
            }
        }

        if(compilationErrors > 0) {
            printErrorASCII();
            fprintf(stderr, "Compilation failed with %d error(s), please check your code and try again.\n", compilationErrors);
        }
    } else {
        fprintf(stderr, "File is empty, nothing to do\n");
        compilationErrors++;
    }

    return commands;
}

/**
 * Attempts to convert the source file to an x86 Assembly file
 * @param srcPTR a pointer to the source file to be compiled
 * @param destPTR a pointer to the destination file.
 */
int createAssemblyFile(FILE *srcPTR, FILE *destPTR) {
    struct commandsArray commands = compile(srcPTR);
    if(compilationErrors == 0) {
        writeToFile(&commands, destPTR);
    }

    fclose(destPTR);
    freeCommandsArray(&commands);

    if(compilationErrors == 0) {
        exit(EXIT_SUCCESS);
    } else {
        exit(EXIT_FAILURE);
    }
}

/**
 * Compiles the specified memeasm file into an object file
 * @param srcPTR a pointer to the source file to be compiled
 * @param destFile the name of the destination file
 */
void createObjectFile(FILE *srcPTR, char *destFile) {
    const char* commandPrefix = "gcc -O -c -x assembler - -o";
    char command[strlen(commandPrefix) + strlen(destFile) + 1];
    strcpy(command, commandPrefix);
    strcat(command, destFile);

    // Pipe assembler code directly to GCC via stdin
    struct commandsArray commands = compile(srcPTR);
    int gccres = 1;
    if(compilationErrors == 0) {
        FILE *gccPTR = popen(command, "w");
        writeToFile(&commands, gccPTR);
        gccres = pclose(gccPTR);
    }

    freeCommandsArray(&commands);
    if(compilationErrors != 0 || gccres != 0) {
        exit(EXIT_FAILURE);
    } else {
        exit(EXIT_SUCCESS);
    }
}

/**
 * Compiles and links the specified memeasm file into an executable
 * @param srcPTR a pointer to the source file to be compiled
 * @param destFile the name of the destination file
 */
void createExecutable(FILE *srcPTR, char *destFile) {
    const char* commandPrefix = "gcc -O -no-pie -x assembler - -o";
    char command[strlen(commandPrefix) + strlen(destFile) + 1];
    strcpy(command, commandPrefix);
    strcat(command, destFile);

    // Pipe assembler code directly to GCC via stdin
    struct commandsArray commands = compile(srcPTR);
    int gccres = 1;
    if(compilationErrors == 0) {
        FILE *gccPTR = popen(command, "w");
        writeToFile(&commands, gccPTR);
        gccres = pclose(gccPTR);
    }

    freeCommandsArray(&commands);
    if(compilationErrors != 0 || gccres != 0) {
        exit(EXIT_FAILURE);
    } else {
        exit(EXIT_SUCCESS);
    }
}
