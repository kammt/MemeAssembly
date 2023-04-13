/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021-2023 Tobias Kamm and contributors

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

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <stdarg.h>

#include "parser/parser.h"
#include "analyser/analyser.h"
#include "translator/translator.h"
#include "logger/log.h"

const struct command commandList[NUMBER_OF_COMMANDS] = {
        ///Functions
        {
            .pattern = "I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun {p}",
            .usedParameters = 1,
            .allowedParamTypes = {FUNC_NAME},
            .analysisFunction = &analyseFunctions,
            .translationPattern = "{0}:"
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
            .translationPattern = "xor rax, rax\n\tret"
        },
        {
            .pattern = "{p}: whomst has summoned the almighty one",
            .usedParameters = 1,
            .allowedParamTypes = {FUNC_NAME},
            .analysisFunction = &analyseCall,
            .translationPattern = "call {0}"
        },

        ///Stack operations
        {
            .pattern = "stonks {p}",
            .usedParameters = 1,
            .allowedParamTypes = {REG64 | DECIMAL | CHAR},
            .analysisFunction = NULL,
            .translationPattern = "push {0}"
        },
        {
            .pattern = "not stonks {p}",
            .usedParameters = 1,
            .allowedParamTypes = {REG64},
            .analysisFunction = NULL,
            .translationPattern = "pop {0}"
        },

        ///Logical Operations
        {
            .pattern = "bitconneeeeeeect {p} {p}",
            .usedParameters = 2,
            .analysisFunction = NULL,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8, REG64 | REG32 | REG16 | REG8 | DECIMAL | CHAR},
            .translationPattern = "and {0}, {1}"
        },
        {
            .pattern = "{p} \\s",
            .usedParameters = 1,
            .analysisFunction = NULL,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8},
            .translationPattern = "not {0}"
        },

        ///Register Manipulation
        {
            .pattern = "sneak 100 {p}",
            .usedParameters = 1,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8},
            .analysisFunction = NULL,
            .translationPattern = "xor {0}, {0}"
        },
        {
            .pattern = "{p} is brilliant, but I like {p}",
            .usedParameters = 2,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8, REG64 | REG32 | REG16 | REG8 | DECIMAL | CHAR},
            .analysisFunction = NULL,
            .translationPattern = "mov {0}, {1}"
        },

        ///Arithmetic operations
        {
            .pattern = "upvote {p}",
            .usedParameters = 1,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8},
            .analysisFunction = NULL,
            .translationPattern = "add {0}, 1"
        },
        {
            .pattern = "downvote {p}",
            .usedParameters = 1,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8},
            .analysisFunction = NULL,
            .translationPattern = "sub {0}, 1"
        },
        {
            .pattern = "parry {p} you filthy casual {p}",
            .usedParameters = 2,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8 | DECIMAL | CHAR, REG64 | REG32 | REG16 | REG8},
            .analysisFunction = NULL,
            .translationPattern = "sub {1}, {0}"
        },
        {
            .pattern = "{p} units are ready, with {p} more well on the way",
            .usedParameters = 2,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8, REG64 | REG32 | REG16 | REG8 | DECIMAL | CHAR},
            .analysisFunction = NULL,
            .translationPattern = "add {0}, {1}"
        },
        {
            .pattern = "upgrades, people. Upgrades {p}",
            .usedParameters = 1,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8},
            .analysisFunction = NULL,
            .translationPattern = "shl {0}, 1"
        },
        {
            .pattern = "they had us in the first half, not gonna lie {p}",
            .usedParameters = 1,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8},
            .analysisFunction = NULL,
            .translationPattern = "shr {0}, 1"
        },
        {
            .pattern = "{p} is getting out of hand, now there are {p} of them",
            .usedParameters = 2,
            .allowedParamTypes = {REG64 | REG32, REG64 | REG32 | DECIMAL | CHAR},
            .analysisFunction = NULL,
            .translationPattern = "imul {0}, {1}"
        },
        {
            .pattern = "look at what {p} needs to mimic a fraction of {p}",
            .usedParameters = 2,
            .allowedParamTypes = {REG64 | DECIMAL | CHAR, REG64},
            .analysisFunction = NULL,
            .translationPattern = "mov QWORD PTR [rip + .Ltmp64], {0}\n\t"
                              "push rdx\n\t"
                              "cqo\n\t"
                              "push rax\n\t"
                              "mov rax, {1}\n\t"
                              "idiv QWORD PTR [rip + .Ltmp64]\n\t"
                              "push rax\n\t"
                              "mov rax, [rsp + 8]\n\t"
                              "pop {1}\n\t"
                              "add rsp, 8\n\t"
                              "pop rdx\n\t"
        },
        {
            .pattern = "{p} UNLIMITED POWER {p}",
            .usedParameters = 2,
            .allowedParamTypes = {REG64, REG64 | DECIMAL | CHAR},
            .analysisFunction = NULL,
            .translationPattern = "mov QWORD PTR [rip + .Ltmp64], {1}\n\t"
                              "cmp QWORD PTR [rip + .Ltmp64], 0\n\t"
                              "jne 2f\n\t" //Jump forward to 2 if not zero
                              //y is zero, load 1 and jump to the end (numeric label 4)
                              "xor {0}, {0}\n\t"
                              "inc {0}\n\t"
                              "jmp 4f\n\t"
                              //Now loop until our y is zero
                              "2: push {0}\n\t" //Preparation: push x to the stack to remember it for later
                              "dec QWORD PTR [rip + .Ltmp64]\n\t"
                              "3: imul {0}, [rsp]\n\t"
                              "dec QWORD PTR [rip + .Ltmp64]\n\t"
                              "jnz 3b\n\t" //If the result was not zero (ZF set from subtraction), jump back
                              "add rsp, 8\n\t"
                              "4:\n\t"
        },


        ///Jumps and Jump Markers
        {
            .pattern = "upgrade",
            .usedParameters = 0,
            .analysisFunction = &analyseJumpMarkers,
            .translationPattern = ".LUpgradeMarker_{F}:"
        },
        {
            .pattern = "fuck go back",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "jmp .LUpgradeMarker_{F}"
        },
        {
            .pattern = "banana",
            .usedParameters = 0,
            .analysisFunction = &analyseJumpMarkers,
            .translationPattern = ".LBananaMarker_{F}:"
        },
        {
            .pattern = "where banana",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "jmp .LBananaMarker_{F}"
        },
        {
            .pattern = "monke {p}",
            .usedParameters = 1,
            .allowedParamTypes = {MONKE_LABEL},
            .analysisFunction = &analyseMonkeMarkers,
            .translationPattern = ".L{0}:"
        },
        {
            .pattern = "return to monke {p}",
            .usedParameters = 1,
            .allowedParamTypes = {MONKE_LABEL},
            .analysisFunction = NULL,
            .translationPattern = "jmp .L{0}"
        },
        {
            .pattern = "who would win? {p} or {p}",
            .usedParameters = 2,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8, REG64 | REG32 | REG16 | REG8 | DECIMAL | CHAR},
            .analysisFunction = &analyseWhoWouldWinCommands,
            .translationPattern = "cmp {0}, {1}\n\tjg .L{0}Wins_{F}\n\tjl .L{1}Wins_{F}"
        },
        {
            .pattern = "{p} wins",
            .usedParameters = 1,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8 | DECIMAL | CHAR},
            .analysisFunction = NULL,
            .translationPattern = ".L{0}Wins_{F}:"
        },
        {
            .pattern = "corporate needs you to find the difference between {p} and {p}",
            .usedParameters = 2,
            .allowedParamTypes = {REG64 | REG32 | REG16 | REG8, REG64 | REG32 | REG16 | REG8 | DECIMAL | CHAR},
            .analysisFunction = &analyseTheyreTheSamePictureCommands,
            .translationPattern = "cmp {0}, {1}\n\tje .LSamePicture_{F}"
        },
        {
            .pattern = "they're the same picture",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = ".LSamePicture_{F}:"
        },

        ///IO-Operations
        {
            .pattern = "what can I say except {p}",
            .usedParameters = 1,
            .analysisFunction = NULL,
            .allowedParamTypes = {REG8 | CHAR},
            .translationPattern = "mov BYTE PTR [rip + .LCharacter], {0}\n\t"
                                  "test rsp, 0xF\n\t"
                                  "jz 1f\n\t"
                                  "sub rsp, 8\n\t"
                                  "call writechar\n\t"
                                  "add rsp, 8\n\t"
                                  "jmp 2f\n\t"
                                  "1: call writechar\n\t"
                                  "2:\n\t"
        },
        {
            .pattern = "let me in. LET ME IIIIIIIIN {p}",
            .usedParameters = 1,
            .analysisFunction = NULL,
            .allowedParamTypes = {REG8},
            .translationPattern = "test rsp, 0xF\n\t"
                                  "jz 1f\n\t"
                                  "sub rsp, 8\n\t"
                                  "call readchar\n\t"
                                  "add rsp, 8\n\t"
                                  "jmp 2f\n\t"
                                  "1: call readchar\n\t"
                                  "2:\n\t"
                                  "mov {0}, BYTE PTR [rip + .LCharacter]\n\t"
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
            .translationPattern = "jmp .LConfusedStonks_{F}:"
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
        {
            .pattern = "oh no! anyway",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "nop"
        },
        {
            .pattern = "we need air support",
            .usedParameters = 0,
            .analysisFunction = NULL,
            .translationPattern = "syscall"
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



/**
 *
 * @param compileState a struct containing all necessary infos. Most notably, it contains the compileMode, optimisation level and all parsed input files
 * @param outputFileName the name of the output file
 */
void compile(struct compileState compileState, char* outputFileName) {
    ///Analysis
    analyseCommands(&compileState);

    //Analysis done. If any errors occurred until now, print to stderr and exit
    if(compileState.compilerErrors > 0) {
        printErrorASCII();
        fprintf(stderr, "Compilation failed with %u error(s), please check your code and try again.\n", compileState.compilerErrors);
        exit(EXIT_FAILURE);
    }

    ///Translation
    FILE* output;
    int gccResult = 0;
    //When generating an assembly file, we open the output file in writing mode directly
    if(compileState.compileMode == assemblyFile) {
        output = fopen(outputFileName, "w") ;
        if(output == NULL) {
            perror("Failed to open output file");
            exit(EXIT_FAILURE);
        }
    //When letting gcc do the work for us (object file or executable), we just pipe the code into gcc via stdin
    } else {
        char* commandPrefix;
        if(compileState.compileMode == objectFile) {
            commandPrefix = "gcc -O -c -x assembler - -o";
        } else {
            #ifndef LINUX
            commandPrefix = "gcc -O -x assembler - -o";
            #else
            commandPrefix = "gcc -O -no-pie -x assembler - -o"; //-no-pie is only defined because for some reason, the generated stabs info does not work when a PIE object is generated
            #endif
        }

        char command[strlen(commandPrefix) + strlen(outputFileName) + 1];
        strcpy(command, commandPrefix);
        strcat(command, outputFileName);

        // Pipe assembler code directly to GCC via stdin
        output = popen(command, "w");
    }

    writeToFile(&compileState, output);

    if(compileState.compileMode == assemblyFile) {
        fclose(output);
    } else {
        gccResult = pclose(output);
    }

    if(gccResult != 0) {
        fprintf(stderr, "gcc exited unexpectedly with exit code %d. If you did not expect this to happen, please report this issue at https://github.com/kammt/MemeAssembly/issues so that it can be fixed\n", gccResult);
        exit(EXIT_FAILURE);
    } else {
        exit(EXIT_SUCCESS);
    }
}
