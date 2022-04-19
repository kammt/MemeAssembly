/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021-2022 Tobias Kamm

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

#ifndef MEMEASSEMBLY_TRANSLATOR_H
#define MEMEASSEMBLY_TRANSLATOR_H

#include "../commands.h"

#include <stdio.h>

void writeToFile(struct compileState* compileState, FILE *outputFile);

#endif //MEMEASSEMBLY_TRANSLATOR_H
