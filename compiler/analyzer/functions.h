#ifndef MEMEASSEMBLY_FUNCTIONS_H
#define MEMEASSEMBLY_FUNCTIONS_H

#include "../commands.h"
#include <stddef.h>

struct function {
    char* name;
    int definedInLine;
    int numberOfCommands;
};

void checkFunctionValidity(struct commandsArray *commandsArray, uint8_t checkForMainFunction);

#endif //MEMEASSEMBLY_FUNCTIONS_H
