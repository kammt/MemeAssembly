#ifndef LOG_H
#define LOG_H

#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

/**
 * Called if there is an error in the specified file. It prints a "Wait, that's illegal!" ASCII-Art and exits the program
 */
void printErrorMessage();

void printSemanticError(char message[], int lineNum);
void printSemanticErrorWithExtraLineNumber(char message[], int lineNum, int originalDefinition);

#endif