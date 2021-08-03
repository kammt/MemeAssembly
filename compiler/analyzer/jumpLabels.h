#ifndef MEMEASSEMBLY_JUMPLABELS_H
#define MEMEASSEMBLY_JUMPLABELS_H

#include "../commands.h"

void checkMonkeJumpLabelValidity(struct commandsArray *commandsArray, int monkeOpcode);
void checkUpgradeValidity(struct commandsArray *commandsArray, int upgradeOpcode);
void checkBananaValidity(struct commandsArray *commandsArray, int bananaOpcode);

#endif //MEMEASSEMBLY_JUMPLABELS_H
