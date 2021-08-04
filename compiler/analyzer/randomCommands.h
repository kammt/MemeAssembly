#ifndef MEMEASSEMBLY_RANDOMCOMMANDS_H
#define MEMEASSEMBLY_RANDOMCOMMANDS_H

#include "../commands.h"

void setConfusedStonksJumpLabel(struct commandsArray *commandsArray);
void chooseLinesToBeDeleted(struct commandsArray *commandsArray, int perfectlyBalancedOpcode);

#endif //MEMEASSEMBLY_RANDOMCOMMANDS_H
