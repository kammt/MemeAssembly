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

#include "randomCommands.h"
#include "../logger/log.h"

#include <stdlib.h>
#include <time.h>

/**
 * Chooses a random line of code in which a random jump marker will be inserted. This is going to be the jump point for all
 * instances of "confused stonks"
 * @param commandsArray the parsed commands
 * @param confusedStonksOpcode the opcode of this command.
 */
void setConfusedStonksJumpLabel(struct compileState* compileState, int confusedStonksOpcode) {
    printDebugMessageWithNumber("Starting confused stonks analysis with provided Opcode:", confusedStonksOpcode, compileState -> logLevel);
    srand((unsigned int) time(NULL));
    compileState -> commandsArray.randomIndex = (size_t) rand() % compileState -> commandsArray.size;
    printDebugMessageWithNumber("Chose random line for jump marker:", (int) compileState -> commandsArray.randomIndex, compileState -> logLevel);
}

/**
 * Checks how many times "perfectly balanced as all things should be" occurred and depending on this, marks random lines as to be ignored
 * @param commandsArray the parsed commands
 * @param perfectlyBalancedOpcode the opcode of this command
 */
void chooseLinesToBeDeleted(struct compileState* compileState, int perfectlyBalancedOpcode) {
    printDebugMessage("Starting analysis of the \"Perfectly balanced...\" command", "", compileState -> logLevel);
    int perfectlyBalancedUsed = 0;
    for(size_t i = 0; i < compileState -> commandsArray.size; i++) {
        if(compileState -> commandsArray.arrayPointer[i].opcode == perfectlyBalancedOpcode) {
            perfectlyBalancedUsed++;
        }
    }

    printDebugMessageWithNumber("\tamount of times perfectly balanced was used:", perfectlyBalancedUsed, compileState -> logLevel);
    //Calculating the number of lines to be deleted
    int linesToBeKept = (int) compileState -> commandsArray.size;
    for(int i = 0; i < perfectlyBalancedUsed; i++) {
        linesToBeKept /= 2;
    }
    int linesToBeDeleted = (int) compileState -> commandsArray.size - linesToBeKept;

    printDebugMessageWithNumber("\tamount of lines to be deleted:", linesToBeDeleted, compileState -> logLevel);
    if(linesToBeDeleted > 0) {
        printThanosASCII(linesToBeDeleted);

        srand((unsigned int) time(NULL));
        int selectedLines = 0;
        int lines[linesToBeDeleted];
        while(selectedLines < linesToBeDeleted) {
            //Generate a random line
            int randomLine = rand() % (int) compileState -> commandsArray.size;
            printDebugMessageWithNumber("\tGenerated random line:", randomLine, compileState -> logLevel);

            //Check if it was already selected
            for(int i = 0; i < selectedLines; i++) {
                if(randomLine == lines[i])  {
                    randomLine = -1;
                    break;
                }
            }
            //If it was already selected, generate a new line
            if(randomLine == -1) {
                printDebugMessage("\t\tLine was already generated", "", compileState -> logLevel);
                continue;
            }

            //If not, add it to the array
            lines[selectedLines++] = randomLine;
            //Set the command at this index as to be ignored
            compileState -> commandsArray.arrayPointer[randomLine].translate = 0;
            printDebugMessage("\t\tLine was added to the list", "", compileState -> logLevel);
        }
    }
    printDebugMessage("Analysis of the \"Perfectly balanced...\" command done", "", compileState -> logLevel);
}
