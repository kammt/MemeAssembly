#include "randomCommands.h"
#include "../logger/log.h"

#include <stdlib.h>
#include <time.h>

/**
 * Chooses a random line of code in which a random jump marker will be inserted. This is going to be the jump point for all
 * instances of "confused stonks"
 * @param commandsArray the parsed commands
 * @param confusedStonksOpcode the opcode of this command. It is not used
 */
void setConfusedStonksJumpLabel(struct commandsArray *commandsArray, int confusedStonksOpcode) {
    srand(time(NULL));
    commandsArray -> randomIndex = (int) (rand() % (commandsArray ->size));
    printDebugMessageWithNumber("Chose random line for jump marker:", commandsArray -> randomIndex);
}

/**
 * Checks how many times "perfectly balanced as all things should be" occurred and depending on this, marks random lines as to be ignored
 * @param commandsArray the parsed commands
 * @param perfectlyBalancedOpcode the opcode of this command
 */
void chooseLinesToBeDeleted(struct commandsArray *commandsArray, int perfectlyBalancedOpcode) {
    printDebugMessage("Starting analysis of the \"Perfectly balanced...\" command", "");
    int perfectlyBalancedUsed = 0;
    for(int i = 0; i < commandsArray -> size; i++) {
        if(commandsArray -> arrayPointer[i].opcode == perfectlyBalancedOpcode) {
            perfectlyBalancedUsed++;
        }
    }

    printDebugMessageWithNumber("\tamount of times perfectly balanced was used:", perfectlyBalancedUsed);
    //Calculating the number of lines to be deleted
    int linesToBeKept = commandsArray -> size;
    for(int i = 0; i < perfectlyBalancedUsed; i++) {
        linesToBeKept /= 2;
    }
    int linesToBeDeleted = commandsArray -> size - linesToBeKept;

    printDebugMessageWithNumber("\tamount of lines to be deleted:", linesToBeDeleted);
    if(linesToBeDeleted > 0) {
        printThanosASCII(linesToBeDeleted);

        srand(time(NULL));
        int selectedLines = 0;
        int lines[linesToBeDeleted];
        while(selectedLines < linesToBeDeleted) {
            //Generate a random line
            int randomLine = (int) (rand() % (commandsArray ->size));
            printDebugMessageWithNumber("\tGenerated random line:", randomLine);

            //Check if it was already selected
            for(int i = 0; i < selectedLines; i++) {
                if(randomLine == lines[i])  {
                    randomLine = -1;
                    break;
                }
            }
            //If it was already selected, generate a new line
            if(randomLine == -1) {
                printDebugMessage("\t\tLine was already generated", "");
                continue;
            }

            //If not, add it to the array
            lines[selectedLines++] = randomLine;
            //Set the command at this index as to be ignored
            commandsArray -> arrayPointer[randomLine].translate = 0;
            printDebugMessage("\t\tLine was added to the list", "");
        }
    }
    printDebugMessage("Analysis of the \"Perfectly balanced...\" command done", "");
}