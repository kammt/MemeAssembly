#include "parameters.h"
#include "../logger/log.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

#define NUMBER_OF_8_BIT_REGISTERS 20
#define NUMBER_OF_64_32_BIT_REGISTERS 32
#define NUMBER_OF_16_BIT_REGISTERS 16
#define NUMBER_OF_ESCAPE_SEQUENCES 10

extern struct command commandList[];

char *registers_64_32_bit[32] = {
        "rax", "eax",
        "rbx", "ebx",
        "rcx", "ecx",
        "rdx", "edx",
        "rdi", "edi",
        "rsi", "esi",
        "rsp", "esp",
        "rbp", "ebp",
        "r8", "r8d",
        "r9", "r9d",
        "r10", "r10d",
        "r11", "r11d",
        "r12", "r12d",
        "r13", "r13d",
        "r14", "r14d",
        "r15", "r15d",
};

char *registers_16_bit[16] = {
        "ax",
        "bx",
        "cx",
        "dx",
        "di",
        "si",
        "sp",
        "bp",
        "r8w",
        "r9w",
        "r10w",
        "r11w",
        "r12w",
        "r13w",
        "r14w",
        "r15w",
};

char *registers_8_bit[NUMBER_OF_8_BIT_REGISTERS] = {
        "al", "ah",
        "bl", "bh",
        "cl", "ch",
        "dl", "dh",
        "dil",
        "sil",
        "spl",
        "bpl",
        "r8b",
        "r9b",
        "r10b",
        "r11b",
        "r12b",
        "r13b",
        "r14b",
        "r15b",
};

char *escapeSequences[NUMBER_OF_ESCAPE_SEQUENCES] = {
        "\\n", "\\s", "space", "\\t", "\\f", "\\b", "\\v", "\\\"", "\\?", "\\\\"
};
char *translatedEscapeSequences[NUMBER_OF_ESCAPE_SEQUENCES] = {
        "\n", "' '", "' '", "\t", "\f", "\b", "\v", "\"", "\?", "\\"
};

/**
 * Compares the parameter against all possible values in a pre-defined list
 * @param parameter the given parameter
 * @param array the array to be compared against
 * @param arraySize the number of items in this array
 * @return 1 if it is present in the array, 0 otherwise
 */
int isInArray(char* parameter, char **array, int arraySize) {
    for(int i = 0; i < arraySize; i++) {
        if(strcmp(parameter, array[i]) == 0) {
            return 1;
        }
    }
    return 0;
}

/**
 * Translates a given escape sequence into a format that is usable in assembly code
 * @param parameter the given parameter
 * @param parameterNum the parameter number
 * @param parsedCommand the struct of the current command
 */
void translateEscapeSequence(char *parameter, int parameterNum, struct parsedCommand *parsedCommand) {
    //Allocate new memory
    char *modifiedParameter = malloc(2);
    if(modifiedParameter == NULL) {
        fprintf(stderr, "Critical error: Memory allocation for command parameter failed!");
        exit(EXIT_FAILURE);
    }

    //Get the index that the escape sequence is in
    int i = 0;
    while(strcmp(parameter, escapeSequences[i]) != 0) {
        i++;
    }

    //Set the value to the translated escape sequence
    strcpy(modifiedParameter, translatedEscapeSequences[i]);

    //Free the old memory
    free(parameter);

    //Set the reference to the new memory block
    (*parsedCommand).parameters[parameterNum] = modifiedParameter;
}

/**
 * Translates a given character into a format that is usable in assembly code
 * @param parameter the given parameter
 * @param parameterNum the parameter number
 * @param parsedCommand the struct of the current command
 */
void translateCharacter(char *parameter, int parameterNum, struct parsedCommand *parsedCommand) {
    //Allocate new memory
    char *modifiedParameter = malloc(4);
    if(modifiedParameter == NULL) {
        fprintf(stderr, "Critical error: Memory allocation for command parameter failed!");
        exit(EXIT_FAILURE);
    }

    //Set the value to the provided character, surrounded by ''
    modifiedParameter[0] = '\'';
    modifiedParameter[1] = parameter[0];
    modifiedParameter[2] = '\'';
    modifiedParameter[3] = '\0';

    //Free the old memory
    free(parameter);

    //Set the reference to the new memory block
    (*parsedCommand).parameters[parameterNum] = modifiedParameter;
}

/**
 * Checks if the given parsed command adheres to the parameter constraints (i.e. the parameters are legal)
 * @param parsedCommand
 * @return 1 if the parameters are valid, 0 otherwise
 */
