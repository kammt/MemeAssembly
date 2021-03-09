#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

/**
 * Called if there is an error in the specified file. It prints a "Wait, that's illegal!" ASCII-Art, lists all errors and exits the program
 * @param errors a String listing all compiler errors
 */
void printErrors(char errors[]) {
    printf("\n");
    printf("\n");
    printf("  __          __   _ _       _   _           _   _       _ _ _                  _ _  \n");
    printf(" \\ \\        / /  (_| |     | | | |         | | ( )     (_| | |                | | | \n");
    printf("  \\ \\  /\\  / __ _ _| |_    | |_| |__   __ _| |_|/ ___   _| | | ___  __ _  __ _| | | \n");
    printf("   \\ \\/  \\/ / _` | | __|   | __| '_ \\ / _` | __| / __| | | | |/ _ \\/ _` |/ _` | | | \n");
    printf("    \\  /\\  | (_| | | |_ _  | |_| | | | (_| | |_  \\__ \\ | | | |  __| (_| | (_| | |_| \n");
    printf("     \\/  \\/ \\__,_|_|\\__( )  \\__|_| |_|\\__,_|\\__| |___/ |_|_|_|\\___|\\__, |\\__,_|_(_) \n");
    printf("                       |/                                           __/ |           \n");
    printf("                                                                   |___/  \n");
    printf("____________________________________________________________________________________ \n");
    printf("\nYour program failed to compile because of errors in your code. Please check your input file and try again. Errors found:\n%s\n", errors);
    printf(" Exiting....\n");
    exit(1);
}

void compile(char srcFile[], char destFile[]) {
    printf("Compile called");
}

void compileAndRun(char srcFile[]) {
    printf("Compile and run called");
    compile(srcFile, "tmp");
}