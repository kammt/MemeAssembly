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

#ifndef MEMEASSEMBLY_COMMANDS_H
#define MEMEASSEMBLY_COMMANDS_H

#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <stdbool.h>

#define NUMBER_OF_COMMANDS 46
#define MAX_PARAMETER_COUNT 2

extern const struct command commandList[];

#define OR_DRAW_25_OPCODE NUMBER_OF_COMMANDS - 2;
#define INVALID_COMMAND_OPCODE NUMBER_OF_COMMANDS - 1;

struct commandLinkedList {
    struct parsedCommand* command;
    unsigned definedInFile;
    struct commandLinkedList* next;
};

struct parsedCommand {
    uint8_t opcode;
    char *parameters[MAX_PARAMETER_COUNT];
    uint8_t paramTypes[MAX_PARAMETER_COUNT];
    uint8_t isPointer; //0 = No Pointer, 1 = first parameter, 2 = second parameter, ...
    size_t lineNum;
    bool translate; //Default is 1 (true). Is set to false in case this command is selected for deletion by "perfectly balanced as all things should be"
};

struct function {
    char* definedInFile;
    size_t definedInLine;
    size_t numberOfCommands;
    struct parsedCommand* commands;
};

struct file {
    char* fileName;
    size_t loc; //lines of code
    size_t functionCount;
    struct function* functions;
    struct parsedCommand* parsedCommands;
    size_t randomIndex; //A variable necessary for the "confused stonks" command
};

typedef enum { noob, bully, obfuscated } compileMode;
typedef enum { executable, assemblyFile, objectFile } outputMode;
typedef enum { intSISD = 0, intSIMD = 1, floatSISD = 2, floatSIMD = 3, doubleSISD = 4, doubleSIMD = 5 } translateMode;
typedef enum { none, o_1 = -1, o_2 = -2, o_3 = -3, o_s, o69420 = 69420} optimisationLevel;
typedef enum { normal, info, debug } logLevel;

struct compileState {
    compileMode compileMode;
    outputMode outputMode;
    uint32_t fileCount;
    struct file* files;

    bool useStabs;
    bool allowIoCommands;
    bool martyrdom;
    translateMode translateMode;
    optimisationLevel optimisationLevel;

    unsigned compilerErrors;
    logLevel logLevel;
};

// Parameter types
#define PARAM_REG64 1
#define PARAM_REG32 2
#define PARAM_REG16 4
#define PARAM_REG8 8
#define PARAM_DECIMAL 16
#define PARAM_CHAR 32
#define PARAM_MONKE_LABEL 64
#define PARAM_FUNC_NAME 128
//Helper macros for register parameter macros
#define PARAM_ISREG(param) (param <= PARAM_REG8 && param > 0)
#define PARAM_REG (PARAM_REG64 | PARAM_REG32 | PARAM_REG16 | PARAM_REG8)

// Command types
#define COMMAND_TYPE_MOV 1
#define COMMAND_TYPE_FUNC_RETURN 2
#define COMMAND_TYPE_FUNC_DEF 3
#define COMMAND_TYPE_FUNC_CALL 4
#define CMD_ISFUNCDEF(opcode) (commandList[opcode].commandType == COMMAND_TYPE_FUNC_DEF)
#define CMD_ISFUNCCALL(opcode) (commandList[opcode].commandType == COMMAND_TYPE_FUNC_CALL)
#define CMD_ISRET(opcode) (commandList[opcode].commandType == COMMAND_TYPE_FUNC_RETURN)
#define CMD_ISMOV(opcode) (commandList[opcode].commandType == COMMAND_TYPE_MOV)

struct command {
    char *pattern;
    uint8_t usedParameters;
    /*
     * Allowed types work as follows: Each bit is assigned to a type of variable. If it is set to one, it is allowed.
     * That way, each parameter can allow multiple variable types.
     *  Bit 0: 64 bit registers
     *  Bit 1: 32 bit registers
     *  Bit 2: 16 bit registers
     *  Bit 3: 8 bit registers
     *  Bit 4: decimal numbers
     *  Bit 5: Characters (including Escape Sequences) / ASCII-code
     *  Bit 6: Valid Monke Jump Label
     *  Bit 7: Valid function name
     */
    uint8_t allowedParamTypes[MAX_PARAMETER_COUNT];
    /*
     * The command type contains a special value that can be used to identify the type of command
     * without specifying the index of the command. Some examples:
     *  - return statements
     *  - mov-command
     */
    uint8_t commandType;
    void (*analysisFunction)(struct commandLinkedList**, unsigned, struct compileState*); //commandLinkedList-list, opcode (index), compileState

    //TODO replace with char* translationPatterns[6];
    char* translationPattern;
};

#define commentStart "What the hell happened here?"
#define multiLineCommentStart "Why, why?"
#define multiLineCommendEnd "Oh, that's why"

#define orDraw25Start "or"
#define orDraw25End "draw 25"

#define pointerSuffix "do you know de wey"

#endif //MEMEASSEMBLY_COMMANDS_H
