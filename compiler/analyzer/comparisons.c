#include "comparisons.h"
#include "../logger/log.h"

void checkWhoWouldWinValidity(struct commandsArray *commandsArray, int whoWouldWinOpcode) {

}

/**
 * Checks that are usages of "corporate needs you to find the difference..." and "they're the same picture" are valid
 * Specifically, it checks that a jump label was defined if a comparison was defined
 * @param commandsArray the parsed commands
 * @param comparisonOpcode the opcode of the comparison command. The opcode of "they're the same picture" must be the one following it
 */
void checkTheyreTheSamePictureValidity(struct commandsArray *commandsArray, int comparisonOpcode) {
    printDebugMessageWithNumber("Starting comparison label validity check for opcode", comparisonOpcode);
    int jumpMarkerDefined = 0;

    //Traverse the command array and save the first occurrence of the jump label. If it then occurs another time, print an error
    for(int i = 0; i < commandsArray->size; i++) {
        struct parsedCommand parsedCommand = commandsArray -> arrayPointer[i];
        if(parsedCommand.opcode == comparisonOpcode + 1) {
            printDebugMessageWithNumber("\tComparison jump label found in line", parsedCommand.lineNum);
            jumpMarkerDefined = parsedCommand.lineNum;
        }
    }

    //If no jump label was defined, we traverse the array again and print an error if any jumps exist
    if(jumpMarkerDefined == 0) {
        for(int i = 0; i < commandsArray->size; i++) {
            struct parsedCommand parsedCommand = commandsArray -> arrayPointer[i];
            if(parsedCommand.opcode == comparisonOpcode) {
                printSemanticError("\"they're the same picture\" wasn't defined anywhere", parsedCommand.lineNum);
            }
        }
    }
    printDebugMessage("Comparison label validity check done", "");
}
