#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

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

int upgradeMarkerDefined = 0; //Is set to 1 if 'upgrade' is defined somewhere. That way, the compiler can check if it is defined multiple times or 'fuck go back' is used without it
int upgradeJumpDefined = 0; //Is set to the line in which 'fuck go back' is in. This is so that the jump marker can be defined under that line without throwing an error

int pictureMarkerDefined = 0; //Is set to 1 if 'they're the same picture' is defined somewhere. That way, the compiler can check if it is defined multiple times or 'fuck go back' is used without it
int pictureJumpDefined = 0; //Is set to the line in which 'coporate needs you to find...' is in. This is so that the jump marker can be defined under that line without throwing an error

int perfectlyBalanced = 0; //Is set to 1 if 'perfectly balanced as all things should be' is defined somewhere

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
 * Called when the keyword is 'or'. It checks if it is a valid or draw 25 command and if so writes it to the file
 * @param token The supplied token
 * @param lineNum The current line Number in the source file
 * @param destPTR a pointer to the destination file
 * @return 0 if successfully compiled, 1 otherwise
 */
int interpretDraw25(char *token, int lineNum, FILE *destPTR) {
    token = strtok(NULL, " ");
    if(strcmp(token, "draw") == 0) {

        token = strtok(NULL, " "); 
        if(strcmp(token, "25\n") == 0 || strcmp(token, "25") == 0) {
            token = strtok(NULL, " ");

            if(token == NULL) {
                fprintf(destPTR, "\tadd eax, 25\n"); //Command successful, write it to the file
                //If there was a jump command in that line, we now have to reverse its side effects
                if(upgradeJumpDefined == lineNum) upgradeJumpDefined = 0;
                if(pictureJumpDefined == lineNum) pictureJumpDefined = 0;
                return 0;
            } else {
                printf(RED "Error in line %d: Expected end of line after 'or draw 25', but instead got '%s'" RESET, lineNum, token);
                return 1;
            }   

        } else {
            printf(RED "Error in line %d: Expected 25 after 'draw'" RESET, lineNum);
            return 1;
        }

    } else {
        printf(RED "Error in line %d: Expected 'draw' after 'or'" RESET, lineNum);
        return 1;
    }
}

int writeLine(FILE *destPTR, char keyword[], char arguments[], char *token, int lineNum){
    //First, we check two scenarios:
    //1. There are other invalid tokens after the supplied command. This should cause an error
    //2. There is 'or draw 25' after this command
    
    token = strtok(NULL, " "); //Attempt to get the next token
    if(token == NULL) {
        fprintf(destPTR, "\t%s %s\n", keyword, arguments); //write the command and leave
        return 0;
    } else if(strcmp(token, "or") == 0) {
        return interpretDraw25(token, lineNum, destPTR);
    } else {
        printf(RED "Error in line %d: Expected end of line after command, but instead got '%s'" RESET, lineNum, token);
        return 1;
    }
}

/**
 * Removes the \n from a string if it is present at the end of the string
 * @param token the token
 */
void removeLineBreak(char *token) {
    if(token[strlen(token)-1] == '\n') {
        token[strlen(token)-1] = '\0';
    }
}

/**
 * Checks whether the supplied token is a valid decimal digit,
 * @param token The supplied token
 * @return 0 if it's a valid number, 1 otherwise
 */
int isValidDigit(char *token) {
    int j = 0;
    while(j<strlen(token)){
        if(token[j] >= '0' && token[j] <= '9')
            j++;
        else if(token[j] == '\n') {
            return 0;
        } else
            return 1;
    }
    return 0;
}

/**
 * Checks whether the supplied token is a valid decimal digit or x86 register keyword
 * @param token The supplied token
 * @param onlyRegister 1 if the value is only allowed to be a x86 register
 * @return 0 if it's valid, 1 otherwise
 */
int isValidValue(char *token, int onlyRegister) {
    if(strcmp(token, "eax") == 0 || strcmp(token, "ax") == 0) {
        return 0;
    }else if(strcmp(token, "ebx") == 0 || strcmp(token, "bx") == 0) {
        return 0;
    } else if(strcmp(token, "ecx") == 0 || strcmp(token, "cx") == 0) {
        return 0;
    } else if(strcmp(token, "edx") == 0 || strcmp(token, "dx") == 0) {
        return 0;
    } else if(strcmp(token, "ebp") == 0 || strcmp(token, "esp") == 0 || strcmp(token, "edi") == 0 || strcmp(token, "esi") == 0) {
        return 0;
    } else if(onlyRegister == 0 && isValidDigit(token) == 0) {
        return 0;
    } else return 1;       
}

