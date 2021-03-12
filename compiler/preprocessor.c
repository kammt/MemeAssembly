#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

#include <time.h> //Random() function

#include "translator.h" //Setting the commands to be ignored or setting jump markers etc.
#include "log.h" //Printing to the console

/**
 * preprocessor.c
 * Some commands require to be detected before they are translated. A good example is the "perfectly balanced as all things should be"
 * When the removal of such commands should be truly random, then they have to be determined before translation. That's what this is for.
 * It 
 * - counts the lines of code
 * - looks for commands requiring preprocessing such as "confused stonks"
 * - Preprocesses them e.g. chooses the lines to be deleted or chooses the jump marker
 */
int perfectlyBalancedFound = 0;
int confusedStonksFound = 0;

int linesOfCode = 0;
char commands[2][60] = {
    "perfectly balanced as all things should be",
    "confused stonks"
};

/**
 * 
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

void preprocessPerfectlyBalanced() {
    printDebugMessage("preprocessPerfectlyBalanced() called", "");
    //For every time 'perfectly balanced ...' is called, we delete half the lines. First, we calculate the number of lines that remain
    //For this, we divide the lines of code by 2 the same number of times the command appeared
    float temp = linesOfCode;
    for(int k = 0; k < perfectlyBalancedFound; k++) {
        temp = temp / 2;
    }
    //Now we calculate the number of lines to be deleted
    int deleteNumberOfLines = linesOfCode - (int) temp;
    printf("Deleting: %d\n", deleteNumberOfLines);

    int linesToDelete[deleteNumberOfLines];
    int i = 0;
    while(i < deleteNumberOfLines) {
        int randomLine = (rand() % (linesOfCode)) + 1; //Choose a random line
        //Check if that line was already generated
        int alreadyExists = 0;
        for(int j = 0; j < i; j++) {
            if(linesToDelete[j] == randomLine) alreadyExists = 1;
        }
        if(alreadyExists) continue;

        linesToDelete[i] = randomLine;
        i++;
        printf("%d\n", randomLine);
    }

    printThanosASCII(deleteNumberOfLines);
}

void preprocessConfusedStonks() {
    int randomJumps[confusedStonksFound];

    printDebugMessage("preprocessConfusedStonks() called", "");
    for(int i = confusedStonksFound - 1; i >= 0; i--) {
        int randomLine = (rand() % (linesOfCode)) + 1;
        randomJumps[i] = randomLine;
        printDebugMessage("Choosing random line...", "");
    }
    
}

void analyseLine(char *line, int lineNum) {
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
                    confusedStonksFound++;
                    break;
                }
                break;
            }
        }
    }
}

void preprocess(FILE *srcPTR) {
    char line[128];
    int lineNum = 1;
    printInfoMessage("Source file opened for reading, searching line-by-line for commands in need of preprocessing");
    while(fgets(line, sizeof(line), srcPTR) != NULL) {
        printDebugMessage("Starting analysis of", line);
        analyseLine(line, lineNum);
        lineNum++;
        printDebugMessage("Done, moving on to next line", "\n");
    }
    printDebugMessage("First Analysis done. Starting preprocessor functions if necessary", "\n");
    srand(time(NULL));

    if(perfectlyBalancedFound >= 1) preprocessPerfectlyBalanced();
    if(confusedStonksFound >= 1) preprocessConfusedStonks();

    printDebugMessage("Preprocessing done, rewinding source pointer...", "\n");
    rewind(srcPTR);
}