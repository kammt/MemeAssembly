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

#include "parameters.h"
#include "../logger/log.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

#define NUMBER_OF_8_BIT_REGISTERS 20
#define NUMBER_OF_64_BIT_REGISTERS 16
#define NUMBER_OF_32_BIT_REGISTERS 16
#define NUMBER_OF_16_BIT_REGISTERS 16
#define NUMBER_OF_ESCAPE_SEQUENCES 10

//Parameters as strings
const char* const paramNames[] = {"64 bit Register", "32 bit Register", "16 bit Register", "8 bit Register", "decimal number", "character", "monke jump marker name", "function name"};

extern struct command commandList[];

char *registers_64_bit[NUMBER_OF_64_BIT_REGISTERS] = {
        "rax",
        "rbx",
        "rcx",
        "rdx",
        "rdi",
        "rsi",
        "rsp",
        "rbp",
        "r8",
        "r9",
        "r10",
        "r11",
        "r12",
        "r13",
        "r14",
        "r15"
};

char *registers_32_bit[NUMBER_OF_32_BIT_REGISTERS] = {
        "eax",
        "ebx",
        "ecx",
        "edx",
        "edi",
        "esi",
        "esp",
        "ebp",
        "r8d",
        "r9d",
        "r10d",
        "r11d",
        "r12d",
        "r13d",
        "r14d",
        "r15d",
};

