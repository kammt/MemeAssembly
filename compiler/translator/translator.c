/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021 Tobias Kamm

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
#include "../logger/log.h"

#include <time.h>
#include <string.h>
#include <unistd.h>
#include <limits.h>
#include <stab.h>

extern char* version_string;
extern struct command commandList[];
extern char *inputFileString;

int optimisationLevel = 0;
int useStabs = 0;

///STABS flags
//Required for stabs so that the function name can be inserted into the return label
char *currentFunctionName;
//If the previous command was ignored and the next label was already printed, this variable is set to 1 so that the label isn't printed twice
int DNLABEL = 0;

/**
 * Creates the first STABS entry in which the origin file is stored
 * @param outputFile the output file
 */
void stabs_writeFileInfo(FILE *outputFile) {
    //Check if the input file string starts with a /. If it does, it is an absolute path
    char cwd[PATH_MAX + 1];
    if(inputFileString[0] == '/') {
        fprintf(outputFile, ".stabs \"%s\", %d, 0, 0, .Ltext0\n", inputFileString, N_SO);
    } else {
        fprintf(outputFile, ".stabs \"%s/%s\", %d, 0, 0, .Ltext0\n", getcwd(cwd, PATH_MAX), inputFileString, N_SO);
    }
}

/**
 * Checks if the current command should not receive a separate STABS line info. Currently, this only affects breakpoints
 * @param opcode the opcode
 * @return 1 if it should be ignored, 0 if it needs line info
 */
int stabs_ignore(int opcode) {
    if(strcmp(commandList[opcode].translationPattern, "int3") == 0) {
        return 1;
    }
    return 0;
}

/**
 * Creates a function info STABS of a given function
 * @param outputFile the output file
 * @param functionName the name of the function
 */
void stabs_writeFunctionInfo(FILE *outputFile, char* functionName) {
    fprintf(outputFile, ".stabs \"%s:F1\", %d, 0, 0, %s\n", functionName, N_FUN, functionName);
    fprintf(outputFile, ".stabn %d, 0, 0, %s\n", N_LBRAC, functionName);
    fprintf(outputFile, ".stabn %d, 0, 0, .Lret_%s\n", N_RBRAC, functionName);
}

/**
 * Is called after a function return command is found. Creates a label for the function info stab to use
 * @param outputFile the output file
 */
void stabs_writeFunctionEndLabel(FILE *outputFile) {
    fprintf(outputFile, "\t.Lret_%s:\n", currentFunctionName);
}

/**
 * Creates a label for the line number STABS to use
 * @param outputFile the output file
 * @param parsedCommand the command that requires a line number info
 */
void stabs_writeLineLabel(FILE *outputFile, struct parsedCommand parsedCommand) {
    fprintf(outputFile, "\t.Lcmd_%d:\n", parsedCommand.lineNum);
}

/**
 * Creates a line number STABS of the provided command
 * @param outputFile the output file
 * @param parsedCommand the command that requires a line number info
 */
void stabs_writeLineInfo(FILE *outputFile, struct parsedCommand parsedCommand) {
    fprintf(outputFile, "\t.stabn %d, 0, %d, .Lcmd_%d\n", N_SLINE, parsedCommand.lineNum, parsedCommand.lineNum);
}

/**
 * Translates a given command into assembly. This includes inserting parameters and creating STABS info if necessary
 * @param commandsArray all commands that were parsed
 * @param index the index of the current command
 * @param outputFile the output file
 */
void translateToAssembly(struct commandsArray *commandsArray, size_t index, FILE *outputFile) {
    struct parsedCommand parsedCommand = commandsArray -> arrayPointer[index];
    if(parsedCommand.opcode != 0 && optimisationLevel == 69420) {
        printDebugMessage("\tCommand is not a function declaration, abort.", "");
        return;
    }

    //If we are supposed to create STABS info, we now need to create labels
    if(useStabs) {
        //If this is a function declaration, update the current function name
        if(parsedCommand.opcode == 0) {
            currentFunctionName = parsedCommand.parameters[0];
        //If this command is supposed to be ignored
        } else if (stabs_ignore(parsedCommand.opcode)) {
            //Already print the start label of the next command
            stabs_writeLineLabel(outputFile, commandsArray -> arrayPointer[index + 1]);
            //Set the DNLABEL variable
            DNLABEL = 1;
        //If it is a regular command
        } else if (DNLABEL == 0) {
            stabs_writeLineLabel(outputFile, parsedCommand);
        //If the previous command was ignored, i.e. DNLABEL was set, reset it to 0
        } else {
            DNLABEL = 0;
        }
    }

    struct command command = commandList[parsedCommand.opcode];
    char *translationPattern = command.translationPattern;

    size_t strLen = strlen(translationPattern);
    for(int i = 0; i < command.usedParameters; i++) {
        strLen += strlen(parsedCommand.parameters[i]);
    }

    char *translatedLine = malloc(strLen + 3); //Include an extra byte for the null-Pointer and two extra bytes in case []-brackets are needed for a pointer
    if(translatedLine == NULL) {
        fprintf(stderr, "Critical error: Memory allocation for command translation failed!");
        exit(EXIT_FAILURE);
    }
    translatedLine[0] = '\0';

    for(size_t i = 0; i < strlen(translationPattern); i++) {
        char character = translationPattern[i];
        if(character >= '0' && character <= (char) command.usedParameters + 47) {
            char *parameter = parsedCommand.parameters[character - 48];

            if(parsedCommand.isPointer == (character - 48) + 1) {
                printDebugMessage("\tAppending pointer parameter", parameter);

                //Manually add braces to the string
                size_t currentStrLen = strlen(translatedLine);

                //Append a '['
                translatedLine[currentStrLen] = '[';
                translatedLine[currentStrLen + 1] = '\0';
                //Append the parameter
                strncat(translatedLine, parameter, strLen);
                //Append a ']'
                currentStrLen = strlen(translatedLine);
                translatedLine[currentStrLen] = ']';
                translatedLine[currentStrLen + 1] = '\0';
            } else {
                printDebugMessage("\tAppending parameter", parameter);
                strncat(translatedLine, parameter, strLen);
            }
        } else {
            char appendix[2] = {character, '\0'};
            strncat(translatedLine, appendix, strLen);
        }
    }

    printDebugMessage("\tWriting to file: ", translatedLine);
    if(parsedCommand.opcode != 0) {
        fprintf(outputFile, "\t");
    }
    fprintf(outputFile, "%s\n", translatedLine);

    printDebugMessage("\tDone, freeing memory", "");
    free(translatedLine);

    //Now, we need to insert more commands based on the current optimisation level
    if (optimisationLevel == -1) {
        //Insert a nop
        fprintf(outputFile, "\tnop\n");
    } else if (optimisationLevel == -2) {
        //Push and pop rax
        fprintf(outputFile, "\tpush rax\n\tpop rax\n");
    } else if (optimisationLevel == -3) {
        //Save and restore xmm0 on the stack using movups
        fprintf(outputFile, "\tmovups [rsp + 8], xmm0\n\tmovups xmm0, [rsp + 8]\n");
    } else if(optimisationLevel == 69420) {
        //If we get here, then this was a function declaration. Insert a ret-statement and exit
        fprintf(outputFile, "\txor rax, rax\n\tret\n");
    }

    if(useStabs && parsedCommand.opcode != 0) {
        //If this was a return statement and this is the end of file or a function definition is followed by it, we reached the end of the function. Define the label for the N_RBRAC stab
        if(parsedCommand.opcode > 0 && parsedCommand.opcode <= 3 && commandsArray -> arrayPointer[index + 1].opcode == 0) {
            stabs_writeFunctionEndLabel(outputFile);
        }
        //In any case, we now need to write the line info to the file
        if(!stabs_ignore(parsedCommand.opcode)) {
            stabs_writeLineInfo(outputFile, parsedCommand);
        }
    }
}

