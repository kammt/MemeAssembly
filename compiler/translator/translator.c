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

#include "translator.h"
#include "../logger/log.h"

#include <time.h>
#include <string.h>
#include <unistd.h>
#include <limits.h>

///STABS flags
#define N_SO 100
#define N_SLINE 68
#define N_FUN 36
#define N_LBRAC 0xc0
#define N_RBRAC 0xe0

extern const char* const version_string;
extern const struct command commandList[];

/**
 * Creates the first STABS entry in which the origin file is stored
 * @param outputFile the output file
 */
void stabs_writeFileInfo(FILE *outputFile, char* inputFileString) {
    //Check if the input file string starts with a /. If it does, it is an absolute path
    char cwd[PATH_MAX + 1];
    if(inputFileString[0] == '/') {
        fprintf(outputFile, ".stabs \"%s\", %d, 0, 0, .Ltext0\n", inputFileString, N_SO);
    } else {
        fprintf(outputFile, ".stabs \"%s/%s\", %d, 0, 0, .Ltext0\n", getcwd(cwd, PATH_MAX), inputFileString, N_SO);
    }
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
void stabs_writeFunctionEndLabel(FILE *outputFile, char* currentFunctionName) {
    fprintf(outputFile, "\t.Lret_%s:\n", currentFunctionName);
}

/**
 * Creates a label for the line number STABS to use
 * @param outputFile the output file
 * @param parsedCommand the command that requires a line number info
 */
void stabs_writeLineLabel(FILE *outputFile, struct parsedCommand parsedCommand) {
    fprintf(outputFile, "\t.Lcmd_%lu:\n", parsedCommand.lineNum);
}

/**
 * Creates a line number STABS of the provided command
 * @param outputFile the output file
 * @param parsedCommand the command that requires a line number info
 */
void stabs_writeLineInfo(FILE *outputFile, struct parsedCommand parsedCommand) {
    fprintf(outputFile, "\t.stabn %d, 0, %lu, .Lcmd_%lu\n", N_SLINE, parsedCommand.lineNum, parsedCommand.lineNum);
}

/**
 *
 * @param compileState
 * @param currentFunctionName
 * @param parsedCommand
 * @param fileNum
 * @param outputFile
 */
void translateToAssembly(struct compileState* compileState, char* currentFunctionName, struct parsedCommand parsedCommand, unsigned fileNum, bool lastCommand, FILE *outputFile) {
    if(parsedCommand.opcode != 0 && compileState->optimisationLevel == o42069) {
        printDebugMessage(compileState -> logLevel, "\tCommand is not a function declaration, abort.", 0);
        return;
    }

    //If we are supposed to create STABS info, we now need to create labels
    if(compileState -> useStabs) {
        //If this is a function declaration, update the current function name
        if(parsedCommand.opcode != 0) {
            stabs_writeLineLabel(outputFile, parsedCommand);
        }
    }

    struct command command = commandList[parsedCommand.opcode];
    char *translationPattern = command.translationPattern;

    if(parsedCommand.opcode != 0) {
        fprintf(outputFile, "\t");
    }
    for(size_t i = 0; i < strlen(translationPattern); i++) {

        //Check if this is a format specifier
        if(translationPattern[i] == '{' && translationPattern[i + 2] == '}') {
            char formatSpecifier = translationPattern[i + 1];
            //If the format_specifier is F, we need to add the value of the current file's index to the string
            if(formatSpecifier == 'F') {
                fprintf(outputFile, "%u", fileNum);
            //Is it a parameter?
            } else if(formatSpecifier <= (char) command.usedParameters + 47) {
                char *parameter = parsedCommand.parameters[formatSpecifier - 48];
                if(parsedCommand.isPointer == (formatSpecifier - 48) + 1) {
                    fprintf(outputFile, "[%s]", parameter);
                } else {
                    fprintf(outputFile, "%s", parameter);
                }
            } else {
                fprintf(stderr, RED "Internal compiler error: " RESET "Invalid translation format specifier '%c' for opcode %u\nPlease report this error at https://github.com/kammt/MemeAssembly/issues/new", formatSpecifier, parsedCommand.opcode);
            }

            //move our pointer along by three characters instead of one, as we just parsed three characters
            i += 2;
        } else {
            fprintf(outputFile, "%c", translationPattern[i]);
        }
    }
    fprintf(outputFile, "\n");

    //Now, we need to insert more commands based on the current optimisation level
    if (compileState -> optimisationLevel == o_1) {
        //Insert a nop
        fprintf(outputFile, "\tnop\n");
    } else if (compileState -> optimisationLevel == o_2) {
        //Push and pop rax
        fprintf(outputFile, "\tpush rax\n\tpop rax\n");
    } else if (compileState -> optimisationLevel == o_3) {
        //Save and restore xmm0 on the stack using movups
        fprintf(outputFile, "\tmovups [rsp + 8], xmm0\n\tmovups xmm0, [rsp + 8]\n");
    } else if(compileState -> optimisationLevel == o42069) {
        //If we get here, then this was a function declaration. Insert a ret-statement and exit
        fprintf(outputFile, "\txor rax, rax\n\tret\n");
    }

    if(compileState -> useStabs && parsedCommand.opcode != 0) {
        //If this was a return statement and this is the end of file or a function definition is followed by it, we reached the end of the function. Define the label for the N_RBRAC stab
        if(lastCommand) {
            stabs_writeFunctionEndLabel(outputFile, currentFunctionName);
        }
        //In any case, we now need to write the line info to the file
        stabs_writeLineInfo(outputFile, parsedCommand);
    }
}

void writeToFile(struct compileState* compileState, FILE *outputFile) {
    time_t t = time(NULL);
    struct tm tm = *localtime(&t);

    fprintf(outputFile, "#\n# Generated by the MemeAssembly compiler %s on %s#\n", version_string, asctime(&tm));
    fprintf(outputFile, ".intel_syntax noprefix\n");

    //Define all functions as global
    for(unsigned i = 0; i < compileState->fileCount; i++) {
        for(size_t j = 0; j < compileState -> files[i].functionCount; j++) {
            //Write the function name with the prefix ".global" to the file
            fprintf(outputFile, ".global %s\n", compileState -> files[i].functions[j].name);
        }
    }

    #ifdef WINDOWS
    //To interact with the Windows API, we need to reference the needed functions
    fprintf(outputFile, "\n.extern GetStdHandle\n.extern WriteFile\n.extern ReadFile\n");
    #endif

    fprintf(outputFile, "\n\n.text\n\t");
    fprintf(outputFile, "\n\n.Ltext0:\n");

    for(unsigned i = 0; i < compileState->fileCount; i++) {
        struct file currentFile = compileState -> files[i];
        //Write the file info if we are using stabs
        if(compileState -> useStabs) {
            stabs_writeFileInfo(outputFile, currentFile.fileName);
        }

        size_t line = 0;
        for(size_t j = 0; j < currentFile.functionCount; j++) {
            struct function currentFunction = currentFile.functions[j];

            for(size_t k = 0; k < currentFunction.numberOfCommands; k++) {
                struct parsedCommand currentCommand = currentFunction.commands[k];

                //Print the confused stonks label now if it should be at this position
                if (line == currentFile.randomIndex) {
                    fprintf(outputFile, "\t.LConfusedStonks_%u: \n", i);
                }

                //If it should be translated, translate it
                if (currentCommand.translate) {
                    translateToAssembly(compileState, currentFunction.name, currentCommand, i,
                                        (k == currentFunction.numberOfCommands - 1), outputFile);
                }

                //Insert STABS function-info
                if (compileState->useStabs) {
                    stabs_writeFunctionInfo(outputFile, currentFunction.name);
                }
                line++;
            }
        }
    }


    //Add an "end marker" if we are using stabs
    if(compileState -> useStabs) {
        fprintf(outputFile, "\n.LEOF:\n");
        fprintf(outputFile, ".stabs \"\", %d, 0, 0, .LEOF\n", N_SO);
    }

    if(compileState->optimisationLevel == o_s) {
        fprintf(outputFile, ".align 536870912\n");
    }
}
