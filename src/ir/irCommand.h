#pragma once
#include <cstdint>
#include <fstream>
#include <vector>
#include <string>
#include <optional>

using resultVec_t = std::vector<uint8_t>;

class IRCommand {
public:
    virtual void assemble(resultVec_t) = 0;
    virtual void transpile(std::ofstream&) = 0;
};

/*
** Resutls in a label definition in ASM, but does nothing when generating machine code
 */
class LabelDefCommand : public IRCommand {
public:
    explicit LabelDefCommand(std::string str) : labelName(std::move(str)) {};
    void assemble(resultVec_t) override {}; //Does nothing
    void transpile(std::ofstream&) override;
private:
    std::string labelName;
};

/*
** Is just to know where commands start and end in the IR format.
*/
class StartCommand : public IRCommand {
public:
    void assemble(resultVec_t) override {}; //Does nothing
    void transpile(std::ofstream&) override {}; //Does nothing
};

class CallCommand : public IRCommand {
public:
    explicit CallCommand(std::string labelName) : labelName(std::move(labelName)) {};
    void assemble(resultVec_t) override {}; //TODO
    void transpile(std::ofstream&) override;
private:
    std::string labelName;
};

//TODO split into mov + ret?
class RetCommand : public IRCommand {
public:
    RetCommand(int64_t value) : val(value) {};
    RetCommand() : val() {};
    void assemble(resultVec_t) override {}; //TODO
    void transpile(std::ofstream&) override;
private:
    std::optional<int64_t> val;
};
