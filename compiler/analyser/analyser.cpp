#include "analyser.h"
#include "../commands.h"
#include "../logger/log.h"
#include <iostream>
#include <format>


void FunctionDefAnalyser::commandEncountered(struct compileState* compileState, struct parsedCommand* parsedCommand) {
    if(CMD_ISFUNCDEF(parsedCommand->opcode)) {
        functionSet.emplace(parsedCommand->parameters[0]);
    } else if(CMD_ISFUNCCALL(parsedCommand->opcode)) {
        if(functionSet.find(parsedCommand->parameters[0]) == functionSet.end()) {
            //Function either does not exist or hasn't been found yet
            errorCandidates.emplace_back(parsedCommand);
        }
    }
}

void FunctionDefAnalyser::endOfFile(struct compileState* compileState) {

}

void FunctionDefAnalyser::analysisEnd(struct compileState* compileState) {
    //If we don't create an executable, then we actually don't care about this error
    if(compileState->outputMode != executable) {
        errorCandidates.clear();
    }

    for(struct parsedCommand* parsedCommand : errorCandidates) {
        if(functionSet.find(parsedCommand->parameters[0]) == functionSet.end()) {
            if(compileState->compileMode != bully) {
                printError(parsedCommand, compileState, std::format("call to undefined function \"{}\"", parsedCommand->parameters[0]));
            } else {
                parsedCommand->translate = 0;
            }
        }
    }
    errorCandidates.clear();
}

void OneLabelJumpAnalyser::commandEncountered(struct compileState* compileState, struct parsedCommand* parsedCommand) {
    if(CMD_ISLABELDEF(parsedCommand->opcode)) {
        if(definition) {
            if(compileState->compileMode != bully) {
                printError(parsedCommand, compileState, std::format("redefinition of \"{}\"-label (already defined in line {})", name, definition->lineNum));
            } else {
                parsedCommand->translate = 0;
            }
        } else {
            definition = parsedCommand;
        }
    } else if(CMD_ISLABELUSE(parsedCommand->opcode)) {
        if(!definition) {
            errorCandidates.emplace_back(parsedCommand);
        }
    }
}

void OneLabelJumpAnalyser::endOfFile(struct compileState* compileState) {
    if(definition) {
        errorCandidates.clear();
    } else {
        for(struct parsedCommand* errorCandidate : errorCandidates) {
            if(compileState->compileMode != bully) {
                printError(errorCandidate, compileState, std::format("jump to undefined \"{}\"-label", name));
            } else {
                errorCandidate->translate = 0;
            }
        }
        errorCandidates.clear();
    }
}

void OneLabelJumpAnalyser::analysisEnd(struct compileState* compileState) {

}
