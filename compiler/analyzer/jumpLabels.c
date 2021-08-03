#include "jumpLabels.h"
#include "../logger/log.h"

extern struct command commandList[];

void checkMonkeJumpLabelValidity(struct commandsArray *commandsArray, int monkeOpcode) {

}

/**
 * Checks if the jump label (upgrade + fuck go back / banana + where banana) are correctly used, i.e. if
 * - a marker is defined when a jump is present
 * - markers are only used once
 * @param commandsArray the parsed commands
 * @param labelOpcode the opcode of the jump label. The jump opcode must be the one after it
 */
void checkJumpLabelValidity(struct commandsArray *commandsArray, int labelOpcode) {
    printDebugMessageWithNumber("Starting jump label validity check for opcode", labelOpcode);
    int jumpMarkerDefined = 0;

    //Traverse the command array and save the first occurrence of the jump label. If it then occurs another time, print an error
    for(int i = 0; i < commandsArray->size; i++) {
        struct parsedCommand parsedCommand = commandsArray -> arrayPointer[i];
        if(parsedCommand.opcode == labelOpcode) {
            if(jumpMarkerDefined == 0) {
                printDebugMessageWithNumber("\tJump Marker found in line", parsedCommand.lineNum);
                jumpMarkerDefined = parsedCommand.lineNum;
            } else {
                printDebugMessageWithNumber("\tExtra Jump Marker found in line", parsedCommand.lineNum);
                printSemanticErrorWithExtraLineNumber("Marker cannot be defined twice", parsedCommand.lineNum, jumpMarkerDefined);
            }
        }
    }

    //If no jump marker was defined, we traverse the array again and print an error if any jumps exist
    if(jumpMarkerDefined == 0) {
        for(int i = 0; i < commandsArray->size; i++) {
            struct parsedCommand parsedCommand = commandsArray -> arrayPointer[i];
            if(parsedCommand.opcode == labelOpcode + 1) {
                printSemanticError("Marker definition missing", parsedCommand.lineNum);
            }
        }
    }
}
