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
 * @return 0 if it is of interest (=> code), 1 if it should be skipped (e.g. it is a comment or it's empty)
 */
int isLineOfInterest() {
    //TODO tabbed comments?
    if(lineLength != 1 && strncmp(line, commentStart, min(lineLength, strlen(commentStart))) != 0) {
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
        if(isLineOfInterest() == 0) {
            loc++;
        }
    }

    printDebugMessageWithNumber("The number of lines are", loc);
    printDebugMessage("Rewinding source pointer", "");
    rewind(inputFile);
    return loc;
}

struct command parseLine(int lineNum) {
    //Temporarily save the line on the stack to be able to restore when a comparison failed
    struct command parsedCommand;
    char lineCpy[strlen(line) + 1];
    strncpy(lineCpy, line, strlen(line) + 1);

    char *savePtrLine;
    char *savePtrPattern;

    //Iterate through all possible commands
    for(int i = 0; i < NUMBER_OF_COMMANDS; i++) {
        strncpy(lineCpy, line, strlen(line) + 1);
        savePtrLine = NULL;
        savePtrPattern = NULL;

        //Copy the current command pattern out of read-only memory
        char commandString[COMMAND_LIST_MAX_STRING_LENGTH + 1];
        strncpy(commandString, commandList[i], COMMAND_LIST_MAX_STRING_LENGTH);

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
                printDebugMessage("\tInterpreting as parameter:", lineToken);
                strncpy(parsedCommand.params[numberOfParameters++], lineToken, min((unsigned int) strlen(lineToken), sizeof(parsedCommand.params[0])));

                //If the line after this parameter contains "do you know de wey", mark it as a pointer
                if(strlen(savePtrLine) >= strlen(pointerSuffix) && strncmp(pointerSuffix, savePtrLine, strlen(pointerSuffix)) == 0) {
                    printDebugMessage("\t\t'do you know de wey' was found, interpreting as pointer", "");
                    //If another parameter is already marked as a variable, throw an error
                    if(parsedCommand.isPointer != 0) {
                        printSemanticError("Only one parameter is allowed to be a pointer", lineNum);
                        //Return something to be added to the array, compilation won't continue anyway. If we wouldn't stop here, an "Failed to parse" error would be printed again
                        return parsedCommand;
                    } else {
                        parsedCommand.isPointer = (uint8_t) numberOfParameters;
                        //Move the save pointer so that "do you know de wey" is not tokenized by strtok_r
                        savePtrLine += strlen(pointerSuffix);
                    }
                }
            } else if(strcmp(commandToken, lineToken) != 0) {
                //If both tokens do not match, try the next command
                printDebugMessage("\tMatching failed, attempting to match next command", "");
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
            printDebugMessage("\tMatching failed, lineToken is NULL while commandToken is not. Attempting to match next command", "");
            break;
        //If the current token is 'or' and the rest of the string is only 'draw 25', then set the opcode as "or draw 25" and return
        } else if(strcmp(lineToken, orDraw25Start) == 0 && strlen(savePtrLine) == strlen(orDraw25End) && strncmp(orDraw25End, savePtrLine, strlen(orDraw25End)) == 0) {
            printDebugMessage("\t'or draw 25' was found, replacing opcode", "");
            parsedCommand.opcode = OR_DRAW_25_OPCODE;
            return parsedCommand;
        }
    }

    printSyntaxError("Failed to parse command:", line, lineNum);
    //Any error will set the "compilationErrors" variable in log.c to 1, meaning that we can safely return something that doesn't make sense
    //We don't exit immediately because we want to print every error possible
    return parsedCommand;
}

struct command *parseCommands(FILE *inputFile) {
    //First, we create an array of command structs
    int loc = getLinesOfCode(inputFile);
    struct command *commands = calloc(sizeof(struct command), (size_t) loc);
    if(commands == NULL) {
        fprintf(stderr, "Critical Error: Memory allocation for command parsing failed");
        exit(EXIT_FAILURE);
    }
    printDebugMessage("Struct array was created successfully", "");

    //Iterate through the file again, this time parsing each line of interest and adding it to our command struct array
    int i = 0;
    int lineNumber = 1;

    while((lineLength = getline(&line, &len, inputFile)) != -1) {
        if(isLineOfInterest() == 0) {
            removeLineBreak(line);
            printDebugMessage("Parsing line:", line);
            *(commands + i) = parseLine(lineNumber);
            i++;
        }
        lineNumber++;
    }

    if(getNumberOfCompilationErrors() > 0) {
        printErrorASCII();
        fprintf(stderr, "File Parsing failed with %d errors, please check your code and try again.\n", getNumberOfCompilationErrors());
        exit(EXIT_FAILURE);
    }
    return commands;
}
