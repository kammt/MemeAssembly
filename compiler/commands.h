#ifndef MEMEASSEMBLY_COMMANDS_H
#define MEMEASSEMBLY_COMMANDS_H

#include <stdint.h>

#define NUMBER_OF_COMMANDS 28
#define COMMAND_LIST_MAX_STRING_LENGTH 66
#define MAX_PARAMETER_COUNT 2
#define MAX_PARAMETER_LENGTH 4

#define OR_DRAW_25_OPCODE NUMBER_OF_COMMANDS - 1;

struct command {
    uint8_t opcode;
    char params[MAX_PARAMETER_COUNT][MAX_PARAMETER_LENGTH];
    uint8_t isPointer; //0 = No Pointer, 1 = first parameter, 2 = second parameter
    int lineNum;
};

struct commandsArray {
    struct command* arrayPointer;
    int size;
};

#define commentStart "What the hell happened here?"

#define orDraw25Suffix "or draw 25"
#define orDraw25Start "or"
#define orDraw25End "draw 25"

#define pointerSuffix "do you know de wey"

#endif //MEMEASSEMBLY_COMMANDS_H
