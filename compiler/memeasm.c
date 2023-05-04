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

#include <stdio.h>
#include <sys/stat.h>
#include <getopt.h>
#include <stdbool.h>
#include <errno.h>
#include <string.h>

#include "compiler.h"
#include "parser/parser.h"
#include "logger/log.h"
extern const char* const versionString;

/**
 * Prints the help page of this command. Launched by using the -h option in the terminal
 */
void printHelpPage(char* programName) {
    printInformationHeader();
    printf("Usage:\n");
    printf(" %s [options] -o outputFile [-i | -d] inputFile\t\tCompiles the specified file into an executable\n", programName);
    printf(" %s [options] -S -o outputFile.S [-i | -d] inputFile\tOnly compiles the specified file and saves it as x86_64 Assembly code\n", programName);
    printf(" %s [options] -O -o outputFile.o [-i | -d] inputFile\tOnly compiles the specified file and saves it an object file\n", programName);
    printf(" %s (-h | --help)\t\t\t\t\tDisplays this help page\n", programName);
    printf(" %s -v\t\t\t\t\t\t\tPrints version information\n\n", programName);
    printf("Compiler options:\n");
    printf(" -O-1 \t\t- reverse optimisation stage 1: A nop is inserted after every command\n");
    printf(" -O-2 \t\t- reverse optimisation stage 2: A register is moved to and from the Stack after every command\n");
    printf(" -O-3 \t\t- reverse optimisation stage 3: A xmm-register is moved to and from the Stack using movups after every command\n");
    printf(" -O-s \t\t- reverse storage optimisation: Intentionally increases the file size by aligning end of the compiled Assembly-code to 536870912B\n");
    printf(" -O69420 \t- maximum optimisation. Reduces the execution to close to 0s by optimising out your entire code\n");
    printf(" -fcompile-mode - Change the compile mode to noob (default), bully, or obfuscated\n");
    printf(" -g \t\t- write debug info into the compiled file. Currently, only the STABS format is supported (Linux-only)\n");
    printf(" -fno-martyrdom - Disables martyrdom\n");
    printf(" -d \t\t- enables debug logs\n");
}

void printExplanationMessage(char* programName) {
    printf("Usage: %s -o outputFile inputFile\n", programName);
}

