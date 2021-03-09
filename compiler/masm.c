#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

char version_string[] = "v0.1";

/**
 * Called if there is an error in the specified file. It prints a "Wait, that's illegal!" ASCII-Art and exits the program
 */
void printIllegal() {
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
    printf("\nYour program failed to compile because of errors in your code. Please check your input file and try again.\n");
    printf("Exiting....\n");
    exit(1);
}

/**
 * Prints an ASCII-Art title and version information.
 */
void printInformationHeader() {
    printf("  __  __                                                     _     _       \n");
    printf(" |  \\/  |                       /\\                          | |   | |      \n");
    printf(" | \\  / | ___ _ __ ___   ___   /  \\   ___ ___  ___ _ __ ___ | |__ | |_   _ \n");
    printf(" | |\\/| |/ _ \\ '_ ` _ \\ / _ \\ / /\\ \\ / __/ __|/ _ \\ '_ ` _ \\| '_ \\| | | | |\n");
    printf(" | |  | |  __/ | | | | |  __// ____ \\\\__ \\__ \\  __/ | | | | | |_) | | |_| |\n");
    printf(" |_|  |_|\\___|_| |_| |_|\\___/_/    \\_\\___/___/\\___|_| |_| |_|_.__/|_|\\__, |\n");
    printf("  A Meme-based programming language.                                  __/ |\n");
    printf("                                                                     |___/ \n\n");
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
    printf("  masm path/to/fileName\t\t\t\t\tCompiles and runs the specified file\n");
    printf("  masm (-c | --compile) path/to/fileName outputFile\tOnly compiles the specified file and saves it as x86-Assembly code\n");
    printf("  masm (-h | --help)\t\t\t\t\tDisplays this help page\n");
    printf("  masm (-v | --version)\t\t\t\t\tDisplays version information\n");
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

    if (argc > 1) {
        if (strcmp(argv[1], help1) == 0 || strcmp(argv[1], help2) == 0)
        {
            printHelpPage();
            return 0;
        }
    }
    return 1; //There no argument, it cannot be a correct command
}

int main(int argc, char* argv[]) {
    //printIllegal();
    int result = interpretArguments(argc, argv);
    if(result == 1) {
        printf("Error! No arguments specified or unknown parameters. Type masm -h to open the help page. \n Exiting...\n");
    }
    
}
