#include "parser.h"
#include <stdio.h>
#include <string.h>

#include "logger/log.h"
#include "commands.h"

char *line = NULL;
size_t len = 0;
ssize_t lineLength;

unsigned int min(unsigned int one, unsigned int two) {
    if(one > two) {
        return two;
    } else {
        return one;
    }
}

/**
 * Removes the \n from a string if it is present at the end of the string
 * @param token the token
 */
void removeLineBreak(char *token) {
    if(token[strlen(token)-1] == '\n') {
        token[strlen(token)-1] = '\0';
    }
}

/**
 * Checks whether this line should be skipped or not
 * @param lineString the parsed line
 * @param lineStringLength the length of this line
 * @return 0 if it is of interest (=> code), 1 if it should be skipped (e.g. it is a comment or it's empty)
 */
int isLineOfInterest(char *lineString, ssize_t lineStringLength) {
    if(lineLength != 1 && strncmp(line, commentStart, min(strlen(line), strlen(commentStart))) != 0) {
        return 0;
    }
    return 1;
}

/**
 * Counts the lines of code in a memeasm file. A line counts as a line of code if:
 *  - it does not start with "What the hell happened here?" (a comment)
 *  - it is not empty
 * @param inputFile the input file
 * @return the number of lines of code in the file
 */
int getLinesOfCode(FILE *inputFile) {
    int loc = 0;

    while((lineLength = getline(&line, &len, inputFile)) != -1) {
        if(isLineOfInterest(line, lineLength) == 0) {
            loc++;
        }
    }

    printDebugMessageWithNumber("The number of lines are", loc);
    printDebugMessage("Rewinding source pointer", "");
    rewind(inputFile);
    return loc;
}

struct command parseLine(char *line, ssize_t lineLength) {
    struct command parsedCommand;
    char linecpy[strlen(line) + 1];
    strncpy(linecpy, line, strlen(line) + 1);



    return parsedCommand;
}

struct command *parseCommands(FILE *inputFile) {
    //First, we create an array of command structs
    int loc = getLinesOfCode(inputFile);
    struct command *commands = calloc(sizeof(struct command), loc);
    if(commands == NULL) {
        fprintf(stderr, "Critical Error: Memory allocation for command parsing failed");
        exit(EXIT_FAILURE);
    }
    printDebugMessage("Struct array was created successfully", "");

    //Iterate through the file again, this time parsing each line of interest and adding it to our command struct array
    int i = 0;

    while((lineLength = getline(&line, &len, inputFile)) != -1) {
        if(isLineOfInterest(line, lineLength) == 0) {
            removeLineBreak(line);
            printDebugMessage("Parsing line:", line);
            *(commands + i) = parseLine(line, lineLength);
        }
    }

    return commands;
}