char *registers_16_bit[NUMBER_OF_16_BIT_REGISTERS] = {
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
        "'\\n'", "' '", "' '", "'\\t'", "'\\f'", "'\\b'", "'\\v'", "'\\\"'", "'\\?'", "'\\\\'"
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
    char *modifiedParameter = malloc(5);
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

void printParameterUsageNote(uint8_t allowedParams) {
    //First, we construct a string that lists all allowed parameter types for this parameter
    //The worst case is: all parameters are used, separated by commas (+8*2 characters) and \0 at the end (+1 character)
    size_t maxSize = 17;
    for(unsigned i = 0; i < 8; i++) {
        maxSize += strlen(paramNames[i]);
    }
    char allowedParamsString[maxSize];
    allowedParamsString[0] = '\0';

    //Iterate through all parameters. If parameter is allowed (nth bit is set), append its name (at index n) to the string
    uint8_t param = 1;
    for(uint8_t i = 0; i < 8; i++) {
        if((param & allowedParams) != 0) {
            strcat(allowedParamsString, paramNames[i]);
            strcat(allowedParamsString, ", ");
        }
        param = param << 1;
    }

    //We assume here that at least one parameter was added to the string. This means that there is an unnecessary comma and space at the end. We remove that by adding a \0
    allowedParamsString[strlen(allowedParamsString) - 2] = '\0';

    //Print it
    printNote("the following parameter types are allowed: %s", 1, allowedParamsString);
}

/**
 * Checks if the given parsed command adheres to the parameter constraints (i.e. the parameters are legal)
 */
void checkParameters(struct parsedCommand *parsedCommand, char* inputFileName, struct compileState* compileState) {
    printDebugMessage(compileState -> logLevel, "Starting parameter validity check", 0);
    for(int parameterNum = 0; parameterNum < commandList[(*parsedCommand).opcode].usedParameters; parameterNum++) {
        //Get the current parameter
        char* parameter = (*parsedCommand).parameters[parameterNum];
        printDebugMessage( compileState -> logLevel, "\tChecking parameter %s", 1, parameter);

        //Get the allowed parameter types for this parameter
        uint8_t allowedTypes = commandList[(*parsedCommand).opcode].allowedParamTypes[parameterNum];

        if((allowedTypes & REG64) != 0) { //64 bit registers
            if(isInArray(parameter, registers_64_bit, NUMBER_OF_64_BIT_REGISTERS)) {
                printDebugMessage( compileState -> logLevel, "\t\tParameter is a 64 bit register", 0);
                continue;
            }
            printDebugMessage( compileState -> logLevel, "\t\tParameter is not a 64 bit register", 0);
        }
        if((allowedTypes & REG32) != 0) { //32 bit registers
            if(isInArray(parameter, registers_32_bit, NUMBER_OF_32_BIT_REGISTERS)) {
                printDebugMessage( compileState -> logLevel, "\t\tParameter is a 32 bit register", 0);
                continue;
            }
            printDebugMessage(compileState -> logLevel, "\t\tParameter is not a 32 bit register", 0);
        }
        if((allowedTypes & REG16) != 0) { //16 bit registers
            if(isInArray(parameter, registers_16_bit, NUMBER_OF_16_BIT_REGISTERS)) {
                printDebugMessage(compileState -> logLevel, "\t\tParameter is a 16 bit register", 0);
                continue;
            }
            printDebugMessage(compileState -> logLevel, "\t\tParameter is not a 16 bit register", 0);
        }
        if((allowedTypes & REG8) != 0) { //8 bit registers
            if(isInArray(parameter, registers_8_bit, NUMBER_OF_8_BIT_REGISTERS)) {
                printDebugMessage(compileState -> logLevel, "\t\tParameter is an 8 bit register", 0);
                continue;
            }
            printDebugMessage(compileState -> logLevel, "\t\tParameter is not an 8 bit register", 0);
        }
        if((allowedTypes & DECIMAL) != 0) { //Decimal number
            char* endPtr;
            long int number = strtol(parameter, &endPtr, 10);
            //If the end pointer does not point to the end of the string, there was an illegal character
            if(*endPtr == '\0') {
                printDebugMessage(compileState -> logLevel, "\t\tParameter is a decimal number", 0);
                if(parsedCommand -> isPointer == parameterNum + 1) {
                    printError(inputFileName, parsedCommand -> lineNum, compileState, "a decimal number cannot be a pointer", 0);
                }
                if(number == 69 || number == 420) {
                    printNiceASCII();
                }
                continue;
            }
            printDebugMessage(compileState -> logLevel, "\t\tParameter is not a decimal number", 0);
        }
        if((allowedTypes & CHAR) != 0) { //Characters (including escape sequences) / ASCII-code
            //Check if any of the escape sequences match
            if(isInArray(parameter, (char **) escapeSequences, NUMBER_OF_ESCAPE_SEQUENCES)) {
                translateEscapeSequence(parameter, parameterNum, parsedCommand);
                printDebugMessage(compileState -> logLevel, "\t\tParameter is an escape sequence and has been translated", 0);
                if(parsedCommand -> isPointer == parameterNum + 1) {
                    printError(inputFileName, parsedCommand -> lineNum, compileState, "a character cannot be a pointer", 0);
                }
                continue;
            //If not, check if the parameter is only one character
            } else if(strlen(parameter) == 1) {
                translateCharacter(parameter, parameterNum, parsedCommand);
                printDebugMessage(compileState -> logLevel, "\t\tParameter is a character, translated to: %s", 1, (*parsedCommand).parameters[parameterNum]);
                if(parsedCommand -> isPointer == parameterNum + 1) {
                    printError(inputFileName, parsedCommand -> lineNum, compileState, "a character cannot be a pointer", 0);
                }
                continue;
            }
            printDebugMessage(compileState -> logLevel, "\t\tParameter is neither a character nor an escape sequence", 0);

            //Now check if it is an ASCII-code
            char* endPtr;
            long result = strtol(parameter, &endPtr, 10);
            //If the end pointer does not point to the end of the string, there was an illegal character
            //We allow values greater than 128 so that it is possible to print unicode in multiple steps. See https://play.golang.org/p/TojzlTMIcJe
            if(*endPtr == '\0' && result >= 0 && result <= 255) {
                printDebugMessage(compileState -> logLevel, "\t\tParameter is an ASCII-code", 0);
                if(parsedCommand -> isPointer == parameterNum + 1) {
                    printError(inputFileName, parsedCommand -> lineNum, compileState, "a character cannot be a pointer", 0);
                }
                continue;
            }
            printDebugMessage(compileState -> logLevel, "\t\tParameter is not an ASCII-code", 0);
        }
        if((allowedTypes & MONKE_LABEL) != 0) { //Monke Jump label
            //Iterate through each character and check if it is either a U or an A
            //Define variables that are set to 1 if either a U or an A are found. That way, you can check if both of them occurred at least once
            uint8_t a_used = 0;
            uint8_t u_used = 0;
            uint8_t unexpectedCharacter = 0;
            for(size_t i = 0; i < strlen(parameter); i++) {
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
                printDebugMessage(compileState -> logLevel, "\t\tParameter is a valid Monke jump label", 0);
                if(parsedCommand -> isPointer == parameterNum + 1) {
                    printError(inputFileName, parsedCommand -> lineNum, compileState, "a jump marker cannot be a pointer", 0);
                }
                continue;
            }
            printDebugMessage(compileState -> logLevel, "\t\tParameter is not a valid Monke jump label", 0);
        }
        if((allowedTypes & FUNC_NAME) != 0) { //Function name
            bool unexpectedCharacter = false;
            for(size_t i = 0; i < strlen(parameter); i++) {
                char character = parameter[i];
                if(i == 0 && character >= '0' && character <= '9') {
                    unexpectedCharacter = true;
                    printDebugMessage(compileState -> logLevel, "\t\tParameter is not a valid function name, as there is a number in the first position", 0);
                    break;
                }
                if(!(character == '_' || character == '$' || character == '.' || (character >= '0' && character <= '9') || (character >= 'A' && character <= 'Z') || (character >= 'a' && character <= 'z'))) {
                    unexpectedCharacter = true;
                    printDebugMessage(compileState -> logLevel, "\t\tParameter is not a valid function name, unexpected character found at position %lu", 1, i);
                    break;
                }
            }
            if(!unexpectedCharacter) {
                printDebugMessage(compileState -> logLevel, "\t\tParameter is a valid function name", 0);
                if(parsedCommand -> isPointer == parameterNum + 1) {
                    printError(inputFileName, parsedCommand -> lineNum, compileState, "a function name cannot be a pointer", 0);
                }
                continue;
            }
        }
        printDebugMessage(compileState -> logLevel, "No checks succeeded, invalid parameter!", 0);
        printError(inputFileName, parsedCommand -> lineNum, compileState, "invalid parameter provided: \"%s\"", 1, parameter);
        printParameterUsageNote(allowedTypes);
    }
}
