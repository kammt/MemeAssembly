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

#ifndef LOG_H
#define LOG_H

#include <stdio.h>  //Printf() function
#include "../compiler.h"

#define RED   "\x1B[31m"
#define GRN   "\x1B[32m"
#define YEL   "\x1B[33m"
#define BLU   "\x1B[34m"
#define MAG   "\x1B[35m"
#define CYN   "\x1B[36m"
#define WHT   "\x1B[37m"
#define RESET "\x1B[0m"

void printInformationHeader();

void printErrorASCII();

void printThanosASCII(size_t deletedLines);

void printNiceASCII();

void printDebugMessage(logLevel logLevel, char* message, unsigned varArgNum, ...);
void printStatusMessage(logLevel logLevel, char* message);

void printError(char* inputFileName, unsigned lineNum, struct compileState* compileState, char* message, unsigned varArgNum, ...);
void printWarning(char* inputFileName, unsigned lineNum, char* message, unsigned varArgNum, ...);

#endif
