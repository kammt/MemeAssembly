/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021 Tobias Kamm

MemeAssembly is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

MemeAssembly is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with MemeAssembly. If not, see <https://www.gnu.org/licenses/>.
*/

#ifndef MEMEASSEMBLY_JUMPLABELS_H
#define MEMEASSEMBLY_JUMPLABELS_H

#include "../commands.h"

void checkMonkeJumpLabelValidity(struct commandsArray *commandsArray, int monkeOpcode);
void checkJumpLabelValidity(struct commandsArray *commandsArray, int upgradeOpcode);

#endif //MEMEASSEMBLY_JUMPLABELS_H
