#include <stdio.h>  //Printf() function
#include <stdlib.h> //Exit() function

#include <string.h> //String functions

#include "log.h" //Printing to the console

/**
 * translator.c:
 * This file concerns the translation of MemeASM code into x86-Assembly. It uses an array of commandPatterns and their respective TranslationPatterns, which are defined below
 * For each line, it attempts to match every commandPattern with the line supplied. If the first tokens don't match, it moves on to the next possible command.
 * If the first word matches, it will try to match all other tokens. If this fails, an error is thrown.
 * The consequence of this design choice is that all commands have to start with a different keyword, or else the translator might choose to parse the wrong pattern.
 * 
 * Adding a new command:
 * 1. Increase the array size of both commandPatterns and translationPatterns by one entry
 * 2. Add the command pattern to array, but before "or draw 25". r stands for an expected register, v for a register or decimal number
 *    For example, the pattern "my awesome command r" would correctly translate the input 'my awesome command eax', but would throw an error if e.g. 'my cool command ebx' is passed to it
 * 3. Define a translation pattern. The numbers 0 to 2 are placeholders which will be replaced with the first, second of third parameter of the commandPattern
 *    Example: the translation pattern "push 0" with the command pattern and input above will be converted to 'push eax'
 *    Note: The numbers 0-2 can be used if they don't store any parameters. Since the command above only has one parameter, I could use the numbers 1 and 2 without them being replaced
 * Warning: the command and translation patterns have to be at the same position in their respective arrays or the translation will fail!
 */

FILE *destPointer;

char commandPatterns[18][60] = {
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
    "ah shit, here we go again",
    "parry v you filthy casual r",
    "no, I don't think I will",
    "perfectly balanced as all things should be",
    "what can I say except c",
    "or draw 25" //Insert new commands above this line
};

char translationPatterns[18][60] = {
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
    "jmp main",
    "sub 1, 0",
    "ret",
    "",
    "push eax\n\tmov al, 0\n\tcall writechar\n\tpop eax",
    "add eax, 25" //Insert new commands above this line
};

int commandArraySize = sizeof(translationPatterns) / sizeof(translationPatterns[0]);    

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
 * Checks whether the supplied token is a valid character.
 * @param token The supplied token
 * @return the quoted character if it's valid, -1 otherwise
 */
char* validChar(char *token)
{
    int len = strlen(token);
    if (len == 1) {
        // Backslashes should also work, so we escape them
        if (token[0] == '\\') {
            return "92";
        } else if (token[0] == '\'') {
            return "39";
        } else {
            char* newtok = (char *)malloc(3);
            newtok[0] = '\'';
            strcpy(&newtok[1], token);
            newtok[2] = '\'';
            return newtok;
        }
    } else if (len == 2 && token[0] == '\\') {
        // Escape sequences according to https://en.wikipedia.org/wiki/Escape_sequences_in_C#Table_of_escape_sequences
        // We just return their ASCII number as a string as that can also be used to print the string
        // This is mostly done because when writing '\n' in asm, it prints just '\', which is weird
        switch (token[1])
        {
        case 'b':
            return "8";
        case 't':
            return "9";
        case 'n':
            return "10";
        case 'v':
            return "11";
        case 'f':
            return "12";
        case 'r':
            return "13";
        case '"':
            return "34";
        case '\'':
            return "39";
        case '?':
            return "63";
        case '\\':
            return "92";
        }
    }

    // Invalid
    return -1;
}

/**
 * Checks whether the supplied token is a valid x86 8 bit register
 * @param token The supplied token
 * @return 0 if it's valid, 1 otherwise
 */
