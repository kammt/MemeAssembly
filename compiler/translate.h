#ifndef TRANSLATE_H
#define TRANSLATE_H

#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

/**
 * Translates the MemeASM File into an x86-Assembly file
 * @param srcPTR a pointer to the source file to be translated
 * @param destPTR a pointer to the destination file. If nonexistent, it will be created
 */
void startTranslation(FILE *srcPTR, FILE *destPTR);

#endif