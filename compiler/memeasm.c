#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function
#include <sys/stat.h>


#include <getopt.h> //Getopt_long function

#include "compiler.h" //Compiler related functions in a separate file

#include "log.h"

int mode = 0; //0 = Compile and Run, 1 = Compile only
FILE *outputFile;
FILE *inputFile;

/**
 * Prints the help page of this command. Launched by using the -h option in the terminal
 */
void printHelpPage(char* programName) {
    printInformationHeader();
    printf("Usage:\n");
    printf("  %s -o outputFile [-i | -d] path/to/fileName\t\t\tCompiles the specified file into an executable\n", programName);
    printf("  %s -c -o outputFile [-i | -d] path/to/fileName\t\t\tOnly compiles the specified file and saves it as Assembly code\n", programName);
    printf("  %s (-h | --help)\t\t\t\t\t\tDisplays this help page\n", programName);
}

void printExplanationMessage(char* programName) {
    printf("Usage: %s [-d | -i] inputFile\n", programName);
}

int main(int argc, char* argv[]) {
    static struct option long_options[] = {
            {"output",      required_argument, 0,      'o'},
            {"help",        no_argument,       0,      'h'},
            {"debug",       no_argument,       0,      'd'},
            {"compile",     no_argument,       0,      'c'},
            {"info",        no_argument,       0,      'i'},
    };

    int opt;
    int option_index = 0;

    while ((opt = getopt_long(argc, argv, "o:hcdi", long_options, &option_index)) != -1) {
        switch (opt) {
            case 'h':
                printHelpPage(argv[0]);
                return 0;
            case 'c':
                mode = 1;
                break;
            case 'd':
                setLogLevel(3);
                break;
            case 'i':
                setLogLevel(2);
                break;
            case 'o':
                outputFile = fopen(optarg, "w");
                //If the pointer is NULL, then the file failed to open. Print an error
                if (outputFile == NULL) {
                    perror("Error in option -o");
                    printExplanationMessage(argv[0]);
                    return 1;
                }

                //Create a stat struct to check if the file is a regular file. If we did not check for this, an argument like "-o /dev/urandom" would pass without errors
                struct stat outputFileStat;
                fstat(fileno(outputFile), &outputFileStat);
                if (!S_ISREG(outputFileStat.st_mode)) {
                    fprintf(stderr,
                            "Error in option -o: Your provided file name does not point to a regular file (e.g. it could be a directory, character device or a socket)\n");
                    fclose(outputFile);
                    printExplanationMessage(argv[0]);
                    return 1;
                }
        }
    }

    if(outputFile == NULL) {
        fprintf(stderr, "Error: No output file specified\n");
        printExplanationMessage(argv[0]);
        return 1;
    } else if(argc < optind + 1) {
        fprintf(stderr, "Error: No input file specified\n");
        printExplanationMessage(argv[0]);
        return 1;
    } else {
        inputFile = fopen(argv[optind], "r");
        //If the pointer is NULL, then the file failed to open. Print an error
        if (inputFile == NULL) {
            perror("Error while opening input file");
            printExplanationMessage(argv[0]);
            return 1;
        }

        //Create a stat struct to check if the file is a regular file. If we did not check for this, an argument like "-o /dev/urandom" would pass without errors
        struct stat inputFileStat;
        fstat(fileno(inputFile), &inputFileStat);
        if (!S_ISREG(inputFileStat.st_mode)) {
            fprintf(stderr,
                    "Error while opening input file: Your provided file name does not point to a regular file (e.g. it could be a directory, character device or a socket)\n");
            fclose(inputFile);
            printExplanationMessage(argv[0]);
            return 1;
        }

        if(mode == 0) {
            createExecutable(inputFile);
        } else {
            compile(inputFile, outputFile);
        }
    }
}
