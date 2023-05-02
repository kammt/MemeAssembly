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

#include "fileParser.h"
#include "parser.h"
#include <stdio.h>
#include <string.h>

#include "../logger/log.h"

#ifdef WINDOWS
/* strtok_r does not exist on Windows and instead is strtok_s. Use a preprocessor directive to replace all occurrences */
# define strtok_r strtok_s
#endif

extern struct command commandList[];

//Used to pseudo-random generation when using bully mode
size_t computedIndex = 69;

/**
 * Removes the \n from a string if it is present at the end of the string
 */
void removeLineBreaksAndTabs(char* line) {
    size_t i = strlen(line) - 1;
    while (line[i] == '\t' || line[i] == '\n' || line[i] == ' ') {
        i--;
    }
    line[i + 1] = '\0';
}

/**
 * Checks whether this line should be skipped or not
 * @return 1 if it is of interest (=> code), 0 if it should be skipped (e.g. it is a comment or it's empty)
 */
int isLineOfInterest(const char* line, ssize_t lineLength) {
    //To support tabbed comments, we need to determine when the text actually starts
    int i = 0;
    while (line[i] == '\t' || line[i] == ' ') {
        i++; //Increase our variable as long as there are only tabs or spaces
    }

    if(line[i] != '\n' && lineLength != i && strncmp((line + i), commentStart, strlen(commentStart)) != 0) {
        return 1;
    }
    return 0;
}

/**
 * A basic implementation of a getline-function.
 * Reads a full line and stores it in lineptr. If lineptr is NULL, it will be initialised and n will be set accordingly. When too little is allocated, 128 more Bytes will be added
 * @param lineptr a pointer to the current storage for lines. May be NULL
 * @param n must be the size of lineptr and will be updated by this function
 * @param stream from where to read
 * @return the number of bytes read if successful, -1 on error (e.g. EOF found, malloc/realloc failed)
 */
ssize_t getLine(char **restrict lineptr, size_t *restrict n, FILE *restrict stream) {
    if(*lineptr == NULL) {
        if(!(*lineptr = malloc(128))) {
            return -1;
        }
        *n = 128;
    }

    char* result = *lineptr;
    char c = 'c';
    ssize_t bytesRead = 0;
    while(c != '\n') {
        size_t readRes = fread(&c, 1, 1, stream);
        if(readRes == 0) {
            if(bytesRead == 0) {
                return -1;
            }
            //If EOF was found somewhere while reading from file, still return this string
            break;
        }

        *result = c;
        bytesRead++;
        result++;
        if(bytesRead == (ssize_t) *n) {
            *n += 128;
            *lineptr = realloc(*lineptr, *n);
            result = *lineptr + bytesRead;

            if(!*lineptr) {
                return -1;
            }
        }
    }

    //On Windows, file endings are done using \r\n. This means that there will be a \r at the end of every string, breaking the entire compiler
    //To fix this, check if the string ends with \r\n. If so, replace it with \n
    //However, we first need to check that we are not reading out of bounds
    if(((uintptr_t) result - 2 >= (uintptr_t) *lineptr) && *(result - 2) == '\r') {
        *(result - 2) = '\n';
        result--;
    }
    //We got to the end of a line or the end of a file, now append a '\0'
    *result = '\0';
    return bytesRead;
}

/**
 * Counts the lines of code in a memeasm file. A line counts as a line of code if:
 *  - it does not start with "What the hell happened here?" (a comment)
 *  - it is not empty
 * @param inputFile the input file
 * @return the number of lines of code in the file
 */
size_t getLinesOfCode(FILE *inputFile) {
    size_t loc = 0;
    char* line = NULL;
    size_t len;
    ssize_t lineLength;

    while((lineLength = getLine(&line, &len, inputFile)) != -1) {
        if(isLineOfInterest(line, lineLength) == 1) {
            loc++;
        }
    }

    rewind(inputFile);
    free(line);
    return loc;
}

/**
 * Frees the memory of variables after they are not needed anymore
 * @param parsedCommand the parsedCommand struct
 * @param numberOfParameters how many variables are currently in use (allocated)
 */
void freeAllocatedMemory(struct parsedCommand parsedCommand, int numberOfParameters) {
    for(int i = 0; i < numberOfParameters; i++) {
        free(parsedCommand.parameters[i]);
    }
}

/**
 * Parses a provided line of code and attempts to match it to a command
 * @param inputFileName the origin file. Required for error printing
 * @param line the line as a string
 * @param lineNum the line number in the origin file. Required for error printing
 * @param compileState the current compile state
 * @return
 */
struct parsedCommand parseLine(char* inputFileName, size_t lineNum, char* line, struct compileState* compileState) {
    struct parsedCommand parsedCommand;
    parsedCommand.lineNum = lineNum; //Set the line number
    parsedCommand.translate = 1;

    //Temporarily save the line on the stack to be able to restore when a comparison failed
    char lineCpy[strlen(line) + 1];
    strcpy(lineCpy, line);