int main(int argc, char* argv[]) {
    struct compileState compileState = {
        .compileMode = noob,
        .optimisationLevel = none,
        .translateMode = intSISD,
        .outputMode = executable,
        .useStabs = false,
        .compilerErrors = 0,
        .logLevel = normal
    };

    char *outputFileString = NULL;
    FILE *inputFile;

    int optimisationLevel = 0;
    int martyrdom = true;
    const struct option long_options[] = {
            {"output",  required_argument, 0, 'o'},
            {"help",    no_argument,       0, 'h'},
            {"debug",   no_argument,       0, 'd'},
            {"fno-martyrdom",    no_argument,&martyrdom, false},
            {"fcompile-mode",    required_argument,0, 'c'},
            { 0, 0, 0, 0 }
    };

    int opt;
    int option_index = 0;

    // On normal systems one wouldn't have to reset optind, but the WebAssembly version
    // requires global variables to be reset; else option parsing wouldn't work when calling main again
    optind = 1;

    while ((opt = getopt_long_only(argc, argv, "o:hO::dgSv", long_options, &option_index)) != -1) {
        switch (opt) {
            case 'h':
                printHelpPage(argv[0]);
                return 0;
            case 'v':
                //Print the version number, but without the "v" at the beginning
                printf("%s\n", versionString+1);
                return 0;
            case 'S':
                compileState.outputMode = assemblyFile;
                break;
            case 'O':
                if(!optarg) {
                    compileState.outputMode = objectFile;
                } else {
                    char* endptr;
                    errno = 0;
                    long res = strtol(optarg, &endptr, 10);
                    if (errno) {
                        perror("Invalid optimisation level specified");
                        return 1;
                    } else if(endptr == optarg || *endptr != '\0' || (res != 69420 && res != -1 && res != -2 && res != -3)) {
                        fprintf(stderr, "Invalid optimisation level specified: %s\n", optarg);
                        return 1;
                    }

                    compileState.optimisationLevel = res;
                }
                break;
            case 'd':
                compileState.logLevel = debug;
                break;
            case 'o':
                outputFileString = optarg;
                break;
            case 'g':
                #ifdef WINDOWS
                //If we use Windows, STABS does not work - output a warning, but don't do anything
                fprintf(stderr, YEL"Info: -g is not supported under Windows, ignoring..\n"RESET);
                #elif defined(MACOS)
		        //If we use MacOS, STABS does not work - output a warning, but don't do anything
                fprintf(stderr, YEL"Info: -g is not supported under MacOS, ignoring..\n"RESET);
		        #else
                compileState.useStabs = true;
                #endif
                break;
            case 'c': //-fcompile-mode
                if(strcmp(optarg, "bully") == 0) { //Bully mode
                    compileState.compileMode = bully;
                } else if(strcmp(optarg, "obfuscated") == 0) { //obfuscated mode
                    compileState.compileMode = obfuscated;
                } else if(strcmp(optarg, "noob") == 0) { //noob mode
                    compileState.compileMode = noob;
                } else {
                    fprintf(stderr, "Error: invalid compile mode (must be one of \"noob\", \"bully\", \"obfuscated\")\n");
                    return 1;
                }
                break;
            case '?':
                fprintf(stderr, "Error: Unknown option provided\n");
                printExplanationMessage(argv[0]);
                return 1;
        }
    }
    compileState.martyrdom = martyrdom;

    if(outputFileString == NULL) {
        fprintf(stderr, "Error: No output file specified\n");
        printExplanationMessage(argv[0]);
        return 1;
    } else if(argc < optind + 1) {
        fprintf(stderr, "Error: No input file(s) specified\n");
        printExplanationMessage(argv[0]);
        return 1;
    } else {
        //We have one or more input files, check how many there are
        //The first is at optind, the last at argc-1
        uint32_t fileCount = argc - optind;

        //Now allocate fileCount file structs on the heap
        struct file* fileStructs = calloc(fileCount, sizeof(struct file));
        if(fileStructs == NULL) {
            fprintf(stderr, "Critical Error: Memory allocation for file parsing failed");
            exit(EXIT_FAILURE);
        }

        //Open each file one by one and parse it into a "struct file"
        for(int i = optind; i < argc; i++) {
            inputFile = fopen(argv[i], "r");
            //If the pointer is NULL, then the file failed to open. Print an error
            if (inputFile == NULL) {
                perror("Failed to open input file");
                printExplanationMessage(argv[0]);
                return 1;
            }

            //Create a stat struct to check if the file is a regular file. If we did not check for this, an input file like "/dev/urandom" would pass without errors
            struct stat inputFileStat;
            fstat(fileno(inputFile), &inputFileStat);
            if (!S_ISREG(inputFileStat.st_mode)) {
                fprintf(stderr,
                        "Error while opening input file: Your provided file name does not point to a regular file (e.g. it could be a directory, character device or a socket)\n");
                fclose(inputFile);
                printExplanationMessage(argv[0]);
                return 1;
            }

            //Set the attribute "fileName" in the struct, because the parsing function uses this attribute for error printing
            fileStructs[i - optind].fileName = argv[i];

            //Parse file
            printDebugMessage(compileState.logLevel, "Opening file \"%s\" successful, parsing file...", 1, argv[i]);
            parseFile(&fileStructs[i - optind], inputFile, &compileState);
            printDebugMessage(compileState.logLevel, "File parsing done, closing file...", 0);
            fclose(inputFile);
        }
        compileState.fileCount = fileCount;
        compileState.files = fileStructs;

        //Convert our optmisationLevel to a value that our struct can work with to make it more readable later on
        //If optimisationLevel == 0, then leave the value at none (default)
        if(optimisationLevel == -1) {
            compileState.optimisationLevel = o_1;
        } else if (optimisationLevel == -2) {
            compileState.optimisationLevel = o_2;
        } else if (optimisationLevel == -3) {
            compileState.optimisationLevel = o_3;
        } else if (optimisationLevel == -4) {
            compileState.optimisationLevel = o_s;
        } else if (optimisationLevel == 69420) {
            compileState.optimisationLevel = o42069;
        }

        compile(compileState, outputFileString);
    }
}
