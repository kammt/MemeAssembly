#ifndef TRANSLATOR_H
#define TRANSLATOR_H

#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

/**
 * Translates the MemeASM File into an x86-Assembly file
 * @param file an array containing all lines
 * @param lineCount the number of lines
 * @param opcodes an empty array in which the opcodes of each line will be written to
 * @param destPTR a pointer pointing to the Assembly-file
 */
void startTranslation(char file[][128], int lineCount, int opcodes[], FILE *destPTR);

void removeLineBreak(char *token);

#endif