#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

#include <time.h> //Random() function

#include "translator.h" //Setting the commands to be ignored or setting jump markers etc.
#include "log.h" //Printing to the console

/**
 * preprocessor.c
 * Some commands require to be detected before they are translated. A good example is the "perfectly balanced as all things should be" command.
 * When the removal of e.g. half the commands should be truly random, then the lines to-be-removed have to be determined before translation. 
 * That's what this is for.
 * It 
 * - looks for commands requiring preprocessing such as "confused stonks"
 * - Preprocesses them e.g. chooses the lines to be deleted or chooses the jump marker
 */
int perfectlyBalancedFound = 0;

int linesOfCode = 0;
int numberOfLines = 0;
char commands[2][60] = {
    "perfectly balanced as all things should be",
    "confused stonks"
};

/**
 * A more minimal version of the compileWithPattern() function in translator.c
 * It checks if the line matches with the command passed to it
 * @param token The first token of the line
 * @param index The array index of the command to be compared against
 * @param lineNum the line number
 * 
 * @returns 0 if successful, 1 if there is a syntax error and -1 if the first tokens don't match
 */
int isCommand(char *token, int index, int lineNum) {
    int probing = 1; //Is set to 0 if the first tokens match. This is so that if compilation fails at the first token, there is no error message

    //A save pointer for strtok_r
    char *savePTRcommand;

    //Copy the command out of read-only memory and create the first token
    char commandsTMP[60];
    strcpy(commandsTMP, commands[index]);
    char *commandToken = strtok_r(commandsTMP, " ", &savePTRcommand);
    
    while(token != NULL && commandToken != NULL) {
        if(strcmp(token, commandToken) != 0) {
            if(probing == 1) return -1;
            printUnexpectedCharacterError(commandToken, token, lineNum);
        }
        probing = 0;

        commandToken = strtok_r(NULL, " ", &savePTRcommand);
        token = strtok(NULL, " ");
    }
    if(token == NULL && commandToken == NULL) {
        return 0;
    } else return 1;
}

/**
 * Called when 'perfectly balanced ...' is used at least once. For every occurrence of the command, it halves the number of commands by overwriting
 * their respective array entries with a line break. After that, it prints a Thanos ASCII
 * @param file the array containing all lines
 */
void preprocessPerfectlyBalanced(char file[][128]) {
    printDebugMessage("preprocessPerfectlyBalanced() called", "");
    //For every time 'perfectly balanced ...' is called, we delete half the lines. First, we calculate the number of lines that remain
    //For this, we divide the lines of code by 2 the same number of times the command appeared
    float temp = linesOfCode;
    for(int k = 0; k < perfectlyBalancedFound; k++) {
        temp = temp / 2;
    }
    //Now we calculate the number of lines to be deleted
    int deleteNumberOfLines = linesOfCode - (int) temp;
    
    int i = 0;
    while(i < deleteNumberOfLines) {
        int randomLine = (rand() % (numberOfLines-1)) + 1; //Choose a random line
        printDebugMessageWithNumber("Chose random line:", randomLine);
        //Check if that line was already cleared
        if(strcmp(file[randomLine], "\n") == 0) continue;

        strcpy(file[randomLine], "\n\0"); //Clear that line
        i++;
        printDebugMessageWithNumber("Line cleared:", randomLine);
    }

    printDebugMessage("Printing Thanos ASCII...", "");
    printThanosASCII(deleteNumberOfLines);
}


/**
 * Called when 'confused stonks' is used in a line. It chooses a random line and replaces this line with a jump to the random line
 * (translator.c adds line_n: to the beginning of every line, n being the line number)
 * @param lineFound the line number in which confused stonks was found
 * @param file the file array 
 */
void preprocessConfusedStonks(int lineFound, char file[][128]) {
    printDebugMessage("preprocessConfusedStonks() called", "");
    int randomLine = (rand() % (linesOfCode)) + 1;
    printDebugMessageWithNumber("Injecting jump to line", randomLine);

    char jumpString[15] = "jmp line_";
    char lineAsString[4];
    sprintf(lineAsString, "%d", randomLine);

    strcat(jumpString, lineAsString);
    printDebugMessage(jumpString, "");

    strcpy(file[lineFound - 1], jumpString);
}

void analyseLine(char *line, int lineNum, char file[][128]) {
    removeLineBreak(line);
    char *token = strtok(line, " ");
    if(token != NULL) {
        linesOfCode++;
        for(int i = 0; i < sizeof(commands) / sizeof(commands[0]); i++) {
            if(isCommand(token, i, lineNum) == 0) {
                switch (i){
                case 0:
                    perfectlyBalancedFound++;
                    break;
                case 1:
                    preprocessConfusedStonks(lineNum, file);
                    break;
                }
                break;
            }
        }
    }
}

void *preprocess(char file[][128], FILE *srcPTR) {
    char line[128];
    int lineNum = 1;
    printInfoMessage("Source file opened for reading, searching line-by-line for commands in need of preprocessing");
    while(fgets(line, sizeof(line), srcPTR) != NULL) {
        printDebugMessage("Starting analysis of", line);
        strcpy(file[lineNum-1], line); //Copy the line into our internal array
        analyseLine(line, lineNum, file);
        lineNum++;
        printDebugMessage("Done, moving on to next line", "\n");
    }
    numberOfLines = lineNum;

    printDebugMessage("First Analysis done. Starting preprocessor functions if necessary", "\n");
    srand(time(NULL));

    if(perfectlyBalancedFound >= 1) preprocessPerfectlyBalanced(file);

    fclose(srcPTR);
    printDebugMessage("Source pointer closed, preprocessing is done.", "\n");
}