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

#include <string.h>
#include "comparisons.h"
#include "analysisHelper.h"
#include "../logger/log.h"

/**
 * Performs parameter analysis for the "who would win?..." and "p wins" commands
 * @param commandLinkedList a list of all occurrences of all commands. Index i contains a linked list of all commands that have opcode i
 * @param opcode the opcode of the "who would win?" command. "p wins" must have the following opcode
 * @param compileState the current compile state
 */
void analyseWhoWouldWinCommands(struct commandLinkedList** commandLinkedList, unsigned opcode, struct compileState* compileState) {
    printDebugMessage(compileState -> logLevel, "Starting analysis for \"who would win?\" command", 0);

    //First check: No comparison jump labels were defined twice
    checkDuplicateDefinition(commandLinkedList[opcode + 1], compileState, true, 1, "comparison jump marker");

    //Second check: "p wins" was declared
    checkCompanionCommandExistence(commandLinkedList[opcode], commandLinkedList[opcode + 1], compileState, 2, true, "comparison jump label");
}


/**
 * Checks that are usages of "corporate needs you to find the difference..." and "they're the same picture" are valid
 * Specifically, it checks that a jump label was defined if a comparison was defined
 * @param commandLinkedList a list of all occurrences of all commands. Index i contains a linked list of all commands that have opcode i
 * @param opcode the opcode of the "corporate needs ...?" command. "they're the same picture" must have the following opcode
 * @param compileState the current compile state
 */
void analyseTheyreTheSamePictureCommands(struct commandLinkedList** commandLinkedList, unsigned opcode, struct compileState* compileState) {
    printDebugMessage(compileState -> logLevel, "Starting analysis for \"corporate needs you to find the difference...\" command", 0);

    //Check 1: Was the equality label defined twice in the same file?
    checkDuplicateDefinition(commandLinkedList[opcode + 1], compileState, true, 0, "\"they're the same picture\"");

    //Check 2: "they're the same picture" must exist if a comparison was used
    checkCompanionCommandExistence(commandLinkedList[opcode], commandLinkedList[opcode + 1], compileState, 0, true, "\"they're the same picture\"");
}
