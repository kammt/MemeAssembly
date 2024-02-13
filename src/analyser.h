#pragma once
#include <unordered_set>
#include <string_view>
#include <string>
#include <vector>
#include <optional>
#include "parser.h"

class Analyser {
public:
    virtual ~Analyser() = default;
    virtual void commandEncountered(parser::parsedCommand_t&) = 0;
    //These functions may be called multiple times at end of file or analysis
    virtual void endOfFile() {};
    virtual void analysisEnd() {};
};

//Functions, monke labels
class DefinitionAnalyser : public Analyser {
public:
    DefinitionAnalyser(std::string name) : name{name} {};
    void commandEncountered(parser::parsedCommand_t&) override;
    void analysisEnd() override;
private:
    std::unordered_set<std::string> labelSet;
    std::vector<parser::parsedCommand_t> errorCandidates;
    std::string name;
};

//Upgrade, fuck go back; banana, where banana
class OneLabelJumpAnalyser : public Analyser {
public:
    OneLabelJumpAnalyser(std::string name) : name{name} {}
    void commandEncountered(parser::parsedCommand_t&) override;
    void endOfFile() override;
private:
    std::string name;
    std::optional<parser::parsedCommand_t> definition;
    std::vector<parser::parsedCommand_t> errorCandidates;
};

class CountOccurrenceAnalyser : public Analyser {
   void commandEncountered(parser::parsedCommand_t&) override;
   size_t occurrences;
};