/**
 * Called when the first keyword is 'stonks'. It checks if it is a valid stonks command and if so writes it to the file
 * @param token The supplied token
 * @param lineNum The current line Number in the source file
 * @param destPTR a pointer to the destination file
 * @return 0 if successfully compiled, 1 otherwise
 */
int interpretStonks(char *token, int lineNum, FILE *destPTR) {
    token = strtok(NULL, " ");
    if(token == NULL){
        printf(RED "Error in line %d: Expected value or register after 'stonks'" RESET, lineNum);
        return 1;
    } else if(isValidValue(token, 0) == 0) {
        return writeLine(destPTR, "push", token, token, lineNum);
        return 0;
    } else {
        printf(RED "Error in line %d: Expected value or register after 'stonks', but got %s" RESET, lineNum, token);
        return 1;
    }
}

/**
 * Called when the first keyword is 'not'. It checks if it is a valid not stonks command and if so writes it to the file
 * @param token The supplied token
 * @param lineNum The current line Number in the source file
 * @param destPTR a pointer to the destination file
 * @return 0 if successfully compiled, 1 otherwise
 */
int interpretNotStonks(char *token, int lineNum, FILE *destPTR) {
    token = strtok(NULL, " ");
    if(strcmp(token, "stonks") == 0) {
        token = strtok(NULL, " ");
        if(token == NULL){
            printf(RED "Error in line %d: Expected value or register after 'not stonks'" RESET, lineNum);
            return 1;
        }  if(isValidValue(token, 1) == 0) {
            return writeLine(destPTR, "pop", token, token, lineNum);
        } else {
            printf(RED "Error in line %d: Expected register after 'not stonks', but got %s" RESET, lineNum, token);
            return 1;
        }
    } else {
        printf(RED "Error in line %d: Expected 'stonks' after 'not'" RESET, lineNum);
        return 1;
    }        
}

/**
 * Called when the first keyword is 'fuck'. It checks if it is a valid 'fuck go back' command and if so writes it to the file
 * @param token The supplied token
 * @param lineNum The current line Number in the source file
 * @param destPTR a pointer to the destination file
 * @return 0 if successfully compiled, 1 otherwise
 */
