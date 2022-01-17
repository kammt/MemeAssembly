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

#include "parser.h"
#include <stdio.h>
#include <string.h>

#include "../logger/log.h"
#include "../commands.h"

#ifdef WINDOWS
/* strtok_r does not exist on Windows and instead is strtok_s. Use a preprocessor directive to replace all occurrences */
# define strtok_r strtok_s
#endif


char *line = NULL;
size_t len = 0;
ssize_t lineLength;

extern struct command commandList[];

unsigned int min(unsigned int one, unsigned int two) {
    if(one > two) {
        return two;
    } else {
        return one;
    }
}

/**
 * Removes the \n from a string if it is present at the end of the string
 */
void removeLineBreaksAndTabs() {
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
int isLineOfInterest() {
    //To support tabbed comments, we need to determine when the text actually starts
    int i = 0;
    while (line[i] == '\t' || line[i] == ' ') {
        i++; //Increase our variable as long as there are only tabs or spaces
    }

    if(lineLength != i + 1 && lineLength != 1 && strncmp((line + i), commentStart, min((unsigned int) lineLength, strlen(commentStart))) != 0) {
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
    //In case we just wrote an EOF, replace it with \n
    *(result - 1) = '\n';

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

    while((lineLength = getLine(&line, &len, inputFile)) != -1) {
        if(isLineOfInterest() == 1) {
            loc++;
        }
    }

    printDebugMessageWithNumber("The number of lines are", (int) loc);
    printDebugMessage("Rewinding source pointer", "");
    rewind(inputFile);
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

struct parsedCommand parseLine(int lineNum) {
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
            printDebugMessage("\tcomparing with", commandToken);
            //If the pattern of the command at this position is only 'p', it is a parameter, save it into the struct
            if(strlen(commandToken) == 1 && commandToken[0] == 'p') {
                printDebugMessage("\t\tInterpreting as parameter:", lineToken);

                char *variable = malloc(strlen(lineToken) + 1);
                if(variable == NULL) {
                    fprintf(stderr, "Critical error: Memory allocation for command parameter failed!");
                    exit(EXIT_FAILURE);
                }
                strcpy(variable, lineToken);
                parsedCommand.parameters[numberOfParameters++] = variable;

                //If the line after this parameter contains "do you know de wey", mark it as a pointer
                if(strlen(savePtrLine) >= strlen(pointerSuffix) && strncmp(pointerSuffix, savePtrLine, strlen(pointerSuffix)) == 0) {
                    printDebugMessage("\t\t\t'do you know de wey' was found, interpreting as pointer", "");
                    //If another parameter is already marked as a variable, print an error
                    if(parsedCommand.isPointer != 0) {
                        printSemanticError("Only one parameter is allowed to be a pointer", lineNum);
                    }
                    parsedCommand.isPointer = (uint8_t) numberOfParameters;
                    //Move the save pointer so that "do you know de wey" is not tokenized by strtok_r
                    savePtrLine += strlen(pointerSuffix);
                }
            } else if(strcmp(commandToken, lineToken) != 0) {
                //If both tokens do not match, try the next command
                printDebugMessage("\t\tMatching failed, attempting to match next command", "");
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
            printDebugMessage("\t\tMatching failed, lineToken is NULL while commandToken is not. Attempting to match next command", "");
            freeAllocatedMemory(parsedCommand, numberOfParameters);
            continue;
        //If the current token is 'or' and the rest of the string is only 'draw 25', then set the opcode as "or draw 25" and return
        } else if(strcmp(lineToken, orDraw25Start) == 0 && strlen(savePtrLine) == strlen(orDraw25End) && strncmp(orDraw25End, savePtrLine, strlen(orDraw25End)) == 0) {
            printDebugMessage("\t\t'or draw 25' was found, replacing opcode", "");
            //Before we replace the opcode though, we need to free memory that was allocated for parameters
            for(int j = 0; j < commandList[i].usedParameters; j++) {
                free(parsedCommand.parameters[j]);
            }
            //Now we can change the opcode and return the struct
            parsedCommand.opcode = OR_DRAW_25_OPCODE;
            return parsedCommand;
        }
    }

    parsedCommand.opcode = INVALID_COMMAND_OPCODE;
    printSyntaxError("Failed to parse command:", line, lineNum);
    //Any error will increase the "compilationErrors" variable in log.c, meaning that we can safely return something that doesn't make sense
    //We don't exit immediately because we want to print every error possible
    return parsedCommand;
}

struct commandsArray parseCommands(FILE *inputFile) {
    //First, we create an array of command structs
    size_t loc = getLinesOfCode(inputFile);
    struct parsedCommand *commands = calloc(sizeof(struct parsedCommand), (size_t) loc);
    if(commands == NULL) {
        fprintf(stderr, "Critical Error: Memory allocation for command parsing failed");
        exit(EXIT_FAILURE);
    }
    printDebugMessage("Struct array was created successfully", "");

    //Iterate through the file again, this time parsing each line of interest and adding it to our command struct array
    int i = 0; //The number of structs in the array
    int lineNumber = 1; //The line number we are currently on. We differentiate between number of commands and number of lines to print the correct line number in case of an error

    //Parse the file line by line
    while((lineLength = getLine(&line, &len, inputFile)) != -1) {
        //Check if the line contains actual code or if it's empty/contains comments
        if(isLineOfInterest() == 1) {
            //Remove \n from the end of the line
            removeLineBreaksAndTabs();
            printDebugMessage("Parsing line:", line);
            //Parse the command and add the returned struct into the array
            *(commands + i) = parseLine(lineNumber);
            //Increase our number of structs in the array
            i++;
        }
        lineNumber++;
    }

    struct commandsArray result = {
            commands, loc, 0
    };

    return result;
}