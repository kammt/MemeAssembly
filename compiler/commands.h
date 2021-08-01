#ifndef MEMEASSEMBLY_COMMANDS_H
#define MEMEASSEMBLY_COMMANDS_H

struct command {
    uint8_t opcode;
    char param1[3];
    char param2[3];
    uint8_t isPointer; //0 = No Pointer, 1 = first parameter, 2 = second parameter
};

const char *functionDeclaration = "I like to have fun, fun, fun, fun, fun, fun, fun ,fun, fun, fun";
const char *functionReturn = "right back at ya, buckaroo";
const char *commentStart = "What the hell happened here?";
const char *orDraw25Suffix = "or draw 25";
const char *pointerSuffix = "do you know de wey";

const char commandList[25][58] = {
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
        "where banana"
};

#endif //MEMEASSEMBLY_COMMANDS_H
