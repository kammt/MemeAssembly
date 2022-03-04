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

#include "jumpLabels.h"
#include "../logger/log.h"

#include <string.h>

extern struct command commandList[];

struct monkeLabel {
    char* labelName;
    int definedInLine;
};


/**
 * Checks if the usage of all monke labels and return jumps are valid. This includes
 *  - that no jump labels were defined twice
 *  - that no returns were used where the label name wasn't defined
 */
void checkMonkeJumpLabelValidity(struct compileState *compileState, int monkeOpcode) {
    printDebugMessage("Beginning Monke jump label validity check", "", compileState -> logLevel);
    //First, traverse the array and count the number of appearences of the monke-command. This is how many array items we need to create
    size_t monkeFound = 0;
    size_t returnMonkeFound = 0;

    for(size_t i = 0; i < compileState -> commandsArray.size; i++) {
        struct parsedCommand parsedCommand = compileState -> commandsArray.arrayPointer[i];
        if(parsedCommand.opcode == monkeOpcode) {
            monkeFound++;
        } else if(parsedCommand.opcode == monkeOpcode + 1) {
            returnMonkeFound++;
        }
    }

    printDebugMessageWithNumber("\tJump label definitions found:", (int) monkeFound, compileState -> logLevel);
    printDebugMessageWithNumber("\tJumps found", (int) returnMonkeFound, compileState -> logLevel);

    printDebugMessage("\tAllocating memroy for structs", "", compileState -> logLevel);

    //Now we allocate memory for our monkeLabel structs
    struct monkeLabel *labelDefinitions = calloc(sizeof(struct monkeLabel), monkeFound);
    struct monkeLabel *labelJumps = calloc(sizeof(struct monkeLabel), returnMonkeFound);
    if(labelDefinitions == NULL || labelJumps == NULL) {
        fprintf(stderr, "Critical error: Memory allocation for command parameter failed!");
        exit(EXIT_FAILURE);
    }

    int labelArrayIndex = 0;
    int jumpArrayIndex = 0;
    for(size_t i = 0; i < compileState -> commandsArray.size; i++) {
        struct parsedCommand parsedCommand = compileState -> commandsArray.arrayPointer[i];
        if(parsedCommand.opcode == monkeOpcode || parsedCommand.opcode == monkeOpcode + 1) {
            struct monkeLabel monkeLabel;
            monkeLabel.definedInLine = parsedCommand.lineNum;
            monkeLabel.labelName = parsedCommand.parameters[0];

            if(parsedCommand.opcode == monkeOpcode) {
                labelDefinitions[labelArrayIndex++] = monkeLabel;
            } else {
                labelJumps[jumpArrayIndex++] = monkeLabel;
            }
        }
    }

    printDebugMessage("\tstructs successfully added to arrays, beginning error checks", "", compileState -> logLevel);
    //First check: Did any monke labels get defined twice (i.e. two definitions with the same label)
    for(int i = 0; i < labelArrayIndex; i++) {
        printDebugMessage("\tChecking jump label", labelDefinitions[i].labelName, compileState -> logLevel);
        for(int j = i + 1; j < labelArrayIndex; j++) {
            printDebugMessage("\t\tComparing against jump label", labelDefinitions[j].labelName, compileState -> logLevel);
            if(strcmp(labelDefinitions[i].labelName, labelDefinitions[j].labelName) == 0) {
                printSemanticErrorWithExtraLineNumber("Monke labels cannot be defined twice", labelDefinitions[j].definedInLine, labelDefinitions[i].definedInLine, compileState);
            }
        }
    }


    //Second check: Did any "return to monke"-commands use a label that doesn't exist?
    for(int i = 0; i < jumpArrayIndex; i++) {
        uint8_t labelFound = 0;
        printDebugMessage("\tChecking return jump", labelJumps[i].labelName, compileState -> logLevel);
        //For every jump, traverse through the label array and see if that label is defined somewhere
        for(int j = 0; j < labelArrayIndex; j++) {
            printDebugMessage("\t\tComparing against jump label", labelDefinitions[j].labelName, compileState -> logLevel);
            //If it is defined, set the value to 1 and escape the loop
            if(strcmp(labelJumps[i].labelName, labelDefinitions[j].labelName) == 0) {
                labelFound = 1;
                break;
            }
        }

        //If we got here with the variable value still 0, it means that no comparison was successful, print an error
        if(labelFound == 0) {
            printSemanticError("This label wasn't defined anywhere", labelDefinitions[i].definedInLine, compileState);
        }
    }

    printDebugMessage("Monke jump label validity check done, freeing memory", "", compileState -> logLevel);
    //Now, we free all memory again
    free(labelDefinitions);
    free(labelJumps);
}

/**
 * Checks if the jump label (upgrade + fuck go back / banana + where banana) are correctly used, i.e. if
 * - a marker is defined when a jump is present
 * - markers are only used once
 * @param compileState -> commands the parsed commands
 * @param labelOpcode the opcode of the jump label. The jump opcode must be the one after it
 */
void checkJumpLabelValidity(struct compileState *compileState, int labelOpcode) {
    printDebugMessageWithNumber("Starting jump label validity check for opcode", labelOpcode, compileState -> logLevel);
    int jumpMarkerDefined = 0;

    //Traverse the command array and save the first occurrence of the jump label. If it then occurs another time, print an error
    for(size_t i = 0; i < compileState -> commandsArray.size; i++) {
        struct parsedCommand parsedCommand = compileState -> commandsArray.arrayPointer[i];
        if(parsedCommand.opcode == labelOpcode) {
            if(jumpMarkerDefined == 0) {
                printDebugMessageWithNumber("\tJump Marker found in line", parsedCommand.lineNum, compileState -> logLevel);
                jumpMarkerDefined = parsedCommand.lineNum;
            } else {
                printDebugMessageWithNumber("\tExtra Jump Marker found in line", parsedCommand.lineNum, compileState -> logLevel);
                printSemanticErrorWithExtraLineNumber("Marker cannot be defined twice", parsedCommand.lineNum, jumpMarkerDefined, compileState);
            }
        }
    }

    //If no jump marker was defined, we traverse the array again and print an error if any jumps exist
    if(jumpMarkerDefined == 0) {
        for(size_t i = 0; i < compileState -> commandsArray.size; i++) {
            struct parsedCommand parsedCommand = compileState -> commandsArray.arrayPointer[i];
            if(parsedCommand.opcode == labelOpcode + 1) {
                printSemanticError("Marker definition missing", parsedCommand.lineNum, compileState);
            }
        }
    }
    printDebugMessage("Jump label validity check done", "", compileState -> logLevel);
}
