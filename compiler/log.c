#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

#include "translate.h" //Translation to Assembly
#include "analyse.h" //Semantic analysis

#include <sys/types.h>
#include <unistd.h>

#define RED   "\x1B[31m"
#define GRN   "\x1B[32m"
#define YEL   "\x1B[33m"
#define BLU   "\x1B[34m"
#define MAG   "\x1B[35m"
#define CYN   "\x1B[36m"
#define WHT   "\x1B[37m"
#define RESET "\x1B[0m"

/**
 * Called if there is an error in the specified file. It prints a "Wait, that's illegal!" ASCII-Art and exits the program
 */
void printErrorMessage() {
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
    printf("\nYour program failed to compile because of errors in your code. Please check your input file and try again.\n");
    printf(" Exiting....\n");
    exit(1);
}

/**
 * Prints a simple semantic error message
 * @param message the error message
 * @param lineNum the line number
 */
void printSemanticError(char message[], int lineNum) {
    printf(RED "Semantic Error in line %d: %s", lineNum, message);
    printErrorMessage();
}

/**
 * Prints a semantic error message concerning multiple definitions
 * @param message the error message
 * @param lineNum the line number
 * @param originalDefinition the line Number in which the original definition was
 */
void printSemanticErrorWithExtraLineNumber(char message[], int lineNum, int originalDefinition) {
    printf(RED "Semantic Error in line %d: %s (already defined in line %d)", lineNum, message, originalDefinition);
    printErrorMessage();
}

/**
 * Prints an error message concerning a wrong token
 * @param expected what the compiler wanted
 * @param got what the token actually was
 * @param lineNum the line number
 */
void printUnexpectedCharacterError(char expected[], char got[], int lineNum) {
    printf(RED "Syntax Error in line %d: Expected %s, but got %s", lineNum, expected, got);
}

/**
 * Prints a generic syntax error
 * @param message the error message
 * @param got the token that the compiler received. It will be inserted at the end of the message, so formatting must match.
 * @param lineNum the line number
 */
void printSyntaxError(char message[], char got[], int lineNum) {
    printf(RED "Syntax Error in line %d: %s '%s'", lineNum, message, got);
}    