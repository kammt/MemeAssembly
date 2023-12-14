#ifndef ANALYZER_H_
#define ANALYZER_H_

#include <unordered_set>
#include <string>
#include <vector>

class Analyser {
public:
    virtual ~Analyser() = default;
    virtual void commandEncountered(struct compileState*, struct parsedCommand*) = 0;
    //These functions may be called multiple times at end of file or analysis
    virtual void endOfFile(struct compileState*) = 0;
    virtual void analysisEnd(struct compileState*) = 0;
};

class FunctionDefAnalyser : public Analyser {
public:
    void commandEncountered(struct compileState*, struct parsedCommand*) override;
    void analysisEnd(struct compileState*) override;
    void endOfFile(struct compileState*) override;
private:
    std::unordered_set<std::string> functionSet;
    std::vector<struct parsedCommand*> errorCandidates;
};

//Upgrade, fuck go back; banana, where banana
class OneLabelJumpAnalyser : public Analyser {
public:
    OneLabelJumpAnalyser(std::string name) {
        this->name = name;
    }
    void commandEncountered(struct compileState*, struct parsedCommand*) override;
    void analysisEnd(struct compileState*) override;
    void endOfFile(struct compileState*) override;
private:
    std::string name;
    struct parsedCommand* definition;
    std::vector<struct parsedCommand*> errorCandidates;
};

#endif // ANALYZER_H_
