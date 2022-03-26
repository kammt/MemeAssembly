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

#include "functions.h"
#include "analysisHelper.h"
#include "../logger/log.h"

#include <stdlib.h>
#include <string.h>
#include <stdio.h>


/**
 * Checks if the function definitions are valid. This includes making sure that
 *  - no function names are used twice
 *  - there is a main function if it is supposed to be executable
 * @param commandLinkedList a list of all occurrences of all commands. Index i contains a linked list of all commands that have opcode i
 * @param opcode the opcode of the function definition
 * @param compileState the current compile state
 */
void analyseFunctions(struct commandLinkedList** commandLinkedList, unsigned opcode, struct compileState* compileState) {
    checkDuplicateDefinition(commandLinkedList[opcode], compileState, false, 1, "function");

    //Check 2: Does a main-function exist?
    if(compileState -> compileMode == executable) {
        bool mainFunctionExists = false;
        struct commandLinkedList *functionDefinition = commandLinkedList[opcode];
        while (functionDefinition != NULL) {
            if (strcmp(functionDefinition->command->parameters[0], "main") == 0) {
                mainFunctionExists = true;
            }

            functionDefinition = functionDefinition -> next;
        }

        if (!mainFunctionExists) {
            printError(compileState -> files[0].fileName, 0, compileState, "unable to create an executable if no main-function was defined", 0);
        }
    }
}