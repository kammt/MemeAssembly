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

#include "functionParser.h"
#include "../logger/log.h"

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
    function.name = functionStart.parameters[0];
    function.definedInLine = (size_t) functionStart.lineNum;
    function.definedInFile = inputFileName;

    printDebugMessage(compileState -> logLevel, "\tParsing function:", 1, functionStart.parameters[0]);

    size_t index = 1;
    size_t functionEndIndex = 0; //This points to the last found return-statement and is 0 if no return statement was found until now

    //Iterate through all commands until a return statement is found or the end of the array is reached
    while (functionStartAtIndex + index < commandsArray.size) {
        struct parsedCommand parsedCommand = *(commandsArray.arrayPointer + (functionStartAtIndex + index));
        //Get the opcode
        uint8_t opcode = parsedCommand.opcode;

        //Is this a new function definition
        if(opcode == 0) {
            //If there hasn't been a return statement until now, throw an error since there was no return statement until now
            if(functionEndIndex == 0) {
                printError(inputFileName, parsedCommand.lineNum, compileState, "expected a return statement, but got a new function definition", 0);
            }
            break;
        } if(opcode > 0 && opcode <= 3) { //Function is a return statement
            functionEndIndex = index;
        }
        index++;
    }
    printDebugMessage(compileState -> logLevel, "\t\tIteration stopped at index %lu", 1, index);

    if(functionEndIndex == 0) {
        printError(inputFileName, functionStart.lineNum, compileState, "function does not return", 0);
    }

    //Our function definition is also a command, hence there are functionEndIndex + 1 commands
    function.numberOfCommands = functionEndIndex + 1;
    return function;
}

void parseFunctions(struct file* fileStruct, struct commandsArray commandsArray, struct compileState* compileState) {
    //First, count how many function definitions there are
    size_t functionDefinitions = 0;
    for (size_t i = 0; i < commandsArray.size; ++i) {
        if(commandsArray.arrayPointer[i].opcode == 0) {
            functionDefinitions++;
        }
    }
    printDebugMessage(compileState -> logLevel, "Number of functions: %lu", 1, functionDefinitions);

    //Now we create our array of functions
    int functionArrayIndex = 0;
    struct function *functions = calloc(sizeof(struct function), functionDefinitions);
    if(functions == NULL) {
        fprintf(stderr, "Critical error: Memory allocation for command parameter failed!");
        exit(EXIT_FAILURE);
    }

    //We now traverse the commands array again, this time parsing the functions
    size_t commandArrayIndex = 0; //At which command we currently are
    while (commandArrayIndex < commandsArray.size) {
        //Here, we have not found another function definition yet. Traverse over all commands. If they are not function definitions, throw an error. If they are, break and start parsing that function
        for (; commandArrayIndex < commandsArray.size; commandArrayIndex++) {
            if(commandsArray.arrayPointer[commandArrayIndex].opcode != 0) {
                printError(fileStruct -> fileName, commandsArray.arrayPointer[commandArrayIndex].lineNum, compileState, "command does not belong to any function", 0);
            } else {
                break;
            }
        }

        if(commandArrayIndex >= commandsArray.size) {
            break;
        }

        //Parse the function
        functions[functionArrayIndex] = parseFunction(commandsArray, fileStruct -> fileName, commandArrayIndex, compileState);
        //Set the commands
        functions[functionArrayIndex].commands = &commandsArray.arrayPointer[commandArrayIndex];
        //Increase our command index so that it points to the next unparsed command
        commandArrayIndex += functions[functionArrayIndex].numberOfCommands + 1;
        //Increase our function array index so that it points to the next uninitialised struct
        functionArrayIndex++;
    }

    //Update the file struct
    fileStruct -> functionCount = functionDefinitions;
    fileStruct -> functions = functions;
}
