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

#include "parser.h"
#include "../commands.h"
#include "fileParser.h"
#include "../logger/log.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char *functionNames[] = {"mprotect", "kill", "signal", "raise", "dump", "atoi",
                         "generateExcellence", "isExcellent", "memeify", "uwufy", "test",
                         "helloWorld", "snake_case_sucks", "gets", "uwu", "skillIssue"};
unsigned numFunctionNames = sizeof(functionNames) / sizeof (char*);

void parseFile(struct file* fileStruct, FILE* inputFile, struct compileState* compileState) {
    //Variable declarations for getLine
    char* line = NULL;
    size_t len = 0;
    ssize_t lineLength;

    //We'll assume that an average MemeASM file has no more than 500 commands
    size_t commandsArraySize = 500;
    struct parsedCommand* parsedCommands = calloc(sizeof(struct parsedCommand), commandsArraySize);
    CHECK_ALLOC(parsedCommands);

    //We'll assume that an average MemeASM file has no more than 10 functions
    size_t functionsArraySize = 10;
    struct function* functions = calloc(sizeof(struct function), functionsArraySize);
    CHECK_ALLOC(functions);

    //Iterate through the file, parsing each line of interest and adding it to our command struct array
    size_t commandCount = 0; //The number of commands that were already parsed
    size_t functionCount = 0; //The number of functions we currently have
    struct function* currentFunction = NULL; //If we currently parse a function, then this points to it, NULL otherwise
    size_t numCommands = 0; //If we currently parse a function, we use this to count how many commands this function has
    size_t lineNumber = 1; //The line number we are currently on. We differentiate between number of commands and number of lines to print the correct line number in case of an error

    //Parse the file line by line
    while((lineLength = getLine(&line, &len, inputFile)) != -1) {
        //Check if the line contains actual code or if it's empty/contains comments
        if(isLineOfInterest(line, lineLength) == 1) {
            ///First, check if there is still enough space
            if(commandCount == commandsArraySize - 1) {
                //Alloc 500 more
                commandsArraySize += 500;
                parsedCommands = realloc(parsedCommands, commandsArraySize);
                CHECK_ALLOC(parsedCommands);
            }

            //Remove \n from the end of the line
            size_t lastChar = strlen(line) - 1;
            while (line[lastChar] == '\t' || line[lastChar] == '\n' || line[lastChar] == ' ') {
                lastChar--;
            }
            line[lastChar + 1] = '\0';

            ///Parse this line
            printDebugMessage( compileState->logLevel, "Parsing line: %s", 1, line);
            //Parse the command and add the returned struct into the array
            parseLine(fileStruct->fileName, lineNumber, line, parsedCommands + commandCount, compileState);
            struct parsedCommand currentCommand = *(parsedCommands + commandCount);

            ///function parsing
            //Increase the number of commands in our current function
            numCommands++;
            if(CMD_ISFUNCDEF(currentCommand.opcode)) {
                //There are two cases here: We currently don't parse a function (good), or we do (bad, the prior function did not return)
                if(currentFunction) {
                    //To make sure we don't brick anything in the following runs, finish the function struct
                    functions[functionCount].numberOfCommands = numCommands - 1; //numCommands already includes this command - which is part of the next function
                    numCommands = 0;
                    functionCount++;

                    if(compileState->compileMode != bully) {
                        printError(fileStruct->fileName, lineNumber, compileState,
                                   "Expected a return statement, but got a new function definition", 0);
                    } else {
                        goto parseFunctionDef;
                    }
                } else {
                    parseFunctionDef:
                    //We got a new function. But first: Do we have enough space?
                    if(functionCount == functionsArraySize - 1) {
                        //Add 10
                        functionCount += 10;
                        functions = realloc(functions, functionCount);
                        CHECK_ALLOC(functions);
                    }

                    //This pointer should start at the function definition - we'll fill in numCommands later
                    currentFunction = &functions[functionCount];
                    currentFunction->commands = parsedCommands + commandCount;
                }
            } else if(CMD_ISRET(currentCommand.opcode) && currentFunction) {
                //We found a return statement. If we currently parse a function, then nice! If not, this is an error condition.
                //This error is handled further down, as part of a "command is not part of any function" error message
                //Anyway, set the number of commands, and do some cleanup
                functions[functionCount].numberOfCommands = numCommands;
                currentFunction = NULL;
                numCommands = 0;
                functionCount++;
            } else if(!currentFunction) {
                //We currently don't parse a function, but found a command that is not a function definition => error
                //TODO bully mode
                printError(fileStruct->fileName, lineNumber, compileState, "command does not belong to any function", 0);
            }
            //Increase our number of commands in the array
            commandCount++;
        }
        lineNumber++;
    }
    //We're done reading the file - if we're still parsing a function, then that is an error, as that function did not return
    if(currentFunction) {
        //To make sure we don't brick anything coming after this, finish the function struct
        functions[functionCount].numberOfCommands = numCommands;
        numCommands = 0;
        functionCount++;

        if(compileState->compileMode != bully) {
            printError(fileStruct->fileName, lineNumber, compileState,"function does not return", 0);
        }
    }

    //Done! Now, just fill out file struct and we're ready to rumble on to the next file
    fileStruct->loc = commandCount;
    fileStruct->parsedCommands = parsedCommands;
    fileStruct->functions = functions;
    fileStruct->functionCount = functionCount;
}
