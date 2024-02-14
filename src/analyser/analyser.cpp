#include "analyser.h"
#include "../command.h"
#include "../compileOpts.h"
#include "../parser/parser.h"
#include "../log.h"
#include <format>
#include <iostream>

namespace analyser {
    void DefinitionAnalyser::commandEncountered(parser::parsedCommand_t& cmd) {
        commandType cmdType {commandList[cmd.opcode].cmdType};
        if(cmdType == commandType::funcDef || cmdType == commandType::labelDef) {
            if(labelSet.find(cmd.params[0]) != labelSet.end()) {
                logger::printError(cmd.filename, cmd.line, std::format("redefinition of {} \"{}\"", this->name, cmd.params[0]));
            } else {
                labelSet.emplace(cmd.params[0]);
            }
        }
    }

    void DefinitionAnalyser::analysisEnd() {
        //If we analyser functions and don't create an executable, then we actually don't care about this error
        if(this->name == "function" && compileOpts.outputMd != outputMode::executable) {
            errorCandidates.clear();
        }

        for(parser::parsedCommand_t& cmd : errorCandidates) {
            if(labelSet.find(cmd.params[0]) == labelSet.end()) {
                logger::printError(cmd.filename, cmd.line, std::format("use of undefined {} \"{}\"", this->name, cmd.params[0]));
            }
        }
    }

    void OneLabelJumpAnalyser::commandEncountered(parser::parsedCommand_t& cmd) {
        commandType cmdType {commandList[cmd.opcode].cmdType};
        if(cmdType == commandType::labelDef) {
            if(definition.has_value()) {
                logger::printError(cmd.filename, cmd.line, std::format("redefinition of {} (already defined in {}:{})", this->name, definition.value().filename, definition.value().line));
            } else {
                definition = {cmd};
            }
        } else if(cmdType == commandType::labelUse) {
            if(!definition) {
                errorCandidates.emplace_back(cmd);
            }
        }
    }

    void OneLabelJumpAnalyser::endOfFile() {
        if(definition) {
            errorCandidates.clear();
        } else {
            for(parser::parsedCommand_t cmd : errorCandidates) {
                logger::printError(cmd.filename, cmd.line, std::format("jump to undefined \"{}\"-label", name));
            }
            errorCandidates.clear();
        }
    }

    void CountOccurrenceAnalyser::commandEncountered(parser::parsedCommand_t& cmd) {
    occurrences++;
    }
}
