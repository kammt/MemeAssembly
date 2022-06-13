/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021-2022 Tobias Kamm and contributors

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
#include <unistd.h>
#include <spawn.h>
#include <sys/wait.h>

#include "parser/parser.h"
#include "analyser/analyser.h"
#include "translator/translator.h"
#include "logger/log.h"


void blockIOCommands(struct commandLinkedList** commandLinkedList, unsigned int opcode, struct compileState* compileState) {
    struct commandLinkedList* ioCommand = commandLinkedList[opcode];
    while (ioCommand != NULL) {
        struct parsedCommand command = *ioCommand->command;

        printError(compileState->files[ioCommand->definedInFile].fileName, command.lineNum, compileState, "I/O commands are unsupported on this platform", 0);
        ioCommand = ioCommand->next;
    }
}

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
            .translationPattern = "mov QWORD PTR [rsp - 0x78], {0}\n\t"
                              "push rdx\n\t"
                              "cqo\n\t"
                              "push rax\n\t"
                              "mov rax, {1}\n\t"
                              "idiv QWORD PTR [rsp - 0x68]\n\t"
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
            .translationPattern = "mov QWORD PTR [rsp - 0x78], {1}\n\t"
                              "cmp QWORD PTR [rsp - 0x78], 0\n\t"
                              "jne 2f\n\t" //Jump forward to 2 if not zero
                              //y is zero, load 1 and jump to the end (numeric label 4)
                              "xor {0}, {0}\n\t"
                              "inc {0}\n\t"
                              "jmp 4f\n\t"
                              //Now loop until our y is zero
                              "2: push {0}\n\t" //Preparation: push x to the stack to remember it for later
                              "dec QWORD PTR [rsp - 0x70]\n\t"
                              "3: imul {0}, [rsp]\n\t"
                              "dec QWORD PTR [rsp - 0x70]\n\t"
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
            .analysisFunction = &blockIOCommands,
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
            .analysisFunction = &blockIOCommands,
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
        fprintf(stderr, "Compilation failed with %u error(s), please check your code and try again.\n", compileState.compilerErrors);
        exit(EXIT_FAILURE);
    }

    ///Translation
    FILE* output;
    int gccResult = 0;
    int gccStdin[2];
    pid_t gccPid;

    //When generating an assembly file, we open the output file in writing mode directly
    if(compileState.compileMode == assemblyFile) {
        output = fopen(outputFileName, "w") ;
        if(output == NULL) {
            perror("Failed to open output file");
            exit(EXIT_FAILURE);
        }
    //When letting gcc do the work for us (object file or executable), we just pipe the code into gcc via stdin
    } else {
        if(compileState.compileMode == objectFile) {
            fprintf(stderr, "-O unsupported on rasptest platform, defaulting to executable\n");
        }

        //We receive out arguments for gcc as a string, we now need to convert it to an array
        //1. How many args are there
        size_t argc = 0;
        for(size_t i = 0; i < strlen(compileState.gcc_args); i++) {
            if(compileState.gcc_args[i] == ' ') {
                argc++;
            }
        }
        argc++; //Arg at the end does not have a space

        char* argv[argc + 1]; //include NULL-arg
        size_t i = 0;
        //2. Clone every arg into the array
        for(compileState.gcc_args = strtok(compileState.gcc_args, " "); compileState.gcc_args != NULL; compileState.gcc_args = strtok(NULL, " ")) {
            argv[i] = strdup(compileState.gcc_args);
            if(argv[i] == NULL) {
                perror("fatal: strdup() of gcc-args failed");
                exit(EXIT_FAILURE);
            }
            i++;
        }
        //3. last arg must be NULL
        argv[i] = NULL;

        /// Pipe assembler code directly to GCC via stdin
        //Set up pipes for stdin
        if(pipe(gccStdin) < 0)
            perror("pipe");


        posix_spawn_file_actions_t action;
        posix_spawn_file_actions_init(&action);
        //stdin
        posix_spawn_file_actions_adddup2(&action, gccStdin[0], STDIN_FILENO);
        posix_spawn_file_actions_addclose(&action, gccStdin[1]);

        //Call posix_spawn
        extern char **environ;
        int res = posix_spawn(&gccPid, argv[0], &action, NULL, argv, environ);
        if(res != 0) {
            fprintf(stderr, "fatal: Failed to call posix_spawn(), return value was %d\nPlease report this error to get this fixed\n", res);
            exit(EXIT_FAILURE);
        }

        //Close unneeded fd
        close(gccStdin[0]);

        output = fdopen(gccStdin[1], "w");
        if(!output) {
            perror("fatal: fdopen() of child's stdin failed");
            exit(EXIT_FAILURE);
        }

    }

    writeToFile(&compileState, output);
    fprintf(output, "\n");
    fclose(output);

    if(compileState.compileMode != assemblyFile) {
        waitpid(gccPid, &gccResult, 0);
    }

    if(gccResult != 0) {
        fprintf(stderr, "gcc exited unexpectedly with exit code %d. If you did not expect this to happen, please report this issue via Zulip (in the respective channel or as a PM to Tobias Kamm) so it can be fixed\n", gccResult);
        exit(EXIT_FAILURE);
    } else {
        exit(EXIT_SUCCESS);
    }
}
