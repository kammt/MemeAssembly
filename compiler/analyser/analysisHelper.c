/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021-2022 Tobias Kamm

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

/**
 * This is a helper function that can be used by analysis functions. It checks for duplicate definitions of commands and prints an error if there is one
 * @param commandLinkedList the list of that command to be checked for duplicate definitions
 * @param compileState the compile state
 * @param oncePerFile when set to true, the same command is allowed once per file. If false, global duplicates will be checked as well
 * @param parametersToCheck how many parameters should be compared
 * @param itemName the name of the item. Will be inserted in the error message ("%s defined twice")
 */
void checkDuplicateDefinition(struct commandLinkedList* commandLinkedList, struct compileState* compileState, bool oncePerFile, uint8_t parametersToCheck, char* itemName) {
    if(parametersToCheck > 2) {
        parametersToCheck = 2;
    }

    struct commandLinkedList* listItem = commandLinkedList;
    while (listItem != NULL) {
        struct parsedCommand* command = listItem -> command;

        printDebugMessage(compileState -> logLevel, "\tLabel duplicity check for %s in line %lu in file %u", 3, itemName, command -> lineNum, listItem -> definedInFile);

        struct commandLinkedList* duplicateItem = listItem -> next;
        while(duplicateItem != NULL) {
            printDebugMessage(compileState -> logLevel, "\t\tComparing against parameter %s", 1, duplicateItem -> command -> parameters[0]);
            if((!oncePerFile || duplicateItem -> definedInFile == listItem -> definedInFile) &&
             (parametersToCheck < 1 || strcmp( command -> parameters[0], duplicateItem -> command -> parameters[0]) == 0) &&
             (parametersToCheck != 2 || strcmp( command -> parameters[1], duplicateItem -> command -> parameters[1]) == 0)) {
                printError(compileState -> files[duplicateItem -> definedInFile].fileName, duplicateItem -> command -> lineNum, compileState, "%s defined twice (already defined in %s:%lu)", 2, itemName, compileState -> files[listItem->definedInFile].fileName, command -> lineNum);
            }

            duplicateItem = duplicateItem -> next;
        }
        listItem = listItem -> next;
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
 * @param itemName the name of the item. Will be inserted in the error message ("%s wasn't defined [for parameter _]")
 */
void checkCompanionCommandExistence(struct commandLinkedList* parentCommands, struct commandLinkedList* childCommands, struct compileState* compileState, uint8_t parametersToCheck, char* itemName) {
    if(parametersToCheck > 2) {
        parametersToCheck = 2;
    }

    struct commandLinkedList* parentCommand = parentCommands;
    while (parentCommand != NULL) {
        bool childFound[2] = {false};
        struct parsedCommand* command = parentCommand -> command;

        printDebugMessage(compileState -> logLevel, "\tLooking for %s of parent command in line %lu in file %u", 3, itemName, command -> lineNum, parentCommand -> definedInFile);

        struct commandLinkedList* childCommand = childCommands;
        while(childCommand != NULL) {

            if(parentCommand -> definedInFile == childCommand -> definedInFile) {
                //The first child was found if either no parameters must match or the first parameter matches
                if(parametersToCheck == 0 || (parametersToCheck >= 1 && strcmp( command -> parameters[0], childCommand -> command -> parameters[0]) == 0)) {
                    childFound[0] = true;
                } else if(parametersToCheck == 2 && strcmp( command -> parameters[1], childCommand -> command -> parameters[0]) == 0)  {
                    childFound[1] = true;
                }
            }

            childCommand = childCommand -> next;
        }

        //We traversed all child commands, now we need to check if everything was defined properly
        if(parametersToCheck == 0) {
            if(!childFound[0]) {
                printError(compileState->files[parentCommand -> definedInFile].fileName, command -> lineNum, compileState,
                           "%s was not defined", 1, itemName);
            }
        } else {
            if(!childFound[0]) {
                printError(compileState->files[parentCommand -> definedInFile].fileName, command -> lineNum, compileState,
                           "%s was not defined for parameter \"%s\"", 2, itemName, command -> parameters[0]);
            }
            if(parametersToCheck == 2 && !childFound[1]) {
                printError(compileState->files[parentCommand -> definedInFile].fileName, command -> lineNum, compileState,
                           "%s was not defined for parameter \"%s\"", 2, itemName, command -> parameters[1]);
            }
        }

        parentCommand = parentCommand -> next;
    }
}

/**
 * This is a helper function that can be used by analysis functions. It checks that for all function calls the call target (function) exists
 * @param functionDefinitions list of function definitions
 * @param functionCalls list of function calls
 * @param compileState the compile state
 */
void checkFunctionCallsValid(struct commandLinkedList *functionDefinitions, struct commandLinkedList *functionCalls, struct compileState *compileState) {
    struct commandLinkedList *call = functionCalls;

    // Go through every function call and make sure we have a matching function definition
    while (call != NULL)
    {
        int callExists = false;

        struct commandLinkedList *definition = functionDefinitions;
        while (definition != NULL)
        {
            // If we find a function definition for this call, then everything is fine
            if (strcmp(call->command->parameters[0], definition->command->parameters[0]) == 0)
            {
                callExists = true;
                break;
            }

            definition = definition->next;
        }

        if (!callExists)
        {
            // We could not find a function definition for this call, so it's incorrect
            printError(compileState->files[call->definedInFile].fileName, call->command->lineNum, compileState,
                       "function \"%s\" was called, but never defined in any input file", 1, call->command->parameters[0]);
        }

        call = call->next;
    }
}
