#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

#include "compiler.h" //Compiler related functions in a seperate file

#define BLU   "\x1B[34m"
#define RESET "\x1B[0m"

char version_string[] = "v0.1";
char release_date[] = "unknown";

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
    printf("This is the MemeAssembly compiler %s, created by Tobias Kamm.\n\n", version_string);
}

/**
 * Prints the help page of this command. Launched by using the -h option in the terminal
 */
void printHelpPage() {
    //https://stackoverflow.com/questions/9725675/is-there-a-standard-format-for-command-line-shell-help-text
    printInformationHeader();
    printf("Usage:\n");
    printf("  memeasm path/to/fileName\t\t\t\t\tCompiles and runs the specified file\n");
    printf("  memeasm (-c | --compile) path/to/fileName outputFile\t\tOnly compiles the specified file and saves it as x86-Assembly code\n");
    printf("  memeasm (-h | --help)\t\t\t\t\t\tDisplays this help page\n");
    printf("  memeasm (-v | --version)\t\t\t\t\tDisplays version information\n");
}

void printVersionInformation() {
    printInformationHeader();
    printf("\n------------Version Information------------\n");
    printf("Version: %s\n", version_string);
    printf("Release date: %s", release_date);
    printf("\n-------------------------------------------\n");
}

/**
 * Parses the command-line arguments specified.
 * @param argc the number of arguments
 * @param argv an array of arguments specified
 * @returns 0 if command was interpreted successfully, 1 otherwise
 */
int interpretArguments(int argc, char* argv[]) {
    char help1[] = "-h", help2[] = "--help";
    char compile1[] = "-c", compile2[] = "--compile";
    char version1[] = "-v", version2[] = "--version";

    if (argc > 1) {
        if (strcmp(argv[1], help1) == 0 || strcmp(argv[1], help2) == 0){
            printHelpPage();
            return 0;
        } else if (strcmp(argv[1], compile1) == 0 || strcmp(argv[1], compile2) == 0) {
            //Compile-option used. Check if there are enough arguments
            if(argc != 4) {
                printf("Command interpretation failed. (-c / --compile) requires 2 additional parameters, but instead got %d\n", argc-2);
                return 1;
            } else {
                //There are enough arguments. Now we need to check if the first is a correct path
                FILE *srcPTR = fopen(argv[2], "r");
                FILE *destPTR = fopen(argv[3], "w");

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
        } else if(argc == 2) {
            if (strcmp(argv[1], version1) == 0 || strcmp(argv[1], version2) == 0) {
                printVersionInformation();
                return 0;
            } else {
                FILE *srcPTR = fopen(argv[1], "r");
                if(srcPTR == NULL) {
                    printf("Command interpretation failed. '%s' is not a valid source path.\n", argv[1]);
                } else {
                    printInformationHeader();
                    compileAndRun(srcPTR);
                    return 0;
                }
                return 1;
            }
        }
    }
    return 1; //There are no arguments, it cannot be a correct command
}

int main(int argc, char* argv[]) {
    //printIllegal();
    int result = interpretArguments(argc, argv);
    if(result == 1) {
        printf("Error! No arguments specified or unknown parameters. Type memeasm -h to open the help page. \n Exiting...\n");
    }
    
}
