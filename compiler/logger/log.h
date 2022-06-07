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

#ifndef LOG_H
#define LOG_H

#include <stdio.h>  //Printf() function
#include "../compiler.h"

#define RED   ""
#define GRN   ""
#define YEL   ""
#define BLU   ""
#define MAG   ""
#define CYN   ""
#define WHT   ""
#define RESET ""

void printInformationHeader();

void printErrorASCII();

void printThanosASCII(size_t deletedLines);

void printNiceASCII();

void printDebugMessage(logLevel logLevel, char* message, unsigned varArgNum, ...);
void printStatusMessage(logLevel logLevel, char* message);

void printError(char* inputFileName, unsigned lineNum, struct compileState* compileState, char* message, unsigned varArgNum, ...);
void printNote(char* message, unsigned varArgNum, ...);

#endif
