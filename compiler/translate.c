#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

#include "log.h" //Printing to the console

#define RED   "\x1B[31m"
#define GRN   "\x1B[32m"
#define YEL   "\x1B[33m"
#define BLU   "\x1B[34m"
#define MAG   "\x1B[35m"
#define CYN   "\x1B[36m"
#define WHT   "\x1B[37m"
#define RESET "\x1B[0m"

/**
 * translate.c:
 * This file concerns the translation of MemeASM code into x86-Assembly. It uses an array of commandPatterns and their respective TranslationPatterns, which are defined below
 * For each line, it attempts to match every commandPattern with the line supplied. If the first tokens don't match, it moves on to the next possible command.
 * If the first word matches, it will try to match all other tokens. If this fails, an error is thrown.
 * The consequence of this design choice is that all commands have to start with a different keyword, or else the translator might choose to parse the wrong pattern.
 * 
 * Adding a new command:
 * 1. Increase the array size of both commandPatterns and translationPatterns by one entry
 * 2. Add the command pattern to the end of the array. r stands for an expected register, v for a register or decimal number
 *    For example, the pattern "my awesome command r" would correctly translate the input 'my awesome command eax', but would throw an error if e.g. 'my cool command ebx' is passed to it
 * 3. Define a translation pattern. The numbers 0 to 2 are placeholders which will be replaced with the first, second of third parameter of the commandPattern
 *    Example: the translation pattern "push 0" with the command pattern and input above will be converted to 'push eax'
 *    Note: The numbers 0-2 can be used if they don't store any parameters. Since the command above only has one parameter, I could use the numbers 1 and 2 without them being replaced
 * Warning: the command and translation patterns have to be at the same position in their respective arrays or the translation will fail!
 */

FILE *srcPointer;
FILE *destPointer;
FILE *analyzerPointer;


char commandPatterns[13][60] = {
        "stonks v",
        "not stonks r",
        "upgrade",
        "fuck go back",
        "guess I'll die",
        "bitconeeeeeeect r v",
        "sneak 100 r",
        "upvote r",
        "downvote r",
        "they're the same picture",
        "corporate needs you to find the difference between r and v",
        "r is great, but I want v",
        "or draw 25"
    };

    char translationPatterns[13][60] = {
        "push 0",
        "pop 0",
        "upgradeMarker:",
        "jmp upgradeMarker",
        "mov eax, [789]",
        "and 0, 1",
        "xor 0, 0",
        "inc 0",
        "dec 0",
        "samePicture:",
        "cmp 0, 1\n\tje samePicture",
        "mov 0, 1",
        "add eax, 25"
    };

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


int generateASM(int lineNum, int opcode, int argCnt, char arguments[3][10]) {
    char result[50] = "";
    char translationPattern[60];

    strcpy(translationPattern, translationPatterns[opcode]);
    /** All looks good. Now we traverse over the implementation pattern character by character. 
    * If the character is a number between 0 and 2, we append the respective arguments to the result string
    * If it is a regular character, we just append the character
    */

    for (int i = 0; i < strlen(translationPattern); i++) {
        //Get the character
        char character = translationPattern[i];
        //Character is a number between 0 and 2?
        if(character >= '0' && character <= '2') { 
            //Convert to an integer
            int index = character - '0';
            //Check if there is an argument with that number. If not, just write the number to the file
            if(index < argCnt) {
                //Append the argument
                strncat(result, arguments[index], 50);
                continue;
            }
        }
        char appendStr[2];
        appendStr[0] = character;
        appendStr[1] = '\0';
        strncat(result, appendStr, 50);
    }
    printDebugMessage("Writing ASM to file:", result);
    fprintf(destPointer, "\t%s\n", result);
    printDebugMessage("Writing Opcode to file...", "");
    fprintf(analyzerPointer, "%d\n", opcode);
    return 0;     
}
      

/**
 * Attempts to compile the specified line with a pattern. It automatically saves arguments and inserts them into a translationPattern after translation
 * @param token A token pointing at the first word in the line
 * @param commandPattern a String showing the pattern of a command. Use r for a register value or v for a decimal value or register. Example: 'stonks v' or 'not stonks r'
 * @param translationPattern a String showing how the command should be translated to x86 Assembly. Use the numbers 0 to 2 as placeholders for the arguments in order of their appearance in the command. Example: 'push 0'
 * @param lineNum an integer holding the current line Number
 * @param opcode the opcode of the current command
 * 
 * @return 0 if successful, 1 if there is a syntax error and -1 if this probably isn't the command at all.
 */
