/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021-2023 Tobias Kamm

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

#ifndef MEMEASSEMBLY_ANALYSISHELPER_H
#define MEMEASSEMBLY_ANALYSISHELPER_H

#include "analyser.h"

void checkDuplicateDefinition(struct commandLinkedList* commandLinkedList, struct compileState* compileState, bool oncePerFile, bool checkFirstParam, char* itemName);
void checkCompanionCommandExistence(struct commandLinkedList* parentCommands, struct commandLinkedList* childCommands, struct compileState* compileState, uint8_t parametersToCheck, bool sameFile, char* itemName);

#endif //MEMEASSEMBLY_ANALYSISHELPER_H
