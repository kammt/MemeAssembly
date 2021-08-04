#include "translator.h"
#include "../logger/log.h"

#include <time.h>
#include <string.h>

extern char* version_string;
extern struct command commandList[];

void translateToAssembly(struct parsedCommand parsedCommand, FILE *outputFile) {
    struct command command = commandList[parsedCommand.opcode];
    char *translationPattern = command.translationPattern;

    size_t strLen = strlen(translationPattern) - command.usedParameters;
    for(int i = 0; i < command.usedParameters; i++) {
        strLen += strlen(parsedCommand.parameters[i]);
    }

    char *translatedLine = malloc(strLen + 3); //Include an extra byte for the null-Pointer and two extra bytes in case []-brackets are needed for a pointer
    if(translatedLine == NULL) {
        fprintf(stderr, "Critical error: Memory allocation for command translation failed!");
        exit(EXIT_FAILURE);
    }
    translatedLine[0] = '\0';

    for(int i = 0; i < strlen(translationPattern); i++) {
        char character = translationPattern[i];
        if(character >= '0' && character <= (char) command.usedParameters + 47) {
            char *parameter = parsedCommand.parameters[character - 48];

            if(parsedCommand.isPointer == (character - 48) + 1) {
                printDebugMessage("\tAppending pointer parameter", parameter);

                //Manually add braces to the string
                size_t currentStrLen = strlen(translatedLine);

                //Append a '['
                translatedLine[currentStrLen] = '[';
                translatedLine[currentStrLen + 1] = '\0';
                //Append the parameter
                strncat(translatedLine, parameter, strLen);
                //Append a ']'
                currentStrLen = strlen(translatedLine);
                translatedLine[currentStrLen] = ']';
                translatedLine[currentStrLen + 1] = '\0';
            } else {
                printDebugMessage("\tAppending parameter", parameter);
                strncat(translatedLine, parameter, strLen);
            }
        } else {
            char appendix[2] = {character, '\0'};
            strncat(translatedLine, appendix, strLen);
        }
    }

    printDebugMessage("\tWriting to file: ", translatedLine);
    if(parsedCommand.opcode != 0) {
        fprintf(outputFile, "\t");
    }
    fprintf(outputFile, "%s\n", translatedLine);

    printDebugMessage("\tDone, freeing memory", "");
    free(translatedLine);
}

void writeToFile(struct commandsArray *commandsArray, FILE *outputFile) {
    time_t t = time(NULL);
    struct tm tm = *localtime(&t);

    fprintf(outputFile, "#\n# Generated by the MemeAssembly compiler %s on %s#\n", version_string, asctime(&tm));
    fprintf(outputFile, ".intel_syntax noprefix\n\n");
    fprintf(outputFile, ".section .data\n\t.LCharacter: .ascii \"a\"\n");
    fprintf(outputFile, ".section .text\n");

    //Traverse the commandsArray to look for any functions
    for(int i = 0; i < commandsArray -> size; i++) {
        if(commandsArray -> arrayPointer[i].opcode == 0 && commandsArray -> arrayPointer[i].translate == 1) {
            //Write the function name with the prefix ".global" to the file
            fprintf(outputFile, ".global %s\n", commandsArray -> arrayPointer[i].parameters[0]);
        }
    }
    fprintf(outputFile, "\n\n");

    for(int i = 0; i < commandsArray -> size; i++) {
        if(i == commandsArray -> randomIndex) {
            fprintf(outputFile, "\t.LConfusedStonks: ");
        }

        if(commandsArray -> arrayPointer[i].translate == 1) {
            printDebugMessageWithNumber("Translating Index:", i);
            translateToAssembly(commandsArray -> arrayPointer[i], outputFile);
        }
    }

    fprintf(outputFile, "\n\nwritechar:\n\tpush rcx\n\tpush r11\n\tpush rax\n\tpush rdi\n\tpush rsi\n\tpush rdx\n\tmov rdx, 1\n\tlea rsi, [rip + .LCharacter]\n\tmov rax, 1\n\tsyscall\n\tpop rdx\n\tpop rsi\n\tpop rdi\n\tpop rax\n\tpop r11\n\tpop rcx\n\t\n\tret");

    printDebugMessage("Done, closing output file", "");
    fclose(outputFile);
}