    //Define save pointers for strtok_r
    char *savePtrLine;
    char *savePtrPattern;

    //Iterate through all possible commands
    for(int i = 0; i < NUMBER_OF_COMMANDS - 2; i++) {
        strcpy(lineCpy, line);
        savePtrLine = NULL;
        savePtrPattern = NULL;

        //Copy the current command pattern out of read-only memory
        char commandString[strlen(commandList[i].pattern) + 1];
        strcpy(commandString, commandList[i].pattern);

        //Tokenize both strings. Tabs at the beginning are allowed and should be ignored, hence they are a delimiter
        char *commandToken = strtok_r(commandString, " \t", &savePtrPattern);
        char *lineToken = strtok_r(lineCpy, " \t", &savePtrLine);

        int numberOfParameters = 0;
        parsedCommand.isPointer = 0;

        //Enter the comparison loop
        while (commandToken != NULL && lineToken != NULL) {
            printDebugMessage(compileState->logLevel, "\tcomparing with %s", 1, commandToken);

            if(strstr(commandToken, "{p}") != NULL) {
                //First check that everything before and after the {p} matches

                //The difference between the starting address of commandToken and the starting address of {p} is the number of characters before {p}
                size_t charsBefore = ((size_t) strstr(commandToken, "{p}") - (size_t) commandToken);
                //The difference between the total string length of commandToken and (charsBefore + 3)
                size_t charsAfter = (strlen(commandToken) - charsBefore - 3);

                if(strncmp(commandToken, lineToken, charsBefore) == 0 &&
                                        strncmp(commandToken + charsBefore + 3, lineToken + strlen(lineToken) - charsAfter, charsAfter) == 0) {
                    printDebugMessage(compileState->logLevel, "\t\t%s contains a parameter", 1, lineToken);

                    size_t parameterLength = strlen(lineToken) - charsBefore - charsAfter;

                    //When allocating space for a function name on MacOS, we need an extra _ -prefix, hence +2
                    char *variable = malloc(parameterLength + 2);
                    if (variable == NULL) {
                        fprintf(stderr, "Critical error: Memory allocation for command parameter failed!");
                        exit(EXIT_FAILURE);
                    }

                    #ifdef MACOS
                    if (i == 0 || i == 4) {
                        strcpy(variable, "_");
                        strncat(variable, lineToken, parameterLength);
                        variable[parameterLength + 1] = '\0';
                    } else {
                    #endif
                    //On Windows and Linux, only this line is executed
                    strncpy(variable, lineToken, parameterLength);
                    variable[parameterLength] = '\0';
                    #ifdef MACOS
                    }
                    #endif

                    parsedCommand.parameters[numberOfParameters++] = variable;

                    //If the line after this parameter contains "do you know de wey", mark it as a pointer
                    if (savePtrLine != NULL && strlen(savePtrLine) >= strlen(pointerSuffix) &&
                        strncmp(pointerSuffix, savePtrLine, strlen(pointerSuffix)) == 0) {
                        printDebugMessage(compileState->logLevel,
                                          "\t\t\t'do you know de wey' was found, interpreting as pointer", 0);
                        //If another parameter is already marked as a variable, print an error
                        //This error is skipped when bully mode is on
                        if (parsedCommand.isPointer != 0 && compileState->compileMode != bully) {
                            printError(inputFileName, lineNum, compileState,
                                       "Only one parameter is allowed to be a pointer", 0);
                        }
                        parsedCommand.isPointer = (uint8_t) numberOfParameters;
                        //Move the save pointer so that "do you know de wey" is not tokenized by strtok_r
                        savePtrLine += strlen(pointerSuffix);
                    }
                } else {
                    //Characters before and after parameter do not match
                    printDebugMessage( compileState->logLevel, "\t\tMatching failed - chars before or after {p} mismatching, attempting to match next command", 0);
                    freeAllocatedMemory(parsedCommand, numberOfParameters);
                    break;
                }
            } else if(strcmp(commandToken, lineToken) != 0) {
                //If both tokens do not match, try the next command
                printDebugMessage( compileState->logLevel, "\t\tMatching failed, attempting to match next command", 0);
                freeAllocatedMemory(parsedCommand, numberOfParameters);
                break;
            }

            //Tokenize both strings again. This time, only spaces are allowed
            commandToken = strtok_r(NULL, " ", &savePtrPattern);
            lineToken = strtok_r(NULL, " ", &savePtrLine);
        }

        if(commandToken != NULL && lineToken != NULL) {
            continue;
        }

        /*Either the line or the command pattern have reached their end. We now have to check what caused the problem
         * - if both are NULL, then there is no problem!
         * - if the commandToken is NULL, then we should have been at the end of the line. Check if the rest is equal to 'or draw 25'. If not, try the next command
         * - if the token is NULL, then the line is too short, try the next command
         */
        if(commandToken == NULL && lineToken == NULL) {
            parsedCommand.opcode = (uint8_t) i;
            return parsedCommand;
        } else if(lineToken == NULL) {
            printDebugMessage(compileState->logLevel, "\t\tMatching failed, lineToken is NULL while commandToken is not. Attempting to match next command", 0);
            freeAllocatedMemory(parsedCommand, numberOfParameters);
            continue;
            //If the current token is 'or' and the rest of the string is only 'draw 25', then set the opcode as "or draw 25" and return
        } else if(strcmp(lineToken, orDraw25Start) == 0 && strlen(savePtrLine) == strlen(orDraw25End) && strncmp(orDraw25End, savePtrLine, strlen(orDraw25End)) == 0) {
            printDebugMessage(compileState->logLevel, "\t\t'or draw 25' was found, replacing opcode", 0);
            //Before we replace the opcode though, we need to free memory that was allocated for parameters
            for(int j = 0; j < commandList[i].usedParameters; j++) {
                free(parsedCommand.parameters[j]);
            }
            //Now we can change the opcode and return the struct
            parsedCommand.opcode = OR_DRAW_25_OPCODE;
            return parsedCommand;
        }
    }

