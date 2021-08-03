#ifndef MEMEASSEMBLY_COMMANDS_H
#define MEMEASSEMBLY_COMMANDS_H

#include <stdint.h>
#include <stddef.h>

#define NUMBER_OF_COMMANDS 31
#define COMMAND_PATTERN_LIST_MAX_STRING_LENGTH 66
#define COMMAND_TRANSLATION_LIST_MAX_STRING_LENGTH 33
#define MAX_PARAMETER_COUNT 2
#define MAX_PARAMETER_LENGTH 4

#define OR_DRAW_25_OPCODE NUMBER_OF_COMMANDS - 2;
#define INVALID_COMMAND_OPCODE NUMBER_OF_COMMANDS - 1;

struct parsedCommand {
    uint8_t opcode;
    char *parameters[MAX_PARAMETER_COUNT];
    uint8_t isPointer; //0 = No Pointer, 1 = first parameter, 2 = second parameter, ...
    int lineNum;
};

struct commandsArray {
    struct parsedCommand* arrayPointer;
    size_t size;
};

struct command {
    char pattern[COMMAND_PATTERN_LIST_MAX_STRING_LENGTH];
    uint8_t usedParameters;
    /*
     * Allowed types work as follows: Each bit is assigned to a type of variable. If it is set to one, it is allowed.
     * That way, each parameter can allow multiple variable types.
     *  Bit 0: 32 bit and 64 bit registers
     *  Bit 1: 16 bit registers
     *  Bit 2: 8 bit registers
     *  Bit 3: decimal numbers
     *  Bit 4: Characters (including Escape Sequences)
     *  Bit 5: ASCII-Code
     *  Bit 6: Valid Monke Jump Label
     *  Bit 7: Valid function name
     */
    uint8_t allowedParamTypes[MAX_PARAMETER_COUNT];
    void (*analysisFunction)(struct commandsArray*, int);
    char translationPattern[COMMAND_TRANSLATION_LIST_MAX_STRING_LENGTH];
};

#define commentStart "What the hell happened here?"

#define orDraw25Suffix "or draw 25"
#define orDraw25Start "or"
#define orDraw25End "draw 25"

#define pointerSuffix "do you know de wey"

#endif //MEMEASSEMBLY_COMMANDS_H
