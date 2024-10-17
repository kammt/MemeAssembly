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

#ifndef MEMEASSEMBLY_PARAMETERS_H
#define MEMEASSEMBLY_PARAMETERS_H

#include "../commands.h"

#define NUMBER_OF_8_BIT_REGISTERS 20
#define NUMBER_OF_64_BIT_REGISTERS 16
#define NUMBER_OF_32_BIT_REGISTERS 16
#define NUMBER_OF_16_BIT_REGISTERS 16
#define NUMBER_OF_ESCAPE_SEQUENCES 10

extern const char *registers_64_bit[NUMBER_OF_64_BIT_REGISTERS];
extern const char *registers_32_bit[NUMBER_OF_32_BIT_REGISTERS];
extern const char *registers_16_bit[NUMBER_OF_16_BIT_REGISTERS];
extern const char *registers_8_bit[NUMBER_OF_8_BIT_REGISTERS];

void checkParameters(struct parsedCommand *parsedCommand, const char* inputFileName, struct compileState* compileState);

#endif //MEMEASSEMBLY_PARAMETERS_H