    if(compileState->compileMode == bully) {
        /*
         * In bully mode, we replace this non-working command with a valid one
         * But we want to make it deterministic, with a lot of possible commands,
         * so we take the sum of ascii characters in this line, and use this value
         * to create the random command
         *
         * The variable "computedIndex" is global, meaning that the value
         * is dependent on the previous illegal commands - *perfection*
         */
        const char* randomParams[] = {"rax", "rcx", "rbx", "r8", "r9", "r10", "r12", "rsp", "rbp", "ax", "al", "r8b", "r9d", "r14b", "99", "1238", "12", "420", "987654321", "8", "9", "69", "8268", "2", "_", "a", "b", "d", "f", "F", "sigreturn", "uaauuaa", "uau", "uu", "main", "gets", "srand", "mprotect", "au", "\\n", "space"};
        unsigned randomParamCount = sizeof randomParams / sizeof(char*);

        for(size_t i = 0; i < strlen(line); i++) {
            computedIndex += line[i];
        }
        computedIndex = ((computedIndex * lineNum) % 420) * inputFileName[0];

        parsedCommand.opcode = computedIndex % (NUMBER_OF_COMMANDS - 1);
        parsedCommand.parameters[0] = strdup(randomParams[computedIndex % randomParamCount]);
        parsedCommand.parameters[1] = strdup(randomParams[(computedIndex * inputFileName[0]) % randomParamCount]);

        if(!parsedCommand.parameters[0] || !parsedCommand.parameters[1]) {
            fprintf(stderr, "Critical error: Memory allocation for command parameter failed!");
            exit(EXIT_FAILURE);
        }
    } else {
        parsedCommand.opcode = INVALID_COMMAND_OPCODE;
        printError(inputFileName, lineNum, compileState, "Invalid command: \"%s\"", 1, line);
        //Any error will increase the "compilationErrors" variable in log.c, meaning that we can safely return something that doesn't make sense
        //We don't exit immediately because we want to print every error possible
    }
    return parsedCommand;
}


/**
 * Parses an input file line by line and fills a provided struct commandsArray
 */
void parseCommands(FILE *inputFile, char* inputFileName, struct compileState* compileState, struct commandsArray* commandsArray) {
    //Variable declarations
    char* line = NULL;
    size_t len = 0;
    ssize_t lineLength;

    //First, we create an array of command structs
    size_t loc = getLinesOfCode(inputFile);
    printDebugMessage(compileState->logLevel, "The number of lines are %lu", 1, loc);

    if(loc == 0) {
        if(compileState->compileMode != bully) {
            printError(inputFileName, 0, compileState, "file does not contain any commands", 0);
        }

        commandsArray->arrayPointer = NULL;
        commandsArray->size = 0;
        return;
    }

    struct parsedCommand *commands = calloc(sizeof(struct parsedCommand), loc);
    if(commands == NULL) {
        fprintf(stderr, "Critical Error: Memory allocation for command parsing failed");
        exit(EXIT_FAILURE);
    }
    printDebugMessage( compileState->logLevel, "Struct array was created successfully", 0);

    //Iterate through the file again, this time parsing each line of interest and adding it to our command struct array
    int i = 0; //The number of structs in the array
    int lineNumber = 1; //The line number we are currently on. We differentiate between number of commands and number of lines to print the correct line number in case of an error

    //Parse the file line by line
    while((lineLength = getLine(&line, &len, inputFile)) != -1) {
        //Check if the line contains actual code or if it's empty/contains comments
        if(isLineOfInterest(line, lineLength) == 1) {
            //Remove \n from the end of the line
            removeLineBreaksAndTabs(line);
            printDebugMessage( compileState->logLevel, "Parsing line: %s", 1, line);
            //Parse the command and add the returned struct into the array
            *(commands + i) = parseLine(inputFileName, lineNumber, line, compileState);
            //Increase our number of structs in the array
            i++;
        }
        lineNumber++;
    }

    commandsArray->size = loc;
    commandsArray->arrayPointer = commands;

    free(line);
}

