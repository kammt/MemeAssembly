#include "functions.h"
#include "../logger/log.h"

#include <stdlib.h>
#include <string.h>
#include <stdio.h>


struct function parseFunction(struct commandsArray *commandsArray, int functionStartAtIndex) {
    struct parsedCommand functionStart = *(commandsArray->arrayPointer + functionStartAtIndex);

    //Define the structs
    struct function function;

    //Allocate a new memory block for the function name
    char* functionName = malloc(strlen(functionStart.parameters[0]) + 1);
    if(functionName == NULL) {
        fprintf(stderr, "Critical error: Memory allocation for command parameter failed!");
        exit(EXIT_FAILURE);
    }

    //Copy the function name
    strcpy(functionName, functionStart.parameters[0]);

    //Set the function name
    function.name = functionName;

    //Set the line number
    function.definedInLine = functionStart.lineNum;

    printDebugMessage("\tParsing function:", functionName);

    int index = 1;
    uint8_t returnStatementFound = 0;
    //Iterate through all commands until a return statement is found or the end of the array is reached
    while (functionStartAtIndex + index < commandsArray -> size) {
        struct parsedCommand parsedCommand = *(commandsArray -> arrayPointer + (functionStartAtIndex + index));
        //Get the opcode
        uint8_t opcode = parsedCommand.opcode;

        if(opcode == 0) { //If it is a function definition, throw an error since there was no return statement until now
            printSemanticError("Expected a return statement, but got a new function definition", parsedCommand.lineNum);
            break;
        } if(opcode > 0 && opcode <= 3) { //Function is a return statement, abort
            returnStatementFound = 1;
            index++;
            break;
        }
        index++;
    }
    printDebugMessageWithNumber("\tIteration stopped at index", index);

    if(returnStatementFound == 0) {
        printSemanticError("No return statement found", functionStart.lineNum);
    }
    function.numberOfCommands = index - 1;
    return function;
}

/**
 * Checks if the function definitions are valid. This includes making sure that
 *  - no function names are used twice
 *  - no commands are outside of function definition
 *  - functions end with a return statement
 *  - there is a main function if it is supposed to be executable
 * @param commandsArray a pointer to the commandsArray created by the parser
 * @param checkForMainFunction whether or not it should be checked if a main function exists
 */
void checkFunctionValidity(struct commandsArray *commandsArray, uint8_t checkForMainFunction) {
    struct parsedCommand *arrayPointer = commandsArray->arrayPointer;

    //First, count how many function definitions there are
    int functionDefinitions = 0;
    for (int i = 0; i < commandsArray->size; ++i) {
        if(arrayPointer[i].opcode == 0) {
            functionDefinitions++;
        }
    }
    printDebugMessageWithNumber("Number of functions:", functionDefinitions);

    //Now we create our array of functions
    int functionArrayIndex = 0;
    struct function *functions = calloc(sizeof(struct function), functionDefinitions);
    if(functions == NULL) {
        fprintf(stderr, "Critical error: Memory allocation for command parameter failed!");
        exit(EXIT_FAILURE);
    }

    printDebugMessage("Starting function parsing", "");

    //We now traverse the commands array again, this time parsing the functions
    int commandArrayIndex = 0; //At which command we currently are
    while (commandArrayIndex < commandsArray->size) {
        for (; commandArrayIndex < commandsArray->size; commandArrayIndex++) {
            //At this point, we are in between function definitions, so any commands that are not function definitions are illegal
            if(arrayPointer[commandArrayIndex].opcode != 0) {
                printSemanticError("Statement does not belong to any function", commandsArray -> arrayPointer[commandArrayIndex].lineNum);
            } else {
                break;
            }
        }

        //Parse the function
        functions[functionArrayIndex] = parseFunction(commandsArray, commandArrayIndex);
        //Increase our command index so that it points to the next unparsed command
        commandArrayIndex += functions[functionArrayIndex].numberOfCommands + 1;
        //Increase our function array index so that it points to the next uninitialised struct
        functionArrayIndex++;
    }

    /*
     * We now need to check the function names, specifically
     * - that no function names appeared twice
     * - that a main function exists (if checkForMainFunction is set to 1)
     */
    uint8_t mainFunctionExists = 0;
    for(int i = 0; i < functionDefinitions; i++) {
        struct function function = functions[i];
        if(strcmp(function.name, "main") == 0) {
            mainFunctionExists = 1;
        }

        //We now traverse through all other function names and check if their names match. If so, print an error
        for(int j = i + 1; j < functionDefinitions; j++) {
            if(strcmp(function.name, functions[j].name) == 0) {
                printSemanticErrorWithExtraLineNumber("Duplicate function definition", functions[j].definedInLine, function.definedInLine);
            }
        }
    }

    if(checkForMainFunction == 1 && mainFunctionExists == 0) {
        printSemanticError("An executable cannot be created if no main-function exists", 1);
    }

    printDebugMessage("Checks done, freeing memory", "");
    //Now, we free all memory again
    for(int i = 0; i < functionDefinitions; i++) {
        //Free the pointer to the function name
        free(functions[i].name);
    }
    //Free the allocated memory for the function array
    free(functions);
}