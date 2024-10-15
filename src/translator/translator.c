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

#include "translator.h"

#include <assert.h>

#include "../logger/log.h"
#include "../analyser/functions.h"

#include <time.h>
#include <string.h>
#include <unistd.h>
#include <limits.h>

extern const char* const versionString;
extern const struct command commandList[];

//Used to pseudo-random generation when using bully mode
extern uint64_t computedIndex;

const char* const martyrdomCode = "push rax\n"
                                  "    push rdi\n"
                                  "    push rsi\n"
                                  "    push rdx\n"
                                  "    push r10\n"
                                  "    push rcx\n"
                                  "    push r11\n"
                                  "    \n"
                                  "    lea rax, [rip + killParent]\n"
                                  "    mov [rip + .Lsa_handler], rax\n"
                                  #ifdef MACOS
                                  //For some reason, signaling SIGINT on MacOS leads to a segmentation fault if the second qword of the sigaction struct
                                        //doesn't contain the address as well
                                        "    mov [rip + .Lsa_handler_2], rax\n"
                                  #endif
                                  "\n"
                                  #ifdef LINUX
                                  "    mov rax, 13\n"
                                  #else
                                  "    mov rax, 0x200002E\n"
                                  #endif
                                  "    mov rdi, 2\n"
                                  "    lea rsi, [rip + .LsigStruct]\n"
                                  "    xor rdx, rdx\n"
                                  "    mov r10, 8\n"
                                  "    syscall\n"
                                  "    \n"
                                  "    pop r11\n"
                                  "    pop rcx\n"
                                  "    pop r10\n"
                                  "    pop rdx\n"
                                  "    pop rsi\n"
                                  "    pop rdi\n"
                                  "    pop rax\n\n";


/**
 * Receives a command and writes its assembly translation into the output file
 * @param compileState the current compile state
 * @param currentFunctionName the name of the current function. Needed for writing some stabs debugging info
 * @param parsedCommand the command to be translated
 * @param fileNum the id of the current file
 * @param outputFile the file where the translation should be written to
 */
void translateToAssembly(struct compileState* compileState, char* currentFunctionName, struct parsedCommand parsedCommand, unsigned fileNum, bool lastCommand, FILE *outputFile) {
    if(commandList[parsedCommand.opcode].commandType != COMMAND_TYPE_FUNC_DEF && compileState->optimisationLevel == o69420) {
        printDebugMessage(compileState->logLevel, "\tCommand is not a function declaration, abort.", 0);
        return;
    }

    const struct command command = commandList[parsedCommand.opcode];
    const char* translationPattern = command.translationPattern;

    if(commandList[parsedCommand.opcode].commandType != COMMAND_TYPE_FUNC_DEF) {
        fprintf(outputFile, "\t");
    }

    for(size_t i = 0; i < strlen(translationPattern); i++) {
        const char* nextParam = strstr(translationPattern + i, "{");

        if(nextParam == NULL) {
            fwrite(translationPattern + i, 1, strlen(translationPattern + i), outputFile);
            break;
        }

        //Print all characters up to the {X}
        size_t charsToPrint = nextParam - i - translationPattern;
        fwrite(translationPattern + i, 1, charsToPrint, outputFile);
        //After this, we point to the character that is between the {}'s
        i += charsToPrint + 1;
        assert(translationPattern[i + 1] == '}');

        const char formatSpecifier = translationPattern[i];
        //If the format_specifier is F, we need to add the value of the current file's index to the string
        if(formatSpecifier == 'F') {
            fprintf(outputFile, "%u", fileNum);
        //Is it a parameter?
        } else if(formatSpecifier >= '0' && formatSpecifier < command.usedParameters + '0') {
            uint8_t index = formatSpecifier - 48;
            char *parameter = parsedCommand.parameters[index];
            if(parsedCommand.isPointer == index + 1) {
                /*
                 * If we are in bully mode, we first need to check if the operand size is unknown (e.g. a pointer
                 * and a decimal number are used). This is because this check is skipped in parameters.c
                 */
                if(compileState->compileMode == bully && commandList[parsedCommand.opcode].usedParameters == 2 && !PARAM_ISREG(parsedCommand.paramTypes[index + 1 % 2])) {
                    const char* operandSizes[] = {"BYTE PTR", "WORD PTR", "DWORD PTR", "QWORD PTR"};
                    fprintf(outputFile, "%s [%s]", operandSizes[computedIndex % 4], parameter);
                } else {
                    fprintf(outputFile, "[%s]", parameter);
                }
            } else {
                /*
                 * If the parameter is a decimal number, write it as a hex string. Fixes issue #73
                 * The check is only needed here, as a decimal number cannot be a pointer
                 */
                if(parsedCommand.paramTypes[index] == PARAM_DECIMAL) {
                    fprintf(outputFile, "0x%llX", strtoll(parameter, NULL, 10));
                } else {
                    fprintf(outputFile, "%s", parameter);
                }
            }
        } else {
            printInternalCompilerError("Invalid translation format specifier '%c' for opcode %u", true, 2, formatSpecifier, parsedCommand.opcode);
            exit(EXIT_FAILURE);
        }

        i += 1;
    }
    fprintf(outputFile, "\n");

    //Now, we need to insert more commands based on the current optimisation level
    if (compileState->optimisationLevel == o_1) {
        //Insert a nop
        fprintf(outputFile, "\tnop\n");
    } else if (compileState->optimisationLevel == o_2) {
        //Push and pop rax
        fprintf(outputFile, "\tpush rax\n\tpop rax\n");
    } else if (compileState->optimisationLevel == o_3) {
        //Save and restore xmm0 on the stack using movups
        fprintf(outputFile, "\tmovups [rsp + 8], xmm0\n\tmovups xmm0, [rsp + 8]\n");
    } else if(compileState->optimisationLevel == o69420) {
        //If we get here, then this was a function declaration. Insert a ret-statement and exit
        fprintf(outputFile, "\txor rax, rax\n\tret\n");
    }
}

