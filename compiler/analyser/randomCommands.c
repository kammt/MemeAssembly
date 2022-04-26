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

#include "randomCommands.h"
#include "../logger/log.h"

#include <stdlib.h>
#include <time.h>

/**
 * Chooses a random line of code for each input file in which a random jump marker will be inserted. This is going to be the jump point for all
 * instances of "confused stonks" within that file
 * @param commandLinkedList a list of all occurrences of all commands. Index i contains a linked list of all commands that have opcode i - unused
 * @param opcode the opcode - unused
 * @param compileState the current compile state
 */
void setConfusedStonksJumpLabel(struct commandLinkedList** commandLinkedList, unsigned opcode, struct compileState* compileState) {
    (void)(commandLinkedList);
    (void)(opcode);

    srand((unsigned int) time(NULL));

    for(unsigned i = 0; i < compileState -> fileCount; i++) {
        if(compileState->files[i].loc == 0) {
            continue;
        }
        compileState -> files[i].randomIndex = ((size_t) rand()) % (compileState -> files[i].loc);
        printDebugMessage(compileState -> logLevel, "Chose random line for jump marker: %lu", 1, compileState -> files[i].randomIndex);
    }
}

/**
 * Checks how many times "perfectly balanced as all things should be" occurred and depending on this, marks random lines as to be ignored
 * @param commandLinkedList a list of all occurrences of all commands. Index i contains a linked list of all commands that have opcode i
 * @param opcode the opcode of this command
 * @param compileState the current compile state
 */
void chooseLinesToBeDeleted(struct commandLinkedList** commandLinkedList, unsigned opcode, struct compileState* compileState) {
    printDebugMessage(compileState -> logLevel, "Starting analysis of the \"Perfectly balanced...\" command", 0);
    unsigned perfectlyBalancedUsed = 0;

    struct commandLinkedList* listItem = commandLinkedList[opcode];
    while (listItem != NULL) {
        perfectlyBalancedUsed++;
        listItem = listItem -> next;
    }

    printDebugMessage(compileState -> logLevel, "\tamount of times perfectly balanced was used: %u", 1, perfectlyBalancedUsed);
    //Calculating the number of lines to be deleted
    size_t loc = 0;
    for(unsigned i = 0; i < compileState -> fileCount; i++) {
        loc += compileState -> files[i].loc;
    }

    size_t linesToBeKept = loc;
    for(unsigned i = 0; i < perfectlyBalancedUsed; i++) {
        linesToBeKept /= 2;
    }
    size_t linesToBeDeleted = loc - linesToBeKept;

    printDebugMessage(compileState -> logLevel, "\tamount of lines to be deleted: %lu", 1, linesToBeDeleted);
    if(linesToBeDeleted > 0) {
        printThanosASCII(linesToBeDeleted);

        srand(time(NULL));
        size_t selectedLines = 0;
        while(selectedLines < linesToBeDeleted) {
            //Generate a random file
            unsigned randomFile = rand() % compileState -> fileCount;

            //Generate a random line within file
            size_t randomLine = rand() % compileState -> files[randomFile].loc;
            printDebugMessage(compileState -> logLevel, "\tGenerated random line %lu in file %u", 2, randomLine, randomFile);

            //Check if it was already selected
            if(compileState -> files[randomFile].parsedCommands[randomLine].translate == false) {
                printDebugMessage(compileState -> logLevel, "\t\tLine was already generated", 0);
                continue;
            }

            //If not, set the translate-field to false
            compileState -> files[randomFile].parsedCommands[randomLine].translate = false;
            printDebugMessage(compileState -> logLevel, "\t\tLine was added to the list", 0);

            selectedLines++;
        }
    }
}