int hasValidParameters(struct parsedCommand *parsedCommand) {
    printDebugMessage("Starting parameter validity check", "");
    for(int parameterNum = 0; parameterNum < commandList[(*parsedCommand).opcode].usedParameters; parameterNum++) {
        //Get the current parameter
        char* parameter = (*parsedCommand).parameters[parameterNum];
        printDebugMessage("\tChecking parameter", parameter);

        //Get the allowed parameter types for this parameter
        uint8_t allowedTypes = commandList[(*parsedCommand).opcode].allowedParamTypes[parameterNum];

        if((allowedTypes & 0b1) != 0) { //64 and 32 bit registers
            if(isInArray(parameter, registers_64_32_bit, NUMBER_OF_64_32_BIT_REGISTERS)) {
                printDebugMessage("\t\tParameter is a 64 bit / 32 bit register", "");
                continue;
            }
            printDebugMessage("\t\tParameter is not a 64 bit / 32 bit register", "");
        }
        if((allowedTypes & 0b10) != 0) { //16 bit registers
            if(isInArray(parameter, registers_16_bit, NUMBER_OF_16_BIT_REGISTERS)) {
                printDebugMessage("\t\tParameter is a 16 bit register", "");
                continue;
            }
            printDebugMessage("\t\tParameter is not a 16 bit register", "");
        }
        if((allowedTypes & 0b100) != 0) { //8 bit registers
            if(isInArray(parameter, registers_8_bit, NUMBER_OF_8_BIT_REGISTERS)) {
                printDebugMessage("\t\tParameter is an 8 bit register", "");
                continue;
            }
            printDebugMessage("\t\tParameter is not an 8 bit register", "");
        }
        if((allowedTypes & 0b1000) != 0) { //Decimal number
            char* endPtr;
            strtol(parameter, &endPtr, 10);
            //If the end pointer does not point to the end of the string, there was an illegal character
            if(*endPtr == '\0') {
                printDebugMessage("\t\tParameter is a decimal number", "");
                continue;
            }
            printDebugMessage("\t\tParameter is not a decimal number", "");
        }
        if((allowedTypes & 0b10000) != 0) { //Characters (including escape sequences)
            //Check if any of the escape sequences match
            if(isInArray(parameter, (char **) escapeSequences, NUMBER_OF_ESCAPE_SEQUENCES)) {
                translateEscapeSequence(parameter, parameterNum, parsedCommand);
                printDebugMessage("\t\tParameter is an escape sequence and has been translated", "");
                continue;
            //If not, check if the parameter is only one character
            } else if(strlen(parameter) == 1) {
                translateCharacter(parameter, parameterNum, parsedCommand);
                printDebugMessage("\t\tParameter is a character, translated to:", (*parsedCommand).parameters[parameterNum]);
                continue;
            }
            printDebugMessage("\t\tParameter is neither a character nor an escape sequence", "");
        }
        if((allowedTypes & 0b1000000) != 0) { //Monke Jump label
            //Iterate through each character and check if it is either a U or an A
            //Define variables that are set to 1 if either a U or an A are found. That way, you can check if both of them occurred at least once
            uint8_t a_used = 0;
            uint8_t u_used = 0;
            uint8_t unexpectedCharacter = 0;
            for(int i = 0; i < strlen(parameter); i++) {
                if(parameter[i] == 'a') {
                    a_used = 1;
                } else if(parameter[i] == 'u') {
                    u_used = 1;
                } else {
                    unexpectedCharacter = 1;
                    break;
                }
            }

            if(a_used == 1 && u_used == 1 && unexpectedCharacter == 0) {
                printDebugMessage("\t\tParameter is a valid Monke jump label", "");
                continue;
            }
            printDebugMessage("\t\tParameter is not a valid Monke jump label", "");
        }
        if((allowedTypes & 0b10000000) != 0) { //Function name
            uint8_t unexpectedCharacter = 0;
            for(int i = 0; i < strlen(parameter); i++) {
                char character = parameter[i];
                if(i == 0 && character >= '0' && character <= '9') {
                    unexpectedCharacter = 1;
                    printDebugMessage("\t\tParameter is not a valid function name, as there is a number in the first position", "");
                    break;
                }
                if(!(character == '_' || character == '$' || character == '.' || (character >= '0' && character <= '9') || (character >= 'A' && character <= 'Z') || (character >= 'a' && character <= 'z'))) {
                    unexpectedCharacter = 1;
                    printDebugMessageWithNumber("\t\tParameter is not a valid function name, unexpected character found at position", i);
                    break;
                }
            }
            if(unexpectedCharacter == 0) {
                printDebugMessage("\t\tParameter is a valid function name", "");
                continue;
            }
        }
        printDebugMessage("No checks succeeded, invalid parameter!", "");
        printSyntaxError("Invalid parameter provided", parameter, (*parsedCommand).lineNum);
        return 0;

    }
    return 1;
}