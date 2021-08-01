#include "parser.h"
#include <stdio.h>
#include <string.h>

#include "logger/log.h"
#include "commands.h"

unsigned int min(unsigned int one, unsigned int two) {
    if(one > two) {
        return two;
    } else {
        return one;
    }
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
    char *line = NULL;
    size_t len = 0;
    ssize_t lineLength;

    while((lineLength = getline(&line, &len, inputFile)) != -1) {
        //If the line is empty, skip it entirely
        if(lineLength == 1) {
            continue;
        }

        //If it starts with the comment string, ignore it
        if(strncmp(line, commentStart, min(strlen(line), strlen(commentStart))) != 0) {
            loc++;
        }
    }

    printDebugMessageWithNumber("The number of lines are", loc);
    printDebugMessage("Rewinding source pointer", "");
    rewind(inputFile);
    return loc;
}

struct command *parseCommands(FILE *inputFile) {
    //First, we create an array of command structs
    int loc = getLinesOfCode(inputFile);
    struct command *commands = calloc(sizeof(struct command), loc);
    if(commands == NULL) {
        fprintf(stderr, "Critical Error: Memory allocation for command parsing failed");
    }
    printDebugMessage("Struct array was created successfully", "");

    return commands;
}
