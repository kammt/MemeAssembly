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
#include <stdlib.h> //Exit() function


#define RED   "\x1B[31m"
#define GRN   "\x1B[32m"
#define YEL   "\x1B[33m"
#define BLU   "\x1B[34m"
#define MAG   "\x1B[35m"
#define CYN   "\x1B[36m"
#define WHT   "\x1B[37m"
#define RESET "\x1B[0m"

void setLogLevel(int newLogLevel);

void printInformationHeader();

void printErrorASCII();

void printThanosASCII(int deletedLines);

void printSuccessMessage(char message[]);
void printStatusMessage(char message[]);
void printInfoMessage(char message[]);
void printDebugMessage(char message[], char *variable);
void printDebugMessageWithNumber(char message[], int variable);

void printSemanticError(char message[], int lineNum);
void printSemanticErrorWithExtraLineNumber(char message[], int lineNum, int originalDefinition);

void printUnexpectedCharacterError(char expected[], char got[], int lineNum);
void printSyntaxError(char message[], char got[], int lineNum);
void printSyntaxErrorWithoutString(char message[], int lineNum);

#endif
