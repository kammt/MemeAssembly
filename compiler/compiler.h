#ifndef COMPILER_H
#define COMPILER_H

#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

int compileMode = 0;

void createExecutable(FILE *srcPTR);
void compile(FILE *srcPTR, FILE *destPTR);

#endif