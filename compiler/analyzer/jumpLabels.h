#ifndef MEMEASSEMBLY_JUMPLABELS_H
#define MEMEASSEMBLY_JUMPLABELS_H

#include "../commands.h"

void checkMonkeJumpLabelValidity(struct commandsArray *commandsArray, int monkeOpcode);
void checkJumpLabelValidity(struct commandsArray *commandsArray, int upgradeOpcode);

#endif //MEMEASSEMBLY_JUMPLABELS_H
