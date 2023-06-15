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

//Used to pseudo-random generation when using bully mode
extern uint64_t computedIndex;
extern char* functionNames[];
extern unsigned numFunctionNames;

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

uint8_t getRegisterSize(uint8_t paramType) {
    switch(paramType) {
        case PARAM_REG8:
            return 8;
        case PARAM_REG16:
            return 16;
        case PARAM_REG32:
            return 32;
        case PARAM_REG64:
            return 64;
        default:
            return 0;
    }
}

/**
 * Returns a random register for the specified size. Returned value may be NULL
 */
char* getRandomRegister(uint8_t paramType) {
    switch(paramType) {
        case PARAM_REG64:
            return strdup(registers_64_bit[computedIndex % NUMBER_OF_64_BIT_REGISTERS]);
        case PARAM_REG32:
            return strdup(registers_32_bit[computedIndex % NUMBER_OF_32_BIT_REGISTERS]);
        case PARAM_REG16:
            return strdup(registers_16_bit[computedIndex % NUMBER_OF_16_BIT_REGISTERS]);
        case PARAM_REG8:
            return strdup(registers_8_bit[computedIndex % NUMBER_OF_8_BIT_REGISTERS]);
        default:
            return NULL;
    }
}

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
    CHECK_ALLOC(modifiedParameter);

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
    CHECK_ALLOC(modifiedParameter);

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
    printNote("the following parameter types are allowed: %s", true, 1, allowedParamsString);
}

/**
 * Checks if the given parsed command adheres to the parameter constraints (i.e. the parameters are legal)
 */