int isValid8BitRegister(char *token) {
    if (strcmp(token, "ah") == 0 || strcmp(token, "al") == 0 
     || strcmp(token, "bh") == 0 || strcmp(token, "bl") == 0 
     || strcmp(token, "ch") == 0 || strcmp(token, "cl") == 0 
     || strcmp(token, "dh") == 0 || strcmp(token, "dl") == 0)
    {
        return 0;
    }
    return 1;
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


int generateASM(int lineNum, int opcode, int argCnt, char arguments[3][10], int opcodes[]) {
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
    fprintf(destPointer, "\tline_%d: \n\t%s\n", lineNum, result);
    printDebugMessageWithNumber("Writing the following Opcode to array:", opcode);
    opcodes[lineNum - 1] = opcode;
    return 0;     
}
      

/**
 * Attempts to compile the specified line with a pattern. It automatically saves arguments and inserts them into a translationPattern after translation
 * @param token A token pointing at the first word in the line
 * @param lineNum an integer holding the current line Number
 * @param opcode the opcode of the current command
 * @param opcodes an array in which the opcode for each line is saved
 * 
 * @return 0 if successful, 1 if there is a syntax error and -1 if this probably isn't the command at all.
 */
int compileWithPattern(char *token, int lineNum, int opcode, int opcodes[]) {
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
        } else if (strcmp(commandToken, "c") == 0) { // token has to be a character
            char* escaped = validChar(token);
            if (escaped == -1) {
                // We also allow 8-Bit registers and numbers for characters
                if (isValid8BitRegister(token) == 0 || isValidDigit(token) == 0) {
                    escaped = token;
                } else if (probing == 0)
                {
                    printSyntaxError("Expected character, number or 8 bit register, but got", token, lineNum);
                    return 1;
                } else return -1;
            }
            strcpy(arguments[argCnt], escaped);
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
        generateASM(lineNum, opcode, argCnt, arguments, opcodes);
    } else if(commandToken == NULL) {
        int result = compileWithPattern(token, lineNum, commandArraySize-1, opcodes);
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
 * @param opcodes an array in which the opcodes are written to
 * 
 * @return 0 if successful, 1 otherwise
 */
int translateLine(char line[], int lineNum, FILE *destPTR, int opcodes[]) {
    removeLineBreak(line);
    char tmpLine[128];
    strcpy(tmpLine, line);

    char *token = strtok(tmpLine, " ");
    if(token != NULL) {
        if(strcmp(token, "jmp") == 0) {
            printDebugMessage("Skipping preprocessor-set jump...", "");
            fprintf(destPTR, "\tline_%d: %s\n", lineNum, line);
            return 0;
        }

        int result;
        for (int i = 0; i < commandArraySize-1; i++)
        {
            result = compileWithPattern(token, lineNum, i, opcodes);
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
    opcodes[lineNum-1] = -1;
    printDebugMessage("Line is empty, skipping...", "");
    return 0;
}


/**
 * Translates the MemeASM File into an x86-Assembly file
 * @param file an array containing all lines
 * @param lineCount the number of lines
 * @param opcodes an empty array in which the opcodes of each line will be written to
 * @param destPTR a pointer pointing to the Assembly-file
 */
void startTranslation(char file[][128], int lineCount, int opcodes[], FILE *destPTR) {
    //Assign file pointers to file-wide variables
    destPointer = destPTR;
    printDebugMessage("opcodes-File opened for writing", "");

    char line[128];
    int lineNum = 1;

    printDebugMessage("Defining section.text", "");
    //Define the main-function
    fprintf(destPTR, "section .text\n");
    printDebugMessage("Defining main-function", "");
    fprintf(destPTR, "global main\n");
    fprintf(destPTR, "main:\n");

    printInfoMessage("Internal array received, starting line-by-line analysis");
    for(int i = 0; i < lineCount; i++) {
        strcpy(line, file[i]);
        printDebugMessage("Starting analysis of", line);
        if(translateLine(line, lineNum, destPTR, opcodes) == 1) printErrorASCII();
        lineNum++;
        printDebugMessage("Done, moving on to next line", "\n");
    }

    //Finally, insert a ret-statement
    fprintf(destPTR, "\n\tret");

    //Insert the writechar function used for the "what can I say except c" command
    fprintf(destPTR, "\n\nwritechar:\n\tpush ebx\n\tpush ecx\n\tpush edx\n\tpush eax\n\tmov edx, 1\n\tmov ecx, esp\n\tmov ebx, 1\n\tmov eax, 4\n\tint 128\n\tpop eax\n\tpop edx\n\tpop ecx\n\tpop ebx\n\t\n\tret");

    printInfoMessage("\nDone, closing destination file...\n");
    
    //Close the destination file
    fclose(destPTR);
}


