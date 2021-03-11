#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

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