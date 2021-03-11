#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

#include "compiler.h" //Compiler related functions in a seperate file

#include "log.h"

char help1[] = "-h", help2[] = "--help";
char compile1[] = "-c", compile2[] = "--compile";
char log_info[] = "-v", log_debug[] = "-vv";

/**
 * Prints the help page of this command. Launched by using the -h option in the terminal
 */
void printHelpPage() {
    //https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text
    printInformationHeader();
    printf("Usage:\n");
    printf("  memeasm path/to/fileName [-v | -vv]\t\t\t\t\tCompiles and runs the specified file\n");
    printf("  memeasm (-c | --compile) [-v | -vv] path/to/fileName outputFile\tOnly compiles the specified file and saves it as x86-Assembly code\n");
    printf("  memeasm (-h | --help)\t\t\t\t\t\t\tDisplays this help page\n");
}

/**
 * Attempts to interpret the options for a 'compile' command
 * @param argc the number of arguments
 * @param argv an array of arguments specified
 * @returns 0 if command was interpreted successfully, 1 otherwise
 */
int interpretCompile(int argc, char *argv[]) {
    if(argc != 4 && argc != 5) {
        printf("Command interpretation failed. (-c / --compile) requires 2 additional parameters, but instead got %d\n", argc-2);
        return 1;
    } else {
        if(argc == 5) {
            if(strcmp(argv[2], log_info) == 0) {
                setLogLevel(2);
            } else if(strcmp(argv[2], log_debug) == 0) {
                setLogLevel(3);
            } else {
                printf("Command interpretation failed. (-c / --compile) requires 2 additional parameters, but instead got %d\n", argc-2);
                return 1;
            }
        }

        //There are enough arguments. Now we need to check if the first is a correct path
        FILE *srcPTR = fopen(argv[argc-2], "r");
        FILE *destPTR = fopen(argv[argc-1], "w");

        if(srcPTR == NULL) {
            printf("Command interpretation failed. '%s' is not a valid source path.\n", argv[2]);
            return 1;
        }
        if(destPTR == NULL) {
            printf("Command interpretation failed. '%s' is not a valid destination path.\n", argv[3]);
            return 1;
        }
        printInformationHeader();
        compile(srcPTR, destPTR);
        return 0;
    }
}

/**
 * Attempts to interpret the options for a 'compile and run' command
 * @param argc the number of arguments
 * @param argv an array of arguments specified
 * @returns 0 if command was interpreted successfully, 1 otherwise
 */
int interpretCompileAndRun(int argc, char *argv[]) {
    if(argc == 3) {
        if(strcmp(argv[1], log_info) == 0) {
            setLogLevel(2);
        } else if(strcmp(argv[1], log_debug) == 0) {
            setLogLevel(3);
        } else {
            printf("Command interpretation failed. One parameter required, but instead got %d\n", argc-2);
            return 1;
        }
    }

    FILE *srcPTR = fopen(argv[argc-1], "r");
    if(srcPTR == NULL) {
        printf("Command interpretation failed. '%s' is not a valid source path.\n", argv[1]);
    } else {
        printInformationHeader();
        compileAndRun(srcPTR);
        return 0;
    }
    return 1;    
}

/**
 * Parses the command-line arguments specified.
 * @param argc the number of arguments
 * @param argv an array of arguments specified
 * @returns 0 if command was interpreted successfully, 1 otherwise
 */
int interpretArguments(int argc, char* argv[]) {
    if (argc > 1) {
        if (strcmp(argv[1], help1) == 0 || strcmp(argv[1], help2) == 0){
            printHelpPage();
            return 0;
        } else if (strcmp(argv[1], compile1) == 0 || strcmp(argv[1], compile2) == 0) {
            //Compile-option used. Check if there are enough arguments
            return interpretCompile(argc, argv);
        } else if(argc == 2 || argc == 3) {
            return interpretCompileAndRun(argc, argv);
        }
    }
    return 1; //There are no arguments, it cannot be a correct command
}

int main(int argc, char* argv[]) {
    int result = interpretArguments(argc, argv);
    if(result == 1) {
        printf("Error! No arguments specified or unknown parameters. Type memeasm -h to open the help page. \n Exiting...\n");
    }
}