int compileWithPattern(char *token, int lineNum, int opcode) {
    int probing = 1; //Is set to 0 if the first tokens match. This is so that if compilation fails at the first token, there is no error message

    //A save pointer for strtok_r
    char *savePTRcommand;
    
    //Copy the commandPattern out of read-only memory and create the first token
    char commandPatternTMP[60];
    strcpy(commandPatternTMP, commandPatterns[opcode]);
    char *commandToken = strtok_r(commandPatternTMP, " ", &savePTRcommand);

    int argCnt = 0; //A counter for the number of arguments
    char arguments[3][10] = {"", "", ""}; //A command can have a maximum of three arguments with itself a maximum of 10 characters

    while (token != NULL && commandToken != NULL)
    {
        printDebugMessage("commandToken: ", commandToken);
        printDebugMessage("token:", token);
        //Step 1: figure out if we need to compare the exact wording (for a keyword e.g. a command) or if it has to be a valid value (e.g. a register)
        if(strcmp(commandToken, "r") == 0) { //token has to be a register
            if(isValidValue(token, 1) != 0) {
                if(probing == 0) {
                    printSyntaxError("Expected register, but got", token, lineNum);
                    return 1;
                } else return -1; 
            }
            //Token is a valid register, set it as argument
            strcpy(arguments[argCnt], token);
            argCnt++;
            probing = 0;
        } else if(strcmp(commandToken, "v") == 0) { //token has to be a value or a register
            if(isValidValue(token, 0) != 0) {
                if(probing == 0) {
                    printSyntaxError("Expected value or register, but got", token, lineNum);
                    return 1;
                } else return -1; 
            }
            //Token is a valid value/register, set it as argument
            strcpy(arguments[argCnt], token);
            argCnt++;
            probing = 0;
        } else {
            if(strcmp(token, commandToken) != 0) {
                if(probing == 0) {
                    printUnexpectedCharacterError(commandToken, token, lineNum);
                    return 1;
                } else return -1; 
            }
            probing = 0;
        }

        //All looks good, now onto the next token
        commandToken = strtok_r(NULL, " ", &savePTRcommand);
        token = strtok(NULL, " ");
    }

    /**Either the line or the command pattern have reached their end. We now have to check what caused the problem
     * - if both are NULL, then there is no problem!
     * - if the commandToken is NULL, then we should have been at the end of the line. Check if the rest is equal to 'or draw 25'. If not, send an error
     * - if the token is NULL, then it is too short, send an error
     */
    if(token == NULL && commandToken == NULL) {
        generateASM(lineNum, opcode, argCnt, arguments);
    } else if(commandToken == NULL) {
        int result = compileWithPattern(token, lineNum, 12);
        if(result == -1) {
            //It isn't or draw 25, so it's an invalid character. Throw an error
            printUnexpectedCharacterError("end of line", token, lineNum);
            return 1;
        } else return result;
    } else {
        //token is NULL, but commandToken isn't
        printUnexpectedCharacterError(commandToken, "end of line", lineNum);
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
int translateLine(char line[], int lineNum, FILE *destPTR, FILE *analyzerPTR) {
    removeLineBreak(line);
    destPTR = destPTR;

    char *token = strtok(line, " ");
    if(token != NULL) {
        int result;
        for (int i = 0; i < 12; i++)
        {
            result = compileWithPattern(token, lineNum, i);
            if(result != -1) break; //-1 is returned if this is not the correct command pattern. If it either returned 0 or 1, then it was the correct command pattern. Return the result.
        }

        if(result == -1) {
            //No command was found
            printSyntaxError("No command was found matching first token", token, lineNum);
            return 1;
        }
        return result;
    }
    //If we got here, then the line is empty. Just ignore that line
    return 0;
}


/**
 * Translates the MemeASM File into an x86-Assembly file
 * @param srcPTR a pointer to the source file to be translated
 * @param destPTR a pointer to the destination file. If nonexistent, it will be created
 */
void startTranslation(FILE *srcPTR, FILE *destPTR) {
    //Assign file pointers to file-wide variables
    analyzerPointer = fopen("opcodes", "w");
    srcPointer = srcPTR;
    destPointer = destPTR;
    printDebugMessage("opcodes-File opened for writing", "");

    char line[128];
    int lineNum = 1;

    //Define the main-function
    fprintf(destPTR, "section .text\n");
    fprintf(destPTR, "global main\n");
    fprintf(destPTR, "main:\n");

    printInfoMessage("Source file opened for reading, starting line-by-line analysis");
    while(fgets(line, sizeof(line), srcPTR) != NULL) {
        printDebugMessage("Starting analysis of", line);
        if(translateLine(line, lineNum, destPTR, analyzerPointer) == 1) printErrorASCII();
        lineNum++;
        printDebugMessage("Done, moving on to next line", " ");
    }

    //Finally, insert a ret-statement
    fprintf(destPTR, "\n\tret");

    printInfoMessage("\nDone, closing all files...\n");
    //Close all files
    fclose(srcPTR);
    fclose(destPTR);
    fclose(analyzerPointer);
}