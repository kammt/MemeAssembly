#ifndef MEMEASSEMBLY_FUNCTIONS_H
#define MEMEASSEMBLY_FUNCTIONS_H

#include "../commands.h"
#include <stddef.h>

struct function {
    char* name;
    size_t definedInLine;
    size_t numberOfCommands;
};

void checkFunctionValidity(struct commandsArray *commandsArray, int functionDeclarationOpcode);

#endif //MEMEASSEMBLY_FUNCTIONS_H
