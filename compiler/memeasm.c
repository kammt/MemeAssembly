#include <stdio.h>  //Printf() function
#include <sys/stat.h>

#include <getopt.h> //Getopt_long function

#include "compiler.h" //Compiler related functions in a separate file

#include "logger/log.h"

FILE *outputFile;
char *outputFileString = NULL;
char *inputFileString = NULL;
FILE *inputFile;

extern int compileMode;
extern int optimisationLevel;
extern int useStabs;


/**
 * Prints the help page of this command. Launched by using the -h option in the terminal
 */
void printHelpPage(char* programName) {
    printInformationHeader();
    printf("Usage:\n");
    printf("  %s [options] -o outputFile [-i | -d] inputFile\t\tCompiles the specified file into an executable\n", programName);
    printf("  %s [options] -S -o outputFile.S [-i | -d] inputFile\tOnly compiles the specified file and saves it as x86_64 Assembly code\n", programName);
    printf("  %s [options] -O -o outputFile.o [-i | -d] inputFile\tOnly compiles the specified file and saves it an object file\n", programName);
    printf("  %s (-h | --help)\t\t\t\t\tDisplays this help page\n\n", programName);
    printf("Compiler options:\n");
    printf("  -O-1 \t\t- reverse optimisation stage 1: A nop is inserted after every command\n");
    printf("  -O-2 \t\t- reverse optimisation stage 2: A register is moved to and from the Stack after every command\n");
    printf("  -O-3 \t\t- reverse optimisation stage 3: A xmm-register is moved to and from the Stack using movups after every command\n");
    printf("  -O-s \t\t- reverse storage optimisation: The compiled Assembly-code is aligned to 536870912B, creating a 1.5GB executable\n");
    printf("  -O69420 \t- maximum optimisation. Reduces the execution to close to 0s by optimising out your entire code\n");
    printf("  -g \t\t- write debug info into the compiled file. Currently, only the STABS format is supported\n");
    printf("  -i \t\t- enables information logs\n");
    printf("  -d \t\t- enables debug logs\n");
}

void printExplanationMessage(char* programName) {
    printf("Usage: %s -o outputFile [-d | -i] inputFile\n", programName);
}

int main(int argc, char* argv[]) {
    static struct option long_options[] = {
            {"output",  required_argument, 0, 'o'},
            {"help",    no_argument,       0, 'h'},
            {"debug",   no_argument,       0, 'd'},
            {"info",    no_argument,       0, 'i'},
            {"O-1",     no_argument,      &optimisationLevel, -1},
            {"O-2",     no_argument,      &optimisationLevel, -2},
            {"O-3",     no_argument,      &optimisationLevel,-3},
            {"O-s",     no_argument,      &optimisationLevel,-4},
            {"O69420",     no_argument,      &optimisationLevel,69420},
            { 0, 0, 0, 0 }
    };

    int opt;
    int option_index = 0;

    while ((opt = getopt_long_only(argc, argv, "o:hOdigS", long_options, &option_index)) != -1) {
        switch (opt) {
            case 'h':
                printHelpPage(argv[0]);
                return 0;
            case 'S':
                compileMode = 0;
                break;
            case 'O':
                compileMode = 1;
                break;
            case 'd':
                setLogLevel(3);
                break;
            case 'i':
                setLogLevel(2);
                break;
            case 'o':
                outputFileString = optarg;
                break;
            case 'g':
                useStabs = 1;
                break;
            case '?':
            default:
                fprintf(stderr, "Error: Unknown option provided\n");
                printExplanationMessage(argv[0]);
                return 1;
        }
    }

    if(outputFileString == NULL) {
        fprintf(stderr, "Error: No output file specified\n");
        printExplanationMessage(argv[0]);
        return 1;
    } else if(argc < optind + 1) {
        fprintf(stderr, "Error: No input file specified\n");
        printExplanationMessage(argv[0]);
        return 1;
    } else {
        inputFileString = argv[optind];
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

        printDebugMessageWithNumber("Optimisation level is", optimisationLevel);

        if(compileMode == 2) {
            createExecutable(inputFile, outputFileString);
        } else if(compileMode == 1) {
            createObjectFile(inputFile, outputFileString);
        } else {
            outputFile = fopen(outputFileString, "w");
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


            return compile(inputFile, outputFile);
        }
    }
}
