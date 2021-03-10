#ifndef COMPILER_H
#define COMPILER_H

#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

void compileAndRun(FILE *srcPTR);
void compile(FILE *srcPTR, FILE *destPTR);

#endif