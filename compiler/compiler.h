#ifndef COMPILER_H
#define COMPILER_H

#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

void createExecutable(FILE *srcPTR, char *destFile);
int compile(FILE *srcPTR, FILE *destPTR);

#endif