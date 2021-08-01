#ifndef MEMEASSEMBLY_COMMANDS_H
#define MEMEASSEMBLY_COMMANDS_H

#define NUMBER_OF_COMMANDS 28
#define COMMAND_LIST_MAX_STRING_LENGTH 66
#define MAX_PARAMETER_COUNT 2
#define MAX_PARAMETER_LENGTH 3

#define OR_DRAW_25_OPCODE NUMBER_OF_COMMANDS - 1;

struct command {
    uint8_t opcode;
    char params[MAX_PARAMETER_COUNT][MAX_PARAMETER_LENGTH];
    uint8_t isPointer; //0 = No Pointer, 1 = first parameter, 2 = second parameter
};

const char *commentStart = "What the hell happened here?";

const char *orDraw25Suffix = "or draw 25";
const char *orDraw25Start = "or";
const char *orDraw25End = "draw 25";

const char *pointerSuffix = "do you know de wey";

char commandList[NUMBER_OF_COMMANDS][COMMAND_LIST_MAX_STRING_LENGTH] = {
        "I like to have fun, fun, fun, fun, fun, fun, fun ,fun, fun, fun p",
        "right back at ya, buckaroo",
        "stonks p",
        "not stonks p",
        "upgrade",
        "fuck go back",
        "guess I'll die",
        "bitconneeeeeeect p p",
        "sneak 100 p",
        "upvote p",
        "downvote p",
        "they're the same picture",
        "corporate needs you to find the difference between p and p",
        "p is brilliant, but I like p",
        "ah shit, here we go again",
        "parry p you filthy casual p",
        "no, I don't think I will",
        "perfectly balanced as all things should be",
        "what can I say except p",
        "upgrades, people. Upgrades p",
        "they had us in the first half, not gonna lie p",
        "monke p",
        "return to monke p",
        "who would win? p or p",
        "p wins",
        "banana",
        "where banana",
        "or draw 25" //Insert commands above this one
};

#endif //MEMEASSEMBLY_COMMANDS_H