void checkParameters(struct parsedCommand *parsedCommand, char* inputFileName, struct compileState* compileState) {
    printDebugMessage(compileState->logLevel, "Starting parameter validity check", 0);
    uint8_t usedParameters = commandList[(*parsedCommand).opcode].usedParameters;
    for(uint8_t parameterNum = 0; parameterNum < usedParameters; parameterNum++) {
        //Get the current parameter
        char* parameter = (*parsedCommand).parameters[parameterNum];
        printDebugMessage( compileState->logLevel, "\tChecking parameter %s", 1, parameter);

        //Get the allowed parameter types for this parameter
        uint8_t allowedTypes = commandList[(*parsedCommand).opcode].allowedParamTypes[parameterNum];

        if((allowedTypes & PARAM_REG64) != 0) { //64 bit registers
            if(isInArray(parameter, registers_64_bit, NUMBER_OF_64_BIT_REGISTERS)) {
                printDebugMessage( compileState->logLevel, "\t\tParameter is a 64 bit register", 0);
                parsedCommand->paramTypes[parameterNum] = PARAM_REG64;
                continue;
            }
            printDebugMessage( compileState->logLevel, "\t\tParameter is not a 64 bit register", 0);
        }
        if((allowedTypes & PARAM_REG32) != 0) { //32 bit registers
            if(isInArray(parameter, registers_32_bit, NUMBER_OF_32_BIT_REGISTERS)) {
                printDebugMessage( compileState->logLevel, "\t\tParameter is a 32 bit register", 0);
                parsedCommand->paramTypes[parameterNum] = PARAM_REG32;
                continue;
            }
            printDebugMessage(compileState->logLevel, "\t\tParameter is not a 32 bit register", 0);
        }
        if((allowedTypes & PARAM_REG16) != 0) { //16 bit registers
            if(isInArray(parameter, registers_16_bit, NUMBER_OF_16_BIT_REGISTERS)) {
                printDebugMessage(compileState->logLevel, "\t\tParameter is a 16 bit register", 0);
                parsedCommand->paramTypes[parameterNum] = PARAM_REG16;
                continue;
            }
            printDebugMessage(compileState->logLevel, "\t\tParameter is not a 16 bit register", 0);
        }
        if((allowedTypes & PARAM_REG8) != 0) { //8 bit registers
            if(isInArray(parameter, registers_8_bit, NUMBER_OF_8_BIT_REGISTERS)) {
                printDebugMessage(compileState->logLevel, "\t\tParameter is an 8 bit register", 0);
                parsedCommand->paramTypes[parameterNum] = PARAM_REG8;
                continue;
            }
            printDebugMessage(compileState->logLevel, "\t\tParameter is not an 8 bit register", 0);
        }
        if((allowedTypes & PARAM_DECIMAL) != 0) { //Decimal number
            char* endPtr;
            long long int number = strtoll(parameter, &endPtr, 10);
            //If the end pointer does not point to the end of the string, there was an illegal character
            if(*endPtr == '\0') {
                printDebugMessage(compileState->logLevel, "\t\tParameter is a decimal number", 0);
                if(parsedCommand->isPointer == parameterNum + 1) {
                    if(compileState->compileMode != bully) {
                        printError(inputFileName, parsedCommand->lineNum, compileState, "a decimal number cannot be a pointer", 0);
                    } else {
                        //Well then it's not a pointer :shrug:
                        parsedCommand->isPointer = 0;
                    }
                }
                if(number == 69) {
                    printNiceASCII();
                } else if(number == 420) {
                    printBongASCII();
                }
                parsedCommand->paramTypes[parameterNum] = PARAM_DECIMAL;
                continue;
            }
            printDebugMessage(compileState->logLevel, "\t\tParameter is not a decimal number", 0);
        }
        if((allowedTypes & PARAM_CHAR) != 0) { //Characters (including escape sequences) / ASCII-code
            //Check if any of the escape sequences match
            if(isInArray(parameter, (char **) escapeSequences, NUMBER_OF_ESCAPE_SEQUENCES)) {
                translateEscapeSequence(parameter, parameterNum, parsedCommand);
                printDebugMessage(compileState->logLevel, "\t\tParameter is an escape sequence and has been translated", 0);
                if(parsedCommand->isPointer == parameterNum + 1) {
                    if(compileState->compileMode != bully) {
                        printError(inputFileName, parsedCommand->lineNum, compileState, "a character cannot be a pointer", 0);
                    } else {
                        //Well then it's not a pointer :shrug:
                        parsedCommand->isPointer = 0;
                    }
                }
                parsedCommand->paramTypes[parameterNum] = PARAM_CHAR;
                continue;
            //If not, check if the parameter is only one character
            } else if(strlen(parameter) == 1) {
                translateCharacter(parameter, parameterNum, parsedCommand);
                printDebugMessage(compileState->logLevel, "\t\tParameter is a character, translated to: %s", 1, (*parsedCommand).parameters[parameterNum]);
                if(parsedCommand->isPointer == parameterNum + 1) {
                    if(compileState->compileMode != bully) {
                        printError(inputFileName, parsedCommand->lineNum, compileState, "a character cannot be a pointer", 0);
                    } else {
                        //Well then it's not a pointer :shrug:
                        parsedCommand->isPointer = 0;
                    }
                }
                parsedCommand->paramTypes[parameterNum] = PARAM_CHAR;
                continue;
            }
            printDebugMessage(compileState->logLevel, "\t\tParameter is neither a character nor an escape sequence", 0);

            //Now check if it is an ASCII-code
            char* endPtr;
            long result = strtol(parameter, &endPtr, 10);
            //If the end pointer does not point to the end of the string, there was an illegal character
            //We allow values greater than 128 so that it is possible to print unicode in multiple steps. See https://play.golang.org/p/TojzlTMIcJe
            if(*endPtr == '\0' && result >= 0 && result <= 255) {
                printDebugMessage(compileState->logLevel, "\t\tParameter is an ASCII-code", 0);
                if(parsedCommand->isPointer == parameterNum + 1) {
                    if(compileState->compileMode != bully) {
                        printError(inputFileName, parsedCommand->lineNum, compileState, "a character cannot be a pointer", 0);
                    } else {
                        //Well then it's not a pointer :shrug:
                        parsedCommand->isPointer = 0;
                    }
                }
                parsedCommand->paramTypes[parameterNum] = PARAM_CHAR;
                continue;
            }
            printDebugMessage(compileState->logLevel, "\t\tParameter is not an ASCII-code", 0);
        }
        if((allowedTypes & PARAM_MONKE_LABEL) != 0) { //Monke Jump label
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
                printDebugMessage(compileState->logLevel, "\t\tParameter is a valid Monke jump label", 0);
                if(parsedCommand->isPointer == parameterNum + 1) {
                    if(compileState->compileMode != bully) {
                        printError(inputFileName, parsedCommand->lineNum, compileState,"a jump marker cannot be a pointer", 0);
                    } else {
                        //Well then it's not a pointer :shrug:
                        parsedCommand->isPointer = 0;
                    }
                }
                parsedCommand->paramTypes[parameterNum] = PARAM_MONKE_LABEL;
                continue;
            }
            printDebugMessage(compileState->logLevel, "\t\tParameter is not a valid Monke jump label", 0);
        }
        if((allowedTypes & PARAM_FUNC_NAME) != 0) { //Function name
            bool unexpectedCharacter = false;
            for(size_t i = 0; i < strlen(parameter); i++) {
                char character = parameter[i];
                if(i == 0 && character >= '0' && character <= '9') {
                    unexpectedCharacter = true;
                    printDebugMessage(compileState->logLevel, "\t\tParameter is not a valid function name, as there is a number in the first position", 0);
                    break;
                }
                if(!(character == '_' || character == '$' || character == '.' || (character >= '0' && character <= '9') || (character >= 'A' && character <= 'Z') || (character >= 'a' && character <= 'z'))) {
                    unexpectedCharacter = true;
                    printDebugMessage(compileState->logLevel, "\t\tParameter is not a valid function name, unexpected character found at position %lu", 1, i);
                    break;
                }
            }
            if(!unexpectedCharacter) {
                printDebugMessage(compileState->logLevel, "\t\tParameter is a valid function name", 0);
                if(parsedCommand->isPointer == parameterNum + 1) {
                    if(compileState->compileMode != bully) {
                        printError(inputFileName, parsedCommand->lineNum, compileState, "a function name cannot be a pointer", 0);
                    } else {
                        //Well then it's not a pointer :shrug:
                        parsedCommand->isPointer = 0;
                    }
                }
                parsedCommand->paramTypes[parameterNum] = PARAM_FUNC_NAME;
                continue;
            }
        }
        printDebugMessage(compileState->logLevel, "No checks succeeded, invalid parameter!", 0);
        if(compileState->compileMode != bully) {
            printError(inputFileName, parsedCommand->lineNum, compileState, "invalid parameter provided: \"%s\"", 1, parameter);
            if(compileState->compileMode != obfuscated) {
                printParameterUsageNote(allowedTypes);
            }
            parsedCommand->paramTypes[parameterNum] = 0;
        } else {
            /* To make this work, we need to replace this parameter with something that works
             * To create something semi-random, we use the same technique used in fileParser.c:
             * "computedIndex", which is dependent on the previous failed parameters, is used to pseudo-randomly
             * generate a valid parameter
             */
            for(size_t i = 0; i < strlen(parameter); i++){
                computedIndex = computedIndex * parameter[i] / 2;
            }

            //Choose a parameter. If it is not allowed, rotate the bitmask until we find a valid one
            uint8_t chosenParameter = 1 << (computedIndex % 8);
            while ((allowedTypes & chosenParameter) == 0) {
                if(chosenParameter == (1 << 7)) {
                    chosenParameter = 1;
                } else {
                    chosenParameter = chosenParameter << 1;
                }
            }

            char* newParam;
            switch(chosenParameter) {
                case PARAM_REG64:
                case PARAM_REG32:
                case PARAM_REG16:
                case PARAM_REG8:
                    newParam = getRandomRegister(chosenParameter);
                    break;
                case PARAM_DECIMAL:
                case PARAM_CHAR:
                    newParam = malloc(10);
                    CHECK_ALLOC(newParam);
                    sprintf(newParam, "%u", (unsigned) computedIndex % 128);
                    break;
                case PARAM_MONKE_LABEL:
                    newParam = malloc(10);
                    int j = 0;
                    CHECK_ALLOC(newParam);
                    unsigned length = computedIndex % 7 + 2;
                    for(unsigned i = 0; i < length; i++) {
                        newParam[j++] = (computedIndex % 2 == 0) ? 'u' : 'a';
                        computedIndex = computedIndex * 3 / 2;
                    }
                    newParam[length] = 0;
                    break;
                case PARAM_FUNC_NAME:
                    newParam = strdup(functionNames[computedIndex % numFunctionNames]);
                    break;
                default:
                    printInternalCompilerError("Random parameter generation unsupported for paramType %u", true, 1, chosenParameter);
                    exit(EXIT_FAILURE);
            }
            if(parsedCommand->isPointer == parameterNum + 1) {
                parsedCommand->isPointer = 0;
            }

            CHECK_ALLOC(newParam);
            //Free the old parameter
            free(parameter);
            //Set the new parameter
            parsedCommand->parameters[parameterNum] = newParam;
            parsedCommand->paramTypes[parameterNum] = chosenParameter;
        }
    }

    //We only compare parameters if there are at least two of them
    if(usedParameters >= 2) {
        //Now do some parameter checks
        //1: If a number and a register are used, they number must fit in the register
        //2: If a number and a 64 Bit register are used (and the command is not a mov-command), can the number be sign-extended from 32 Bits?
        if((parsedCommand->paramTypes[0] == PARAM_DECIMAL && PARAM_ISREG(parsedCommand->paramTypes[1])) ||
           (parsedCommand->paramTypes[1] == PARAM_DECIMAL && PARAM_ISREG(parsedCommand->paramTypes[0])))  {
            unsigned decimalIndex = (parsedCommand->paramTypes[0] == PARAM_DECIMAL) ? 0 : 1;
            unsigned regIndex = 1 - decimalIndex;
            unsigned regSize = getRegisterSize(parsedCommand->paramTypes[regIndex]);

            long long number = strtoll(parsedCommand->parameters[decimalIndex], NULL, 10);
            /*
             * To calculate the number of bits needed, we need to keep in mind that we use the two's complement. So there are two scenarios:
             *  - positive number: just check how many leading zeros there are. These zeros are not needed => 64 - clz
             *  - negative number: Meaning the last bit is a 1. Flip all bits. Now do the same calculation as before, but add 1 at the end,
             *                     since the number requires at least one bit at the end that is set to one
             */
            unsigned bitsNeeded = (number == 0) ? 1 : ((__builtin_clzll(number) > 0) ? 64 - __builtin_clzll(number) : 64 - __builtin_clzll(~number) + 1);
            if(bitsNeeded > regSize) {
                if(compileState->compileMode != bully) {
                    printError(inputFileName, parsedCommand->lineNum, compileState,
                               "invalid parameter combination: '%s' (%u bits) does not fit into register '%s' of size %u",
                               3, parsedCommand->parameters[decimalIndex], bitsNeeded, parsedCommand->parameters[regIndex], regSize);
                } else {
                    //We replace the number with something that is guaranteed to fit into all registers
                    char* newParam = malloc(10);
                    CHECK_ALLOC(newParam);
                    sprintf(newParam, "%u", (unsigned) computedIndex % 256);

                    free(parsedCommand->parameters[decimalIndex]);
                    parsedCommand->parameters[decimalIndex] = newParam;
                }
            //If command is not mov, are the last 33 Bits all 0 or all 1?
            } else if(commandList[parsedCommand->opcode].commandType != COMMAND_TYPE_MOV && regSize == 64 && !((number & 0xFFFFFFFF80000000) == 0 || (number | 0x7FFFFFFF) == -1)) {
                if(compileState->compileMode != bully) {
                    printError(inputFileName, parsedCommand->lineNum, compileState,
                               "invalid parameter combination: 64 Bit arithmetic operation commands require the decimal number to be sign-extendable from 32 Bits",0);
                } else {
                    //We just make the number shorter than 32 bits :bigBrain:
                    parsedCommand->parameters[decimalIndex][computedIndex % 32] = 0;

                    //Also, change computedIndex. Just because
                    computedIndex += (number & 0xFFF);
                }
            }
        }

        //3: If two (or potentially more) registers are used, they must be of the same size
        uint8_t currentReg = 0; // The first encountered register is set as the expected size. If 0, no register was found until now
        for (int i = 0; i < usedParameters; i++) { //Go over all parameters
            uint8_t paramType = parsedCommand->paramTypes[i];
            if (currentReg == 0) { //If no register was found yet...
                if (PARAM_ISREG(paramType) && parsedCommand->isPointer != i + 1) { //...and this parameter is a register...
                    currentReg = paramType; //...set it as the expected size
                }
            } else if (PARAM_ISREG(paramType) && parsedCommand->isPointer != i + 1 && paramType != currentReg) { //If we then find another register with differing size, throw an error
                if(compileState->compileMode != bully) {
                    printError(inputFileName, parsedCommand->lineNum, compileState,
                               "invalid parameter combination: cannot combine registers of different size", 0);
                } else {
                    //We just replace this register with one of the correct size
                    free(parsedCommand->parameters[i]);
                    parsedCommand->parameters[i] = getRandomRegister(currentReg);
                    CHECK_ALLOC(parsedCommand->parameters[i]);

                    parsedCommand->paramTypes[i] = currentReg;
                }
            }
        }

        //4: If there's a pointer parameter, is the other parameter a register? If not, we do not know the operand size, throw an error
        //This check is skipped in bully mode and done in translator.c, as fixing this issue requires to edit the translated assembly code
        if(compileState->compileMode != bully) {
            if (parsedCommand->isPointer != 0 && !PARAM_ISREG(parsedCommand->paramTypes[parsedCommand->isPointer % 2])) {
                printError(inputFileName, parsedCommand->lineNum, compileState,
                           "invalid parameter combination: operand size unknown", 0);
            }
        }
    }
}