void writeToFile(struct compileState* compileState, FILE *outputFile) {
    time_t t = time(NULL);
    struct tm tm = *localtime(&t);

    fprintf(outputFile, "#\n# Generated by the MemeAssembly compiler %s on %s#\n", versionString, asctime(&tm));
    fprintf(outputFile, ".intel_syntax noprefix\n");
    fprintf(outputFile, ".global _start\n");

    //Define all functions as global
    for(unsigned i = 0; i < compileState->fileCount; i++) {
        for(size_t j = 0; j < compileState->files[i].functionCount; j++) {
            //Only write if the function definition is to be translated
            if(compileState->files[i].functions[j].commands[0].translate) {
                //Write the function name with the prefix ".global" to the file
                fprintf(outputFile, ".global %s\n", compileState->files[i].functions[j].commands[0].parameters[0]);
            }
        }
    }

    fprintf(outputFile, "\n.data\n\t");
    fprintf(outputFile, ".LCharacter: .ascii \"a\"\n\t.Ltmp64: .byte 0, 0, 0, 0, 0, 0, 0, 0\n");

    //Struct for martyrdom command
    #ifdef LINUX
    fprintf(outputFile, "\t.LsigStruct:\n"
                        "\t\t.Lsa_handler: .quad 0\n"
                        "\t\t.quad 0x04000000\n"
                        "\t\t.quad 0, 0\n\n");
    #elif defined(MACOS)
    fprintf(outputFile, "\t.LsigStruct:\n"
                        "\t\t.Lsa_handler: .quad 0\n"
                        "\t\t.Lsa_handler_2: .quad 0\n"
                        "\t\t.quad 0, 0\n\n");
    #endif

    fprintf(outputFile, "\n\n.text\n\t");
    fprintf(outputFile, "\n\n.Ltext0:\n");
    fprintf(outputFile, "_start:\n"
                            "\tmov edi, [rsp]\n"
                            "\tlea rsi, [rsp + 8]\n"
                            "\tcall main\n"
                            "\tmov edi, eax\n"
                            "\tmov eax, 60\n"
                            "\tsyscall\n\n");

    if(compileState->allowIoCommands) {
        fprintf(outputFile, "killParent:\n"
                            #ifdef LINUX
                            "    mov rax, 110\n"
                            #else
                            "    mov rax, 0x2000027\n"
                            #endif
                            "    syscall\n"
                            "\n"
                            "    mov rdi, rax\n"
                            "    mov rsi, 9\n"
                            #ifdef LINUX
                            "    mov rax, 62\n"
                            #else
                            "    mov rax, 0x2000025\n"
                            #endif
                            "    syscall\n"
                            "\n"
                            "    mov rdi, 0\n"
                            "    mov rax, 60\n"
                            "    syscall\n"
                            "    ret\n\n");
    }

    /*
     * If we're in bully mode and an executable is to be generated, we omitted the check
     * if there was a main-function
     * We do that check now. If no main function exists, the first function in the file becomes the main function
     */
    if(compileState->compileMode == bully && compileState->outputMode == executable && !mainFunctionExists(compileState)) {
        fprintf(outputFile, "\n.global main\n\t");
        fprintf(outputFile, "\nmain:\n\t");
        fprintf(outputFile, "%s", martyrdomCode);
    }

    for(unsigned i = 0; i < compileState->fileCount; i++) {
        struct file currentFile = compileState->files[i];

        size_t line = 0;
        for(size_t j = 0; j < currentFile.functionCount; j++) {
            struct function currentFunction = currentFile.functions[j];
            char* functionName = currentFunction.commands[0].parameters[0];

            for(size_t k = 0; k < currentFunction.numberOfCommands; k++) {
                const char *const mainFuncName =
                #ifdef MACOS
                        "_main";
                #else
                        "main";
                #endif

                if (compileState->martyrdom && compileState->allowIoCommands && k == 1 && strcmp(functionName, mainFuncName) == 0) {
                    fprintf(outputFile, "%s", martyrdomCode);
                }

                struct parsedCommand currentCommand = currentFunction.commands[k];

                //Print the confused stonks label now if it should be at this position
                if (line == currentFile.randomIndex) {
                    fprintf(outputFile, "\t.LConfusedStonks_%u: \n", i);
                }

                //If it should be translated, translate it
                if (currentCommand.translate) {
                    translateToAssembly(compileState, functionName, currentCommand, i,
                                        (k == currentFunction.numberOfCommands - 1), outputFile);
                }

                line++;
            }
        }
    }

    if(compileState->optimisationLevel == o_s) {
        fprintf(outputFile, ".align 536870912\n");
    }
}
