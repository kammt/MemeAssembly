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

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include "../commands.h"

#ifndef MEMEASSEMBLY_PARSER_H
#define MEMEASSEMBLY_PARSER_H

/**
 * Parses an input file line by line and converts it to a command struct array which is saved in compileState->commands
 */
void parseCommands(FILE *inputFile, char* inputFileName, struct compileState* compileState);

#endif //MEMEASSEMBLY_PARSER_H
