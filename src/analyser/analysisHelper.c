/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021-2023 Tobias Kamm

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

#include "analysisHelper.h"
#include "../logger/log.h"
#include <string.h>

bool isEqualParameter(enum parameterType param0Type, union parameter param0, enum parameterType param1Type, union parameter param1) {
    if (param0Type == param1Type) {
        switch(param0Type) {
            case REG64:
            case REG32:
            case REG16:
            case REG8:
                return param0.reg8 == param1.reg8;
            case FUNC_NAME:
                return strcmp(param0.str, param1.str) == 0;
            case MONKE_LABEL:
                return param0.monkeLbl == param1.monkeLbl;
            default:
                printInternalCompilerError("Unable to compare parameters of parameter type %d", true, 1, param0Type);
                return false;
        }
    } else {
        return false;
    }
}

/**
 * This is a helper function that can be used by analysis functions. It checks for duplicate definitions of commands and prints an error if there is one
 * @param commandLinkedList the list of that command to be checked for duplicate definitions
 * @param compileState the compile state
 * @param oncePerFile when set to true, the same command is allowed once per file. If false, global duplicates will be checked as well
 * @param checkFirstParam if the first parameter (e.g. function name) should be compared as well
 * @param itemName the name of the item. Will be inserted in the error message ("%s defined twice")
 */
void checkDuplicateDefinition(struct commandLinkedList* commandLinkedList, struct compileState* compileState, bool oncePerFile, bool checkFirstParam, char* itemName) {
    struct commandLinkedList* listItem = commandLinkedList;
    while (listItem != NULL) {
        struct parsedCommand* command = listItem->command;

        printDebugMessage(compileState->logLevel, "\tLabel duplicity check for %s in line %lu in file %u", 3, itemName, command->lineNum, listItem->definedInFile);

        struct commandLinkedList* duplicateItem = listItem->next;
        while(duplicateItem != NULL) {
            printDebugMessage(compileState->logLevel, "\t\tComparing against parameter %s", 1, duplicateItem->command->parameters[0]);
            if(!oncePerFile || duplicateItem->definedInFile == listItem->definedInFile) {
                if(!checkFirstParam || isEqualParameter(listItem->command->paramTypes[0], listItem->command->parameters[0], duplicateItem->command->paramTypes[0], duplicateItem->command->parameters[0])) {
                    if(compileState->compileMode != bully) {
                        printError(compileState->files[duplicateItem->definedInFile].fileName, duplicateItem->command->lineNum, compileState,
                                   "%s defined twice (already defined in %s:%lu)", 2, itemName, compileState->files[listItem->definedInFile].fileName, command->lineNum);
                    } else {
                        //To fix this error in bully mode, only the first definition is valid, i.e. duplicateItem is removed
                        duplicateItem->command->translate = false;
                    }
                }
            }
            duplicateItem = duplicateItem->next;
        }
        listItem = listItem->next;
    }
}

/**
 * This is a helper function that can be used by analysis functions. It checks if for a given command, a "companion command" exists within the same file.
 * Examples:
 *  - When "where banana" is used, "banana" must be defined somewhere in the file
 *  - When "who would win? rax or 8" is used, "rax wins" and "8 wins" must be defined
 * @param parentCommands the parent command (which would be "where banana" and "who would win?..." in this case) - a command that requires another one if used
 * @param childCommands the child commands ("banana" and "_ wins" in our examples)
 * @param compileState the compile state
 * @param parametersToCheck how many parameters should be checked. Note that when this is set to e.g. 2,
 *                          it will look for a command with a first parameter that matches param1 and a separate command with a first parameter that matches param2
 * @param sameFile whether or not the companion command has to exist within the same file. Is true for all commands except function calls
 * @param itemName the name of the item. Will be inserted in the error message ("%s wasn't defined [for parameter _]")
 */
void checkCompanionCommandExistence(struct commandLinkedList* parentCommands, struct commandLinkedList* childCommands, struct compileState* compileState, uint8_t parametersToCheck, bool sameFile, char* itemName) {
    if(parametersToCheck > MAX_PARAMETER_COUNT) {
        parametersToCheck = MAX_PARAMETER_COUNT;
    }

    struct commandLinkedList* parentCommand = parentCommands;
    while (parentCommand != NULL) {
        bool childFound[parametersToCheck];
        for(int i = 0; i < parametersToCheck; i++) {
            childFound[i] = false;
        }
        const struct parsedCommand* command = parentCommand->command;

        printDebugMessage(compileState->logLevel, "\tLooking for %s of parent command in line %lu in file %u", 3, itemName, command->lineNum, parentCommand->definedInFile);

        const struct commandLinkedList* childCommand = childCommands;
        while(childCommand != NULL) {
            if(!sameFile || parentCommand->definedInFile == childCommand->definedInFile) {
                for(int i = 0; i < parametersToCheck; i++) {
                    if(!childFound[i] && isEqualParameter(command->paramTypes[i], command->parameters[i], childCommand->command->paramTypes[i], childCommand->command->parameters[i])) {
                        childFound[i] = true;
                    }
                }
            }

            childCommand = childCommand->next;
        }

        //We traversed all child commands, now we need to check if everything was defined properly
        if(parametersToCheck == 0) {
            if(!childFound[0]) {
                if(compileState->compileMode != bully) {
                    printError(compileState->files[parentCommand->definedInFile].fileName, command->lineNum,
                               compileState,
                               "%s was not defined", 1, itemName);
                } else {
                    //In bully mode, we just disregard the parent command
                    parentCommand->command->translate = false;
                }
            }
        } else {
            for(int i = 0; i < parametersToCheck; i++) {
                if(!childFound[i]) {
                    if(compileState->compileMode != bully) {
                        printError(compileState->files[parentCommand->definedInFile].fileName, command->lineNum,
                                   compileState,
                                   "%s was not defined for parameter #%u", 1, itemName, i);
                    } else {
                        //In bully mode, we just disregard the parent command
                        parentCommand->command->translate = false;
                    }
                }
            }
        }

        parentCommand = parentCommand->next;
    }
}
