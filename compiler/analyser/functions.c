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

#include "functions.h"
#include "../logger/log.h"

#include <string.h>
/**
 * Checks if a main function was defined anywhere
 */
bool mainFunctionExists(struct compileState* compileState) {
    for(unsigned fileNum = 0; fileNum < compileState->fileCount; fileNum++) {
        for(unsigned funcNum = 0; funcNum < compileState->files[fileNum].functionCount; funcNum++) {
            const char* const mainFunctionName =
                #ifdef MACOS
                    "_main";
                #else
                    "main";
                #endif
            if (strcmp(compileState->files[fileNum].functions[funcNum].commands[0].parameters[0], mainFunctionName) == 0) {
                return true;
            }
        }
    }
    return false;
}

