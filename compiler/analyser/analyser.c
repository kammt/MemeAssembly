/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021-2023 Tobias Kamm

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

#include "analyser.h"
#include "../logger/log.h"

extern struct command commandList[NUMBER_OF_COMMANDS];

void analyseCommands(struct compileState* compileState) {
    struct commandLinkedList* commandLinkedList[NUMBER_OF_COMMANDS] = {NULL};

    //Traverse all files
    for(unsigned i = 0; i < compileState->fileCount; i++) {
        struct file file = compileState->files[i];

        //Traverse all functions
        for(unsigned j = 0; j < file.functionCount; j++) {
            struct function function = file.functions[j];

            //Traverse all commands
            for(unsigned k = 0; k < function.numberOfCommands; k++) {
                struct parsedCommand parsedCommand = function.commands[k];
                //Analyse parameters
                checkParameters(&function.commands[k], compileState->files[i].fileName, compileState);

                //Add to command's linkedList
                //Create Linked List item
                struct commandLinkedList* commandLinkedListItem = malloc(sizeof(struct commandLinkedList));
                if(commandLinkedListItem == NULL) {
                    fprintf(stderr, "Critical error: Memory allocation for command analysis failed!");
                    exit(EXIT_FAILURE);
                }

                //Fill struct
                commandLinkedListItem->next = NULL;
                commandLinkedListItem->definedInFile = i;
                commandLinkedListItem->command = &function.commands[k];

                //If there is no linked list yet, make this the first item
                //If there are items, add it to the end of the list
                struct commandLinkedList* lastItem = commandLinkedList[parsedCommand.opcode];
                if(lastItem == NULL) {
                    commandLinkedList[parsedCommand.opcode] = commandLinkedListItem;
                } else {
                    while (lastItem->next != NULL) {
                        lastItem = lastItem->next;
                    }

                    lastItem->next = commandLinkedListItem;
                }
            }
        }
    }

    //Now go through each command and call its analysis function - if it has one
    for(unsigned i = 0; i < NUMBER_OF_COMMANDS; i++) {
        struct command command = commandList[i];
        if(command.analysisFunction != NULL) {
            command.analysisFunction(commandLinkedList, i, compileState);
        }
    }


    //Checks done, freeing memory
    printDebugMessage(compileState->logLevel, "Analysis done, freeing memory", 0);
    for(unsigned i = 0; i < NUMBER_OF_COMMANDS; i++) {
        struct commandLinkedList* itemToBeFreed = commandLinkedList[i];
        while (itemToBeFreed != NULL) {
            struct commandLinkedList* toBeFreed = itemToBeFreed;
            itemToBeFreed = itemToBeFreed->next;
            free(toBeFreed);
        }
    }
}
