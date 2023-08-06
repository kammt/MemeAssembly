/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021-2023 Tobias Kamm and contributors

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

#include "log.h"
#include <stdarg.h>
#include <string.h>

const char* const versionString = "v1.6";
const char* const platformSuffix =
    #ifdef WINDOWS
        "Windows";
    #elif defined(MACOS)
        "MacOS";
    #else
        "Linux";
    #endif

const char* const randomErrorMessages[] = {
        //sudo
        "you are not in the sudoers file. This incident will be reported.",
        "you are not allowed to run sudo. This incident will be reported.",
        //erno - cause why not?
        "ENOSPC: No space left on device",
        "ENOENT: No such file or directory",
        //glibc
        "double free or corruption (top)",
        "corrupted top size",
        "double free or corruption (!prev)",
        "free: invalid pointer",
        //gcc
        "implicit declaration of function `gets'",
        "invalid type argument of `unary *'",
        "passing arg 2 of `strcpy' makes pointer from integer without a cast",
        "syntax error before '}' token",
        "ld returned exit 1 status",
        "no match for ‘operator==’ in ‘__first.__gnu_cxx::__normal_iterator::operator* [with _Iterator = std::vector*, _Container = std::vector >, __gnu_cxx::__normal_iterator::reference = std::vector&]() == __val’",
        "error: cannot bind rvalue ‘(short unsigned int)((const char*)\"\")’ to ‘short unsigned int&’",
        "no match for 'operator<<' (operand types are 'std::ostream' {aka 'std::basic_ostream<char>'} and 'std::array<int, 1>')",
        "invalid conversion from `int' to `std::_Rb_tree_node<std::pair<const int, double> >*'",
        //ld
        "relocation truncated to fit: R_X86_64_PC32 against symbol `main'",
        "(.ARM.exidx.text._ZNSt8_Rb_treeIiSt4pairIKiSt10shared_ptrIN4SWGL7ContextEEESt10_Select1stIS6_ESt4lessIiESaIS6_EE13_Rb_tree_implISA_Lb1EED2Ev[_ZNSt8_Rb_treeIiSt4pairIKiSt10shared_ptrIN4SWGL7ContextEEESt10_Select1stIS6_ESt4lessIiESaIS6_EE13_Rb_tree_implISA_Lb1EED5Ev]+0x0): relocation truncated to fit: R_ARM_PREL31 against `.text._ZNSt8_Rb_treeIiSt4pairIKiSt10shared_ptrIN4SWGL7ContextEEESt10_Select1stIS6_ESt4lessIiESaIS6_EE13_Rb_tree_implISA_Lb1EED2Ev'",
        "error adding symbols: DSO missing from command line",
        "undefined reference to `main'",
        //INTERCAL
        "programmer is insufficiently polite",
        "programmer is excessively polite",
        //actual MemeASM errors
        "a decimal number cannot be a pointer",
        "a function name cannot be a pointer",
        "invalid parameter combination: 64 Bit arithmetic operation commands require the decimal number to be sign-extendable from 32 Bits",
        "function does not return",
        "an executable cannot be created if no main function exists"
};

/**
 * Prints an ASCII-Art title and version information.
 */
void printInformationHeader() {
    printf(BLU"  __  __                                                    "RED" _     _       \n");
    printf(BLU" |  \\/  |                      "RED" /\\                          | |   | |      \n");
    printf(BLU" | \\  / | ___ _ __ ___   ___   "RED"/  \\   ___ ___  ___ _ __ ___ | |__ | |_   _ \n");
    printf(BLU" | |\\/| |/ _ \\ '_ ` _ \\ / _ \\ "RED"/ /\\ \\ / __/ __|/ _ \\ '_ ` _ \\| '_ \\| | | | |\n");
    printf(BLU" | |  | |  __/ | | | | |  __/"RED"/ ____ \\\\__ \\__ \\  __/ | | | | | |_) | | |_| |\n");
    printf(BLU" |_|  |_|\\___|_| |_| |_|\\___"RED"/_/    \\_\\___/___/\\___|_| |_| |_|_.__/|_|\\__, |\n");
    printf(RESET"  A Meme-based programming language.             " RED "                     __/ |\n");
    printf("                                                                     |___/ \n\n"RESET);
    printf("For more information, a list of commands and code examples, please visit https://github.com/kammt/MemeAssembly.\n");
    printf("This is the MemeAssembly compiler %s (%s), created by Tobias Kamm.\n\n", versionString, platformSuffix);
}

/**
 * Called if there is an error in the specified file. It prints a "Wait, that's illegal!" ASCII-Art and exits the program
 */
void printErrorASCII() {
    printf("\n");
    printf("\n");
    printf(YEL "  __          __   _ _       _   _           _   _       _ _ _                  _ _  \n");
    printf(" \\ \\        / /  (_| |     | | | |         | | ( )     (_| | |                | | | \n");
    printf("  \\ \\  /\\  / __ _ _| |_    | |_| |__   __ _| |_|/ ___   _| | | ___  __ _  __ _| | | \n");
    printf("   \\ \\/  \\/ / _` | | __|   | __| '_ \\ / _` | __| / __| | | | |/ _ \\/ _` |/ _` | | | \n");
    printf("    \\  /\\  | (_| | | |_ _  | |_| | | | (_| | |_  \\__ \\ | | | |  __| (_| | (_| | |_| \n");
    printf("     \\/  \\/ \\__,_|_|\\__( )  \\__|_| |_|\\__,_|\\__| |___/ |_|_|_|\\___|\\__, |\\__,_|_(_) \n");
    printf("                       |/                                           __/ |           \n");
    printf("                                                                   |___/  \n" RESET);
}

