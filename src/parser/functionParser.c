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

#include <string.h>
#include "functionParser.h"
#include "../logger/log.h"

extern const struct command commandList[];
char *functionNames[] = {"mprotect", "kill", "signal", "raise", "dump", "atoi",
                         "generateExcellence", "isExcellent", "memeify", "uwufy", "test",
                         "helloWorld", "snake_case_sucks", "gets", "uwu", "skillIssue"};
unsigned numFunctionNames = sizeof(functionNames) / sizeof (char*);

/**
 * Creates a function struct by starting at the function definition and then traversing the
 * command array until a return statement, new function definition or end of array is found
 * @param commandsArray a pointer to the commands array created by the parser
 * @param functionStartAtIndex at which index of the array the function definition is
 * @param functionDeclarationOpcode the opcode of the function declaration command. The three return commands must be the three consecutive opcodes
 * @return a function struct containing all parsed information
 */
struct function parseFunction(struct commandsArray commandsArray, char* inputFileName, size_t functionStartAtIndex, struct compileState* compileState) {
    struct parsedCommand functionStart = *(commandsArray.arrayPointer + functionStartAtIndex);

    //Define the structs
    struct function function;
    function.definedInLine = (size_t) functionStart.lineNum;
    function.definedInFile = inputFileName;

    printDebugMessage(compileState->logLevel, "\tParsing function:", 1, functionStart.parameters[0]);

    size_t index = 1;
    size_t functionEndIndex = 0; //This points to the last found return-statement and is 0 if no return statement was found until now

    //Iterate through all commands until a return statement is found or the end of the array is reached
    while (functionStartAtIndex + index < commandsArray.size) {
        struct parsedCommand parsedCommand = *(commandsArray.arrayPointer + (functionStartAtIndex + index));
        //Get the opcode
        uint8_t opcode = parsedCommand.opcode;

        //Is this a new function definition
        if(commandList[opcode].commandType == COMMAND_TYPE_FUNC_DEF) {
            //If the previous command was a return statement, everything is fine. But if not, then we have a new function
            //definition, while the previous function did not return yet
            if(functionEndIndex != functionStartAtIndex + index - 1) {
                if(compileState->compileMode != bully) {
                    //Throw an error since the last statement was not a return
                    printError(inputFileName, parsedCommand.lineNum, compileState,
                               "expected a return statement, but got a new function definition", 0);
                }
            }
            break;
        } else if(commandList[opcode].commandType == COMMAND_TYPE_FUNC_RETURN) { //command is a return statement
            functionEndIndex = functionStartAtIndex + index;
        }
        index++;
    }
    printDebugMessage(compileState->logLevel, "\t\tIteration stopped at index %lu", 1, index);

    if(functionEndIndex == 0 && compileState->compileMode != bully) {
        printError(inputFileName, functionStart.lineNum, compileState, "function does not return", 0);
    }

    //Our function definition is also a command, hence there are functionEndIndex - functionStartAtIndex + 1 commands
    function.numberOfCommands = (functionEndIndex != 0) ? (functionEndIndex - functionStartAtIndex) + 1 : 1;
    return function;
}

void parseFunctions(struct file* fileStruct, struct commandsArray commandsArray, struct compileState* compileState) {
    //First, count how many function definitions there are
    size_t functionDefinitions = 0;
    for (size_t i = 0; i < commandsArray.size; ++i) {
        if(commandList[commandsArray.arrayPointer[i].opcode].commandType == COMMAND_TYPE_FUNC_DEF) {
            functionDefinitions++;
        }
    }
    printDebugMessage(compileState->logLevel, "Number of functions: %lu", 1, functionDefinitions);

    //Now we create our array of functions
    int functionArrayIndex = 0;
    struct function *functions = calloc(sizeof(struct function), functionDefinitions);
    CHECK_ALLOC(functions);

    //We now traverse the commands array again, this time parsing the functions
    size_t commandArrayIndex = 0; //At which command we currently are
    while (commandArrayIndex < commandsArray.size) {
        /*
         * Here, we have not found another function definition yet. Traverse over all commands.
         * - If they are not function definitions, we need to check if bully mode is on
         *    - if not, just throw a compiler error for each command that does not belong to a function
         *    - if so, those "orphaned" commands are added to newly created functions, with a random function name
         * - If they are, break and start parsing that function
         */
        size_t startIndex = commandArrayIndex;
        bool orphanedCommands = false;
        for (; commandArrayIndex < commandsArray.size; commandArrayIndex++) {
            if (commandList[commandsArray.arrayPointer[commandArrayIndex].opcode].commandType != COMMAND_TYPE_FUNC_DEF) {
                orphanedCommands = true;
                if(compileState->compileMode != bully) {
                    printError(fileStruct->fileName, commandsArray.arrayPointer[commandArrayIndex].lineNum,
                               compileState, "command does not belong to any function", 0);
                }
            } else {
                break;
            }
        }

        //If we're in bully mode and there were orphaned commands, then they range from startIndex to commandArrayIndex - 1
        //Inject a fake function with those commands
        if(compileState->compileMode == bully && orphanedCommands) {
            char* funcName = functionNames[commandArrayIndex % (sizeof(functionNames) / sizeof(char*))];

            //We create two extra commands (function definition and return)
            size_t numCommands = commandArrayIndex - startIndex + 2;
            struct parsedCommand *commands = calloc(numCommands, sizeof(struct parsedCommand));
            //There is one more function, so resize our functions array
            //We cannot use reallocarray(), since it does not exist on MacOS/Windows :(
            size_t functionsArraySize = 0;
            if (__builtin_umull_overflow(++functionDefinitions, sizeof(struct function), &functionsArraySize)) {
                fprintf(stderr, "Critical error: Memory allocation for command parameter failed!");
                exit(EXIT_FAILURE);
            }
            functions = realloc(functions, functionsArraySize);
            CHECK_ALLOC(functions);
            CHECK_ALLOC(commands);

            //Copy over the commands
            //The first one is a function definition
            commands[0].opcode = 0;
            commands[0].lineNum = 69; //That doesn't matter, there are no error messages anyway
            commands[0].parameters[0].str = funcName;
            commands[0].translate = true;

            //memcpy the other commands
            //We don't check for mult-overflow here, since numCommands*sizeof(...) was performed earlier in calloc, which succeeded => no overflow
            memcpy(commands + 1, &commandsArray.arrayPointer[startIndex], (numCommands - 2) * sizeof(struct parsedCommand));

            //The last one is a return
            commands[numCommands - 1].opcode = 1;
            commands[numCommands - 1].lineNum = 420;
            commands[numCommands - 1].translate = true;


            //Set the function-struct accordingly
            functions[functionArrayIndex].definedInFile = fileStruct->fileName;
            functions[functionArrayIndex].numberOfCommands = numCommands;
            functions[functionArrayIndex].commands = commands;

            //Increase the index
            functionArrayIndex++;
        }

        if(commandArrayIndex >= commandsArray.size) {
            break;
        }

        //Parse the function
        functions[functionArrayIndex] = parseFunction(commandsArray, fileStruct->fileName, commandArrayIndex, compileState);
        //Set the commands
        functions[functionArrayIndex].commands = &commandsArray.arrayPointer[commandArrayIndex];
        //Increase our command index so that it points to the next unparsed command
        commandArrayIndex += functions[functionArrayIndex].numberOfCommands;
        //Increase our function array index so that it points to the next uninitialised struct
        functionArrayIndex++;
    }

    //Update the file struct
    fileStruct->functionCount = functionDefinitions;
    fileStruct->functions = functions;
}
