/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021 Tobias Kamm and contributors

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
#include "log.h"

int compilationErrors = 0;
int logLevel = 1; //Default value
char* version_string = "v1.3";
char* platform_suffix =
    #ifdef WINDOWS
        "Windows";
    #elif defined(MACOS)
        "MacOS";
    #else
        "Linux";
    #endif

void setLogLevel(int newLogLevel) {
    logLevel = newLogLevel;
}

/**
 * Prints an ASCII-Art title and version information.
 */
void printInformationHeader() {
    printf(BLU"  __  __                                                     _     _       \n");
    printf(" |  \\/  |                       /\\                          | |   | |      \n");
    printf(" | \\  / | ___ _ __ ___   ___   /  \\   ___ ___  ___ _ __ ___ | |__ | |_   _ \n");
    printf(" | |\\/| |/ _ \\ '_ ` _ \\ / _ \\ / /\\ \\ / __/ __|/ _ \\ '_ ` _ \\| '_ \\| | | | |\n");
    printf(" | |  | |  __/ | | | | |  __// ____ \\\\__ \\__ \\  __/ | | | | | |_) | | |_| |\n");
    printf(" |_|  |_|\\___|_| |_| |_|\\___/_/    \\_\\___/___/\\___|_| |_| |_|_.__/|_|\\__, |\n");
    printf(RESET"  A Meme-based programming language.             " BLU "                     __/ |\n");
    printf("                                                                     |___/ \n\n"RESET);
    printf("For more information, a list of commands and code examples, please visit https://github.com/kammt/MemeAssembly.\n");
    printf("This is the MemeAssembly compiler %s (%s), created by Tobias Kamm.\n\n", version_string, platform_suffix);
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
void printThanosASCII(int deletedLines) {
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
    printf(MAG "%d lines of code\n\n" RESET, deletedLines);
}

/**
 * Called when a decimal parameter with value 420 or 69 is encountered. It prints a Nice ASCII art
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
 * A success message. Will always be printed
 * @param message the message
 */
void printSuccessMessage(char message[]) {
    printf(GRN "%s \n" RESET, message);
}

/**
 * A status message. Will always be printed
 * @param message the message
 */
void printStatusMessage(char message[]) {
    if(logLevel >= 2) {
        printf(YEL "%s \n" RESET, message);
        fflush( stdout );
    }
}

/**
 * An information message. Will only be printed if -d or -i is active
 * @param message the message
 */
void printInfoMessage(char message[]) {
    if(logLevel >= 2) {
        printf("%s \n", message);
        fflush( stdout );  
    }
}

/**
 * A debug message. Will only be printed if -d is active
 * @param message the message
 * @param variable an optional string variable
 */
void printDebugMessage(char message[], char *variable) {
    if(logLevel == 3) {
        printf("%s %s \n", message, variable);
        fflush( stdout );  
    }
}

/**
 * A debug message. Will only be printed if -d is active
 * @param message the message
 * @param variable an optional integer variable
 */
void printDebugMessageWithNumber(char message[], int variable) {
    if(logLevel == 3) {
        printf("%s %d \n", message, variable);
        fflush( stdout );  
    }
}

/**
 * Prints a simple semantic error message
 * @param message the error message
 * @param lineNum the line number
 */
void printSemanticError(char message[], int lineNum) {
    compilationErrors += 1;
    fprintf(stderr, RED "Semantic Error in line %d: %s\n" RESET, lineNum, message);
}

/**
 * Prints a semantic error message concerning multiple definitions
 * @param message the error message
 * @param lineNum the line number
 * @param originalDefinition the line Number in which the original definition was
 */
void printSemanticErrorWithExtraLineNumber(char message[], int lineNum, int originalDefinition) {
    compilationErrors += 1;
    fprintf(stderr, RED "Semantic Error in line %d: %s (already defined in line %d)\n" RESET, lineNum, message, originalDefinition);
}

/**
 * Prints an error message concerning a wrong token
 * @param expected what the compiler wanted
 * @param got what the token actually was
 * @param lineNum the line number
 */
void printUnexpectedCharacterError(char expected[], char got[], int lineNum) {
    compilationErrors += 1;
    fprintf(stderr, RED "Syntax Error in line %d: Expected %s, but got %s\n" RESET, lineNum, expected, got);
}

/**
 * Prints a generic syntax error
 * @param message the error message
 * @param got the token that the compiler received. It will be inserted at the end of the message, so formatting must match.
 * @param lineNum the line number
 */
void printSyntaxError(char message[], char got[], int lineNum) {
    compilationErrors += 1;
    fprintf(stderr, RED "Syntax Error in line %d: %s '%s'\n" RESET, lineNum, message, got);
}

/**
 * Prints a generic syntax error
 * @param message the error message
 * @param lineNum the line number
 */
void printSyntaxErrorWithoutString(char message[], int lineNum) {
    compilationErrors += 1;
    fprintf(stderr, RED "Syntax Error in line %d: %s\n" RESET, lineNum, message);
}