int interpretJumpMarker(char *token, int lineNum, FILE *destPTR) {
    token = strtok(NULL, " ");
    if(strcmp(token, "go") == 0) {
        token = strtok(NULL, " ");
        if(strcmp(token, "back") == 0) {
            upgradeJumpDefined = lineNum;
            return writeLine(destPTR, "jmp", "upgrade", token, lineNum);
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
 * Called when the first keyword is 'they're'. It checks if it is a valid 'they're the same picture' command and if so writes it to the file
 * @param token The supplied token
 * @param lineNum The current line Number in the source file
 * @param destPTR a pointer to the destination file
 * @return 0 if successfully compiled, 1 otherwise
 */
int interpretSamePicture(char *token, int lineNum, FILE *destPTR) {
    token = strtok(NULL, " ");
    if(strcmp(token, "the") == 0) {
        token = strtok(NULL, " ");
        if(strcmp(token, "same") == 0) {
            token = strtok(NULL, " ");
            if(strcmp(token, "picture") == 0) {
                if(pictureMarkerDefined == 0) {
                    pictureMarkerDefined = 1;
                    return writeLine(destPTR, "samePicture:", "", token, lineNum);
                } else {
                    printf(RED "Error in line %d: 'they're the same picture' cannot be defined twice" RESET, lineNum);
                    return 1;
                }
            } else {
                printf(RED "Error in line %d: Expected 'picture' after 'same'" RESET, lineNum);
                return 1;
            }    
        } else {
            printf(RED "Error in line %d: Expected 'same' after 'the'" RESET, lineNum);
            return 1;
        }
    } else {
        printf(RED "Error in line %d: Expected 'the' after 'they're'" RESET, lineNum);
        return 1;
    }    
}


/**
 * Called when the first keyword is 'guess'. It checks if it is a valid 'guess I'll die' command and if so writes it to the file
 * @param token The supplied token
 * @param lineNum The current line Number in the source file
 * @param destPTR a pointer to the destination file
 * @return 0 if successfully compiled, 1 otherwise
 */
int interpretGuessIllDie(char *token, int lineNum, FILE *destPTR) {
    token = strtok(NULL, " ");
    if(strcmp(token, "I'll") == 0) {
        token = strtok(NULL, " ");
        if(strcmp(token, "die") == 0) {
            return writeLine(destPTR, "mov", "eax, [0x0000]\n", token, lineNum);
            return 0;
        } else {
            printf(RED "Error in line %d: Expected 'die' after 'I'll'" RESET, lineNum);
            return 1;
        }
    } else {
        printf(RED "Error in line %d: Expected 'I'll' after 'fuck'" RESET, lineNum);
        return 1;
    }    
}   

/**
 * Called when the first keyword is 'bitconeeeeeeect'. It checks if it is a valid 'bitconeeeeeeect' command and if so writes it to the file
 * @param token The supplied token
 * @param lineNum The current line Number in the source file
 * @param destPTR a pointer to the destination file
 * @return 0 if successfully compiled, 1 otherwise
 */
int interpretBitconnect(char *token, int lineNum, FILE *destPTR) {
    token = strtok(NULL, " ");
    if(isValidValue(token, 1) == 0) {
        char firstToken[128]; //Save the first token for later
        strcpy(firstToken, token);
        strcat(firstToken, ", ");

        token = strtok(NULL, " ");
        if(isValidValue(token, 0) == 0) {
            strcat(firstToken, token);
            return writeLine(destPTR, "and", firstToken, token, lineNum);
            return 0;
        } else {
            printf(RED "Error in line %d: Expected value or register, but got %s" RESET, lineNum, token);
            return 1;
        }
    } else {
        printf(RED "Error in line %d: Expected register, but got %s" RESET, lineNum, token);
        return 1;
    }    
}


/**
 * Called when the first keyword is 'sneak'. It checks if it is a valid 'sneak 100' command and if so writes it to the file
 * @param token The supplied token
 * @param lineNum The current line Number in the source file
 * @param destPTR a pointer to the destination file
 * @return 0 if successfully compiled, 1 otherwise
 */
int interpretSneak100(char *token, int lineNum, FILE *destPTR) {
    token = strtok(NULL, " ");
    if(strcmp(token, "100") == 0) {
        token = strtok(NULL, " ");
        if(isValidValue(token, 1) == 0) {
            char arguments[10];
            strcpy(arguments, token);
            strcat(arguments, ", ");
            strcat(arguments, token);
            return writeLine(destPTR, "xor", arguments, token, lineNum);
            return 0;
        } else {
            printf(RED "Error in line %d: Expected register, but got %s" RESET, lineNum, token);
            return 1;
        }
    } else {
        printf(RED "Error in line %d: Expected '100' after 'sneak'" RESET, lineNum);
        return 1;
    }    
}  

/**
 * Called when the first keyword is a register. It checks if it is a valid 'this is great, but I want this' command and if so writes it to the file
 * @param token The supplied token
 * @param lineNum The current line Number in the source file
 * @param destPTR a pointer to the destination file
 * @return 0 if successfully compiled, 1 otherwise
 */
int interpretMov(char *token, int lineNum, FILE *destPTR) {
    char arguments[20];
    strcpy(arguments, token);

    token = strtok(NULL, " ");
    if(strcmp(token, "is") == 0) {
        token = strtok(NULL, " ");
        if(strcmp(token, "great,") == 0) {
            token = strtok(NULL, " ");
            if(strcmp(token, "but") == 0) {
                token = strtok(NULL, " ");
                if(strcmp(token, "I") == 0) {
                    token = strtok(NULL, " ");
                    if(strcmp(token, "want") == 0) {
                        token = strtok(NULL, " ");
                        if(isValidValue(token, 0) == 0) {
                            strcat(arguments, ", ");
                            strcat(arguments, token);
                            return writeLine(destPTR, "mov", arguments, token, lineNum);
                        } else {
                            printf(RED "Error in line %d: Expected value of register, but instead got %s" RESET, lineNum, token);
                            return 1;
                        }
                    } else {
                        printf(RED "Error in line %d: Expected 'want' after 'I'" RESET, lineNum);
                        return 1;
                    }
                } else {
                    printf(RED "Error in line %d: Expected 'I' after 'but'" RESET, lineNum);
                    return 1;
                }
            } else {
                printf(RED "Error in line %d: Expected 'but' after 'great,'" RESET, lineNum);
                return 1;
            }
        } else {
            printf(RED "Error in line %d: Expected 'great,' after 'is'" RESET, lineNum);
            return 1;
        }
    } else {
        printf(RED "Error in line %d: Expected 'is' after register name" RESET, lineNum);
        return 1;
    }    
}

/**
 * Called when the first keyword is 'corporate'. It checks if it is a valid 'corporate needs you to find...' command and if so writes it to the file
 * @param token The supplied token
 * @param lineNum The current line Number in the source file
 * @param destPTR a pointer to the destination file
 * @return 0 if successfully compiled, 1 otherwise
 */
int interpretCmp(char *token, int lineNum, FILE *destPTR) {
    char arguments[30];

    token = strtok(NULL, " ");
    if(strcmp(token, "needs") == 0) {
        token = strtok(NULL, " ");
        if(strcmp(token, "you") == 0) {
            token = strtok(NULL, " ");
            if(strcmp(token, "to") == 0) {
                token = strtok(NULL, " ");
                if(strcmp(token, "find") == 0) {
                    token = strtok(NULL, " ");
                    if(strcmp(token, "the") == 0) {
                        token = strtok(NULL, " ");
                        if(strcmp(token, "difference") == 0) {
                            token = strtok(NULL, " ");
                            if(strcmp(token, "between") == 0) {
                                token = strtok(NULL, " ");
                                if(isValidValue(token, 0) == 0) {
                                    strcpy(arguments, token);
                                    token = strtok(NULL, " ");
                                    if(strcmp(token, "and") == 0) {
                                        token = strtok(NULL, " ");
                                        if(isValidValue(token, 0) == 0) {
                                            strcat(arguments, ", ");
                                            strcat(arguments, token);
                                            strcat(arguments, "\n\tje samePicture");
                                            pictureJumpDefined = lineNum;
                                            return writeLine(destPTR, "cmp", arguments, token, lineNum);
                                        } else {
                                            printf(RED "Error in line %d: Expected value of register, but instead got %s" RESET, lineNum, token);
                                            return 1;
                                        }    
                                    } else {
                                        printf(RED "Error in line %d: Expected 'and' after parameter" RESET, lineNum);
                                        return 1;
                                    }                                        
                                } else {
                                    printf(RED "Error in line %d: Expected value of register, but instead got %s" RESET, lineNum, token);
                                    return 1;
                                }
                            } else {
                                printf(RED "Error in line %d: Expected 'between' after 'difference'" RESET, lineNum);
                                return 1;
                            } 
                        } else {
                            printf(RED "Error in line %d: Expected 'difference' after 'the'" RESET, lineNum);
                            return 1;
                        }                            
                        
                    } else {
                        printf(RED "Error in line %d: Expected 'the' after 'find'" RESET, lineNum);
                        return 1;
                    }
                } else {
                    printf(RED "Error in line %d: Expected 'find' after 'to'" RESET, lineNum);
                    return 1;
                }
            } else {
                printf(RED "Error in line %d: Expected 'to' after 'you'" RESET, lineNum);
                return 1;
            }
        } else {
            printf(RED "Error in line %d: Expected 'you,' after 'needs'" RESET, lineNum);
            return 1;
        }
    } else {
        printf(RED "Error in line %d: Expected 'needs' after 'coporate'" RESET, lineNum);
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
    removeLineBreak(line);
    char *token = strtok(line, " ");
    if(token != NULL) {
        //printf("Got token: ");
        //printf(token);

        if(strcmp(token, "\n") == 0) {
            printf("Info: line %d is empty, skipping\n", lineNum);
        } else if(strcmp(token, "stonks") == 0) {
            return interpretStonks(token, lineNum, destPTR);
        } else if(strcmp(token, "not") == 0) {
            return interpretNotStonks(token, lineNum, destPTR);                
        } else if(strcmp(token, "upgrade") == 0 || strcmp(token, "upgrade\n") == 0) {
            if(upgradeMarkerDefined == 0) {
                upgradeMarkerDefined = 1;
                return writeLine(destPTR, "upgrade:", "", token, lineNum);
            } else {
                printf(RED "Error in line %d: 'upgrade' jump marker can only be defined once" RESET, lineNum);
                return 1;
            }
        } else if(strcmp(token, "fuck") == 0) {
            return interpretJumpMarker(token, lineNum, destPTR);   
        } else if (strcmp(token, "guess") == 0) {
            return interpretGuessIllDie(token, lineNum, destPTR);   
        } else if (strcmp(token, "bitconeeeeeeect") == 0){
            return interpretBitconnect(token, lineNum, destPTR);
        } else if (strcmp(token, "sneak") == 0){
            return interpretSneak100(token, lineNum, destPTR);
        } else if (strcmp(token, "upvote") == 0){
            token = strtok(NULL, " ");
            if(isValidValue(token, 1) == 0) {
                return writeLine(destPTR, "inc", token, token, lineNum);
            } else {
                printf(RED "Error in line %d: Expected register, but got %s" RESET, lineNum, token);
                return 1;
            }
        } else if (strcmp(token, "downvote") == 0){
            token = strtok(NULL, " ");
            if(isValidValue(token, 1) == 0) {
                return writeLine(destPTR, "dec", token, token, lineNum);
            } else {
                printf(RED "Error in line %d: Expected register, but got %s" RESET, lineNum, token);
                return 1;
            }
        } else if (strcmp(token, "they're") == 0) {
            return interpretSamePicture(token, lineNum, destPTR);   
        } else if (strcmp(token, "corporate") == 0) {
            return interpretCmp(token, lineNum, destPTR);   
        } else if (isValidValue(token, 1) == 0){
            return interpretMov(token, lineNum, destPTR);
        } else {            
            printf(RED "Error in line %d: Unknown token: %s" RESET, lineNum, token);
            return 1;
        }

    }
}

/**
 * Attempts to convert the source file to an x86 Assembly file
 * @param srcPTR a pointer to the source file to be compiled
 * @param destPTR a pointer to the destination file. If nonexistent, it will be created
 */
void compile(FILE *srcPTR, FILE *destPTR) {
    //Variables
    char line[128];
    int lineNum = 1;
    fprintf(destPTR, "section .text\n");
    fprintf(destPTR, "global main\n");
    fprintf(destPTR, "main:\n");

    //printf("Source file opened for reading, starting line-by-line analysis\n");
    while(fgets(line, sizeof(line), srcPTR) != NULL) {
        //printf("Analysing: ");
        //printf(line);
        if(interpretLine(line, lineNum, destPTR) == 1) printErrorMessage();
        //printf("Done with line, continuing...\n");
        lineNum++;
    }

    fprintf(destPTR, "\n\tret");
    fclose(srcPTR);
    fclose(destPTR);

    //Now that we traversed the entire file, we still need to check if jumps were defined without markers
    if(pictureJumpDefined != 0 && pictureMarkerDefined == 0) {
        printf(RED "Error in line %d: 'coporate needs you to find the difference between ...' was used, but 'they're the same picture' wasn't defined anywhere" RESET, pictureJumpDefined);
        printErrorMessage();
    } else if(upgradeJumpDefined != 0 && upgradeMarkerDefined == 0) {
        printf(RED "Error in line %d: 'fuck go back' used, but no 'upgrade' marker was found" RESET, upgradeJumpDefined); 
        printErrorMessage();
    }


    printf(GRN "File compiled successfully!" RESET);
    printf("\n");
}

/**
 * Compiles, links and runs the specified memeasm-file
 * @param srcPTR a pointer to the source file to be compiled
 */
void compileAndRun(FILE *srcPTR) {
    FILE *destPTR = fopen("tmp.asm","w");
    compile(srcPTR, destPTR);

    printf(YEL "Calling nasm...\n" RESET);
    fflush( stdout );
    system("nasm -f elf32 -o tmp.o tmp.asm");

    printf(YEL "Linking...\n" RESET);
    fflush( stdout );
    system("gcc tmp.o -g -o tmp -m32 -fno-pie -no-pie");
    
    printf(YEL "Running file...\n" RESET);
    fflush( stdout );
    execv ("tmp", NULL);
    //TODO delete temporary files
}