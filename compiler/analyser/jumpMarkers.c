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

#include "jumpMarkers.h"
#include "analysisHelper.h"
#include "../logger/log.h"

/**
 * Checks if the usage of all monke labels and return jumps are valid. This includes
 *  - that no jump labels were defined twice
 *  - that no returns were used where the label name wasn't defined
 * @param commandLinkedList a list of all occurrences of all commands. Index i contains a linked list of all commands that have opcode i
 * @param opcode the opcode of the "monke" command. The "return to monke" command must have the following opcode
 * @param compileState the current compile state
 */
void analyseMonkeMarkers(struct commandLinkedList** commandLinkedList, unsigned opcode, struct compileState* compileState) {
    printDebugMessage(compileState -> logLevel, "Beginning Monke jump label validity check", 0);

    checkDuplicateDefinition(commandLinkedList[opcode], compileState, false, 1, "monke jump marker");
    checkCompanionCommandExistence(commandLinkedList[opcode + 1], commandLinkedList[opcode], compileState, 1, false, "monke jump marker");
}

/**
 * Checks if the jump label (upgrade + fuck go back / banana + where banana) are correctly used, i.e. if
 * - a marker is defined when a jump is present
 * - markers are only used once
 * @param commandLinkedList a list of all occurrences of all commands. Index i contains a linked list of all commands that have opcode i
 * @param opcode the opcode of the marker command. The marker jump must have the following opcode
 * @param compileState the current compile state
 */
void analyseJumpMarkers(struct commandLinkedList** commandLinkedList, unsigned opcode, struct compileState* compileState) {
    printDebugMessage(compileState -> logLevel, "Starting jump label validity check for opcode %u", 1, opcode);

    checkDuplicateDefinition(commandLinkedList[opcode], compileState, true, 0, "jump marker");
    checkCompanionCommandExistence(commandLinkedList[opcode + 1], commandLinkedList[opcode], compileState, 0, true, "jump marker");
}
