//
// Created by tobias on 01.08.21.
//

#include <bits/types/FILE.h>
#include <stdint.h>

#ifndef MEMEASSEMBLY_PARSER_H
#define MEMEASSEMBLY_PARSER_H

struct command {
    uint8_t opcode;
    char param1[3];
    char param2[3];
    uint8_t isPointer; //0 = No Pointer, 1 = first parameter, 2 = second parameter
};

struct command parseCommands(FILE *inputFile);

#endif //MEMEASSEMBLY_PARSER_H
