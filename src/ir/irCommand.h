#pragma once
#include <cstdint>
#include <vector>
#include <string>
#include <optional>

class IRCommand {
public:
    virtual ~IRCommand() = default;
    virtual void assemble(std::vector<uint8_t>) = 0;
};

class CallCommand : public IRCommand {
public:
    CallCommand(std::string labelName) : labelName(std::move(labelName)) {};
    void assemble(std::vector<uint8_t>) override {};
private:
    std::string labelName;
};

class RetCommand : public IRCommand {
public:
    RetCommand(int64_t value) : val(value) {};
    RetCommand() : val() {};
    void assemble(std::vector<uint8_t>) override {};
private:
    std::optional<int64_t> val;
};
