#ifndef MEMEASSEMBLY_FUNCTIONS_H
#define MEMEASSEMBLY_FUNCTIONS_H

#include "../commands.h"
#include <stddef.h>

struct function {
    char* name;
    struct commandsArray commands;
};

struct functionsArray{
    struct function* functionsPointer;
    size_t size;
};

#endif //MEMEASSEMBLY_FUNCTIONS_H