void writeToFile(struct commandsArray *commandsArray, FILE *outputFile) {
    time_t t = time(NULL);
    struct tm tm = *localtime(&t);

    fprintf(outputFile, "#\n# Generated by the MemeAssembly compiler %s on %s#\n", version_string, asctime(&tm));
    fprintf(outputFile, ".intel_syntax noprefix\n");

    //Traverse the commandsArray to look for any functions
    for(size_t i = 0; i < commandsArray -> size; i++) {
        if(commandsArray -> arrayPointer[i].opcode == 0 && commandsArray -> arrayPointer[i].translate == 1) {
            //Write the function name with the prefix ".global" to the file
            fprintf(outputFile, ".global %s\n", commandsArray -> arrayPointer[i].parameters[0]);
        }
    }

    fprintf(outputFile, "\n.section .data\n\t.LCharacter: .ascii \"a\"\n");

    //Write the file info if we are using stabs
    if(useStabs) {
        stabs_writeFileInfo(outputFile);
    }

    fprintf(outputFile, "\n\n.section .text\n");

    fprintf(outputFile, "\n\n.Ltext0:\n");

    for(size_t i = 0; i < commandsArray -> size; i++) {
        if(i == commandsArray -> randomIndex) {
            fprintf(outputFile, "\t.LConfusedStonks: ");
        }

        if(commandsArray -> arrayPointer[i].translate == 1) {
            printDebugMessageWithNumber("Translating Index:", (int) i);
            translateToAssembly(commandsArray, i, outputFile);
        }
    }

    //If the optimisation level is 42069, then this function will not be used as all commands are optimised out
    if(optimisationLevel != 42069) {
        fprintf(outputFile, "\n\nwritechar:\n\tpush rcx\n\tpush r11\n\tpush rax\n\tpush rdi\n\tpush rsi\n\tpush rdx\n\tmov rdx, 1\n\tlea rsi, [rip + .LCharacter]\n\tmov rax, 1\n\tsyscall\n\tpop rdx\n\tpop rsi\n\tpop rdi\n\tpop rax\n\tpop r11\n\tpop rcx\n\t\n\tret\n");

        fprintf(outputFile, "\n\nreadchar:\n\tpush rcx\n\tpush r11\n\tpush rax\n\tpush rdi\n\tpush rsi\n\tpush rdx\n\n\tmov rdx, 1\n\tlea rsi, [rip + .LCharacter]\n\tmov rdi, 0\n\tmov rax, 0\n\tsyscall\n\t\n\tpop rdx\n\tpop rsi\n\tpop rdi\n\tpop rax\n\tpop r11\n\tpop rcx\n\tret\n");
    }

    //If we are using stabs, we now need to save all function info to the file
    if(useStabs) {
        //Traverse the commandsArray to look for any functions
        for(size_t i = 0; i < commandsArray -> size; i++) {
            if(commandsArray -> arrayPointer[i].opcode == 0 && commandsArray -> arrayPointer[i].translate == 1) {
                //Write the stabs info
                stabs_writeFunctionInfo(outputFile, commandsArray -> arrayPointer[i].parameters[0]);
            }
        }

        fprintf(outputFile, "\n.LEOF:\n");
        fprintf(outputFile, ".stabs \"\", %d, 0, 0, .LEOF\n", N_SO);
    }

    if(optimisationLevel == -4) {
        fprintf(outputFile, ".align 536870912\n");
    }

    printDebugMessage("Done, closing output file", "");
    fclose(outputFile);
}