/**
 * Called when the command 'perfectly balanced as all things should be' is used. It prints a Snap ASCII art
 * @param deletedLines the number of lines that got deleted
 */
void printThanosASCII(size_t deletedLines) {
    printf("\n");
    printf("\n");
    printf(YEL "   _____                 \n");
    printf("  / ____|         \n");
    printf(" | (___  _ __   __ _ _ __ \n");
    printf("  \\___ \\| '_ \\ / _` | '_ \\ \n");
    printf("  ____) | | | | (_| | |_) | \n");
    printf(" |_____/|_| |_|\\__,_| .__/  \n");
    printf("                    | |  \n");
    printf("                    |_|    \n" RESET);
    printf(GRN "\nDid you do it?\n" RESET);
    printf(MAG "Yes\n" RESET);
    printf(GRN "What did it cost?\n" RESET);
    printf(MAG "%lu lines of code\n\n" RESET, deletedLines);
}

/**
 * Called when a decimal parameter with value 69 is encountered. It prints a "Nice" ASCII art
 */
void printNiceASCII() {
    printf("\n");
    printf("\n");
    printf("\x1B[38;5;197m" "  _   _ _          \n");
    printf("\x1B[38;5;197m" " | \\ | (_)         \n");
    printf("\x1B[38;5;198m" " |  \\| |_  ___ ___ \n");
    printf("\x1B[38;5;198m" " | . ` | |/ __/ _ \\\n");
    printf("\x1B[38;5;199m" " | |\\  | | (_|  __/\n");
    printf("\x1B[38;5;199m" " |_| \\_|_|\\___\\___|\n\n" RESET);
}

/**
 * Called when a decimal parameter with value 420 is encountered. It prints a "Bong" ASCII art
 */
void printBongASCII() {
    printf("\n");
    printf("\n");
    printf("\x1B[38;5;197m" "    .===. (  \n");
    printf("\x1B[38;5;197m" "    |   |  ) \n");
    printf("\x1B[38;5;198m" "    |   | (  \n");
    printf("\x1B[38;5;198m" "    |   | )  \n");
    printf("\x1B[38;5;199m" "    |   \\*/ \n");
    printf("\x1B[38;5;199m" "  ,'    //.  \n");
    printf("\x1B[38;5;199m" " :~~~~~//~~; \n");
    printf("\x1B[38;5;199m" "   `.  // .' \n");
    printf("\x1B[38;5;199m" " sc`-------' \n\n" RESET);
}

/**
 * Prints a debug message. It can be called with a variable number of arguments that will be inserted in the respective places in the format string
 */
void printDebugMessage(logLevel logLevel, char* message, unsigned varArgNum, ...) {
    if(logLevel == debug) {
        va_list vaList;
        va_start(vaList, varArgNum);

        vprintf(message, vaList);
        printf("\n");
    }
}

/**
 * Prints an error message. It can be called with a variable number of arguments that will be inserted in the respective places in the format string
 * @param inputFileName name of the input file
 * @param lineNum the line number in which the error occurred
 * @param compileState the current compileState. Needed to increase the error count
 * @param message the message (with printf-like formatting)
 * @param varArgNum How many variable arguments were passed (important!)
 * @param ... variable arguments
 */
void printError(char* inputFileName, unsigned lineNum, struct compileState* compileState, char* message, unsigned varArgNum, ...) {
    compileState->compilerErrors++;

    //First, only print the file name and line
    printf("%s:%u: " RED "error: " RESET, inputFileName, lineNum);

    if(compileState->compileMode != obfuscated) {
        //Initialise va_list to pass it on to vprintf
        va_list vaList;
        va_start(vaList, varArgNum);
        //Now print the custom message with variable args
        vprintf(message, vaList);
        printf("\n");
    } else {
        //Obfuscated mode: print a random error message instead
        uint64_t computedIndex = lineNum;
        for(size_t i = 0; i < strlen(inputFileName); i++) {
            computedIndex += inputFileName[i];
        }
        for(size_t i = 0; i < strlen(message); i++) {
            computedIndex += message[i];
        }
        computedIndex += varArgNum;

        printf("%s\n", randomErrorMessages[computedIndex % (sizeof(randomErrorMessages) / sizeof(char*))]);
    }
}

/**
 * Prints a note. It can be called with a variable number of arguments that will be inserted in the respective places in the format string
 * @param message the message (with printf-like formatting)
 * @param indent whether the entire message should be indented
 * @param varArgNum How many variable arguments were passed (important!)
 * @param ... variable arguments
 */
void printNote(char* message, bool indent, unsigned varArgNum, ...) {
    //Initialise va_list to pass it on to vprintf
    va_list vaList;
    va_start(vaList, varArgNum);

    //First, only print the file name and line
    if(indent) printf("\t");
    printf(MAG "note: " RESET);
    //Now print the custom message with variable args
    vprintf(message, vaList);
    printf("\n");
}

/**
 * Prints an internal compiler error, after which the compiler terminates
 * @param message the message (with printf-like formatting)
 * @param report whether to print a message telling the user to report this error
 * @param varArgNum How many variable arguments were passed (important!)
 * @param ... variable arguments
 */
void printInternalCompilerError(char* message, bool report, unsigned varArgNum, ...) {
    //Initialise va_list to pass it on to vprintf
    va_list vaList;
    va_start(vaList, varArgNum);

    //First, only print the file name and line
    fprintf(stderr, RED "Internal compiler error: " RESET);
    //Now print the custom message with variable args
    vfprintf(stderr, message, vaList);
    fprintf(stderr, "\n");
    if(report) fprintf(stderr, "Please report this error at https://github.com/kammt/MemeAssembly/issues/new");
}
