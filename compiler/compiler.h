#ifndef COMPILER_H
#define COMPILER_H

#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

/**
 * Called if there is an error in the specified file. It prints a "Wait, that's illegal!" ASCII-Art, lists all errors and exits the program
 * @param errors a String listing all compiler errors
 */
void printErrors(char errors[]);

void compileAndRun(char srcFile[]);
void compile(char srcFile[], char destFile[]);

#endif