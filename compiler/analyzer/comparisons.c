#include <string.h>
#include "comparisons.h"
#include "../logger/log.h"

struct comparison {
    char* parameter1;
    char* parameter2;
    int definedInLine;
};

struct comparisonJumpLabel {
    char* parameter;
    int definedInLine;
};


/**
 * Checks the validity of "Who would win" comparisons. It checks the following
 * - that no jump markers are defined twice
 * - that jump markers exist required by a comparison
 * @param commandsArray the parsed commands
 * @param whoWouldWinOpcode the opcode of the "Who would win" command. The opcode of the jump marker must be the one following it.
 */
void checkWhoWouldWinValidity(struct commandsArray *commandsArray, int whoWouldWinOpcode) {
    printDebugMessageWithNumber("Starting \"Who would win\" comparison validity check for opcode", whoWouldWinOpcode);

    //Traverse the command Array and count how many comparisons and jump labels there are
    size_t comparisonsUsed = 0;
    size_t comparisonJumpLabelsUsed = 0;
    for(size_t i = 0; i < commandsArray-> size; i++) {
        struct parsedCommand parsedCommand = commandsArray -> arrayPointer[i];
        if(parsedCommand.opcode == whoWouldWinOpcode) {
            printDebugMessageWithNumber("\t\tComparison found in line", parsedCommand.lineNum);
            comparisonsUsed++;
        } else if(parsedCommand.opcode == whoWouldWinOpcode + 1) {
            printDebugMessageWithNumber("\t\tComparison jump label found in line", parsedCommand.lineNum);
            comparisonJumpLabelsUsed++;
        }
    }

    printDebugMessageWithNumber("\tNumber of comparisons:", (int) comparisonsUsed);
    printDebugMessageWithNumber("\tNumber of comparison labels:", (int) comparisonJumpLabelsUsed);
    printDebugMessage("\tAllocating memory for structs", "");

    //Allocate memory for the structs
    struct comparison *comparisons = calloc(sizeof(struct comparison), comparisonsUsed);
    struct comparisonJumpLabel *comparisonJumpLabels = calloc(sizeof(struct comparisonJumpLabel), comparisonJumpLabelsUsed);
    if(comparisons == NULL || comparisonJumpLabels == NULL) {
        fprintf(stderr, "Critical error: Memory allocation for command parameter failed!");
        exit(EXIT_FAILURE);
    }

    //Traverse the command array again and add the objects to the respective arrays
    int comparisonArrayIndex = 0;
    int comparisonJumpArrayIndex = 0;
    for(size_t i = 0; i < commandsArray-> size; i++) {
        struct parsedCommand parsedCommand = commandsArray -> arrayPointer[i];
        if(parsedCommand.opcode == whoWouldWinOpcode) {
            struct comparison comparison = {
                    parsedCommand.parameters[0],
                    parsedCommand.parameters[1],
                    parsedCommand.lineNum
            };
            comparisons[comparisonArrayIndex++] = comparison;
        } else if(parsedCommand.opcode == whoWouldWinOpcode + 1) {
            struct comparisonJumpLabel comparisonJumpLabel =  {
                    parsedCommand.parameters[0],
                    parsedCommand.lineNum
            };
            comparisonJumpLabels[comparisonJumpArrayIndex++] = comparisonJumpLabel;
        }
    }

    printDebugMessage("\tMemory allocation and struct creation successful, starting checks", "");
    /*
     * We now need to check the following things
     * - That no labels were defined twice
     * - That a "p wins" was declared if p was used in a comparison
     */
    for(size_t i = 0; i < comparisonJumpLabelsUsed; i++) {
        printDebugMessage("\tLabel duplicity check for parameter", comparisonJumpLabels[i].parameter);
        for(size_t j = i + 1; j < comparisonJumpLabelsUsed; j++) {
            printDebugMessage("\t\tComparing against parameter", comparisonJumpLabels[j].parameter);
            if(strcmp(comparisonJumpLabels[i].parameter, comparisonJumpLabels[j].parameter) == 0) {
                printSemanticErrorWithExtraLineNumber("Comparison jump markers cannot be defined twice", comparisonJumpLabels[j].definedInLine, comparisonJumpLabels[i].definedInLine);
            }
        }
    }

    for(size_t i = 0; i < comparisonsUsed; i++) {
        uint8_t parameter1Defined = 0;
        uint8_t parameter2Defined = 0;
        printDebugMessage("\tLabel existence check for parameter", comparisons[i].parameter1);
        printDebugMessage("\tLabel existence check for parameter", comparisons[i].parameter2);

        for(size_t j = 0; j < comparisonJumpLabelsUsed; j++) {
            printDebugMessage("\t\tComparing against parameter", comparisonJumpLabels[j].parameter);
            if(strcmp(comparisons[i].parameter1, comparisonJumpLabels[j].parameter) == 0) {
                parameter1Defined = 1;
            }
            if(strcmp(comparisons[i].parameter2, comparisonJumpLabels[j].parameter) == 0) {
                parameter2Defined = 1;
            }
        }

        if(parameter1Defined == 0) {
            printSemanticError("No comparison jump marker defined for first parameter", comparisons[i].definedInLine);
        }
        if(parameter2Defined == 0) {
            printSemanticError("No comparison jump marker defined for second parameter", comparisons[i].definedInLine);
        }
    }

    printDebugMessage("\tChecks done, freeing memory", "");
    free(comparisons);
    free(comparisonJumpLabels);

    printDebugMessage("\"Who would win\" comparison validity check done", "");
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
    for(size_t i = 0; i < commandsArray->size; i++) {
        struct parsedCommand parsedCommand = commandsArray -> arrayPointer[i];
        if(parsedCommand.opcode == comparisonOpcode + 1) {
            printDebugMessageWithNumber("\tComparison jump label found in line", parsedCommand.lineNum);
            jumpMarkerDefined = parsedCommand.lineNum;
        }
    }

    //If no jump label was defined, we traverse the array again and print an error if any jumps exist
    if(jumpMarkerDefined == 0) {
        for(size_t i = 0; i < commandsArray->size; i++) {
            struct parsedCommand parsedCommand = commandsArray -> arrayPointer[i];
            if(parsedCommand.opcode == comparisonOpcode) {
                printSemanticError("\"they're the same picture\" wasn't defined anywhere", parsedCommand.lineNum);
            }
        }
    }
    printDebugMessage("Comparison label validity check done", "");
}
