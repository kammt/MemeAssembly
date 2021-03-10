#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

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
    printf(" Exiting....\n");
    exit(1);
}

void writeLine(FILE *destPTR, char keyword[], char arguments[]){
    fprintf(destPTR, "\t%s %s", keyword, arguments);
}

int isValidDigit(char *token) {
    int j = 0;
    while(j<strlen(token)){
        if(token[j] >= '0' && token[j] <= '9')
            j++;
        else
            return 1;
    }
    return 0;
}

int isValidValue(char *token) {
    if(strcmp(token, "eax") == 0 || strcmp(token, "ax") == 0) {
        return 0;
    } else if(strcmp(token, "eax\n") == 0 || strcmp(token, "ax\n") == 0) {
        return 0;
    } else if(strcmp(token, "ebx") == 0 || strcmp(token, "bx") == 0) {
        return 0;
    } else if(strcmp(token, "ebx\n") == 0 || strcmp(token, "bx\n") == 0) {
        return 0;
    } else if(strcmp(token, "ecx\n") == 0 || strcmp(token, "cx\n") == 0) {
        return 0;
    } else if(strcmp(token, "ecx") == 0 || strcmp(token, "cx") == 0) {
        return 0;
    } else if(strcmp(token, "edx") == 0 || strcmp(token, "dx") == 0) {
        return 0;
    } else if(strcmp(token, "edx\n") == 0 || strcmp(token, "dx\n") == 0) {
        return 0;
    } else if(strcmp(token, "ebp") == 0 || strcmp(token, "esp") == 0 || strcmp(token, "edi") == 0 || strcmp(token, "esi") == 0) {
        return 0;
    } else if(strcmp(token, "ebp\n") == 0 || strcmp(token, "esp\n") == 0 || strcmp(token, "edi\n") == 0 || strcmp(token, "esi\n") == 0) {
        return 0;
    } else if(isValidDigit(token) == 0) {
        return 0;
    } else return 1;       
}

int interpretStonks(char *token, int lineNum, FILE *destPTR) {
    token = strtok(NULL, " ");
    if(token == NULL){
        printf(RED "Error in line %d: Expected value or register after 'stonks'" RESET, lineNum);
        return 1;
    } else if(isValidValue(token) == 0) {
        writeLine(destPTR, "push", token);
        return 0;
    } else {
        printf(RED "Error in line %d: Expected value or register after 'stonks', but got %s" RESET, lineNum, token);
        return 1;
    }
}

int interpretNotStonks(char *token, int lineNum, FILE *destPTR) {
    token = strtok(NULL, " ");
    if(strcmp(token, "stonks") == 0) {
        token = strtok(NULL, " ");
        if(token == NULL){
            printf(RED "Error in line %d: Expected value or register after 'not stonks'" RESET, lineNum);
            return 1;
        } else {
            writeLine(destPTR, "pop", token);
            return 0;
        }
    } else {
        printf(RED "Error in line %d: Expected 'stonks' after 'not'" RESET, lineNum);
        return 1;
    }        
}

int interpretJumpMarker(char *token, int lineNum, FILE *destPTR) {
    token = strtok(NULL, " ");
    if(strcmp(token, "go") == 0) {
        token = strtok(NULL, " ");
        if(strcmp(token, "back") == 0 || strcmp(token, "back\n") == 0) {
            writeLine(destPTR, "jmp", "marker");
            return 0;
        } else {
            printf(RED "Error in line %d: Expected 'back' after 'go'" RESET, lineNum);
            return 1;
        }
    } else {
        printf(RED "Error in line %d: Expected 'go' after 'fuck'" RESET, lineNum);
        return 1;
    }    
}

/**
 * Attempts to interpret the command in this line. If successful, it writes the command to the destination file
 * @param line the line to be interpreted
 * @param lineNum the current Line number
 * @param destPTR the destination file to be written to
 * @return 0 if successful, 1 otherwise
 */
int interpretLine(char line[], int lineNum, FILE *destPTR) {
    char *token = strtok(line, " ");
    if(token != NULL) {
        //printf("Got token: ");
        //printf(token);

        if(strcmp(token, "stonks") == 0) {
            return interpretStonks(token, lineNum, destPTR);
        } else if(strcmp(token, "not") == 0) {
            return interpretNotStonks(token, lineNum, destPTR);                
        } else if(strcmp(token, "upgrade") == 0 || strcmp(token, "upgrade\n") == 0) {
            writeLine(destPTR, "marker:", "\n");
        } else if(strcmp(token, "fuck") == 0) {
            return interpretJumpMarker(token, lineNum, destPTR);   
        }else {
            printf(RED "Error in line %d: Unknown token: %s" RESET, lineNum, token);
            return 1;
        }

    }
}

void compile(FILE *srcPTR, FILE *destPTR) {
    //Variables
    char line[128];
    int lineNum = 1;
    //https://www.programiz.com/c-programming/c-file-input-output
    fprintf(destPTR, "global _start\n");
    fprintf(destPTR, "_start:\n");

    //printf("Source file opened for reading, starting line-by-line analysis\n");
    while(fgets(line, sizeof(line), srcPTR) != NULL) {
        //printf("Analysing: ");
        //printf(line);
        if(interpretLine(line, lineNum, destPTR) == 1) printErrorMessage();
        //printf("Done with line, continuing...\n");
        lineNum++;
    }

    fclose(srcPTR);
    fclose(destPTR);
    printf(GRN "File compiled successfully!" RESET);
    printf("\n");
}

void compileAndRun(FILE *srcPTR) {
    FILE *destPTR = fopen("tmp.asm","w");
    compile(srcPTR, destPTR);
    system("nasm -f elf32 -o tmp.o tmp.asm");
    system("ld -m elf_i386 -o tmp tmp.o");
    system("./tmp");
}