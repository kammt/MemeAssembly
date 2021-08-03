#ifndef MEMEASSEMBLY_COMPARISONS_H
#define MEMEASSEMBLY_COMPARISONS_H

#include "../commands.h"

void checkWhoWouldWinValidity(struct commandsArray *commandsArray, int whoWouldWinOpcode);
void checkTheyreTheSamePictureValidity(struct commandsArray *commandsArray, int whoWouldWinOpcode);

#endif //MEMEASSEMBLY_COMPARISONS_H
