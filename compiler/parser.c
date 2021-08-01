#include "parser.h"
#include <stdio.h>
#include <string.h>
#include "logger/log.h"

char *functionDeclaration = "I like to have fun, fun, fun, fun, fun, fun, fun ,fun, fun, fun";
char *functionReturn = "right back at ya, buckaroo";
char *commentStart = "What the hell happened here?";

char commandList[19][58] = {
        "stonks p",
        "not stonks p",
        "upgrade",
        "fuck go back",
        "guess I'll die",
        "bitconneeeeeeect p p",
        "sneak 100 p",
        "upvote p",
        "downvote p",
        "they're the same picture",
        "corporate needs you to find the difference between p and p",
        "p is brilliant, but I like p",
        "ah shit, here we go again",
        "parry p you filthy casual p",
        "no, I don't think I will",
        "perfectly balanced as all things should be",
        "what can I say except p",
        "upgrades, people. Upgrades p",
        "they had us in the first half, not gonna lie p",
};

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

struct command parseCommands(FILE *inputFile) {
    int loc = getLinesOfCode(inputFile);
    struct command *commands = calloc(sizeof(struct command), loc);
    if(commands == NULL) {
        fprintf(stderr, "Critical Error: Memory allocation for command parsing failed");
    }
}
