#pragma once
#include <functional>
#include <memory>
#include <string_view>
#include "analyser/analyser.h"
#include "analyser/parameters.h"

enum class commandType {
    funcDef,    //This is a function definition
    ret,        //This returns from a function
    labelDef,   //This defines a label (NOT a function)
    labelUse,   //This uses a defined label (NOT a function)
    normal      //Nothing special about this command
};

#define NO_PARAM 0
#define PARAM_REG64 1
#define PARAM_REG32 2
#define PARAM_REG16 4
#define PARAM_REG8 8
#define PARAM_DECIMAL 16
#define PARAM_CHAR 32
#define PARAM_MONKE_LABEL 64
#define PARAM_FUNC_NAME 128
//Helper macros for register parameter macros
#define PARAM_REG (PARAM_REG64 | PARAM_REG32 | PARAM_REG16 | PARAM_REG8)


struct command_t {
    //The pattern used while parsing. This is just the command syntax, with {} denoting a parameter
    //Example: "{} is brilliant, but I like {}"
    std::string_view pattern;
    //A pointer to the analyser object used for analysis
    std::shared_ptr<analyser::AnalyserBase> analyser;
    //A function that creates the IR, based on the parameters provided to the command
    std::function<void(std::array<analyser::parameter_t, 2>)> generateIR;
    //If this command has a special meaning during analysis, then save that here
    commandType cmdType = commandType::normal;
    //Which parameters are allowed. Specified by or-ing the macros listed above together
    std::array<uint8_t, 2> allowedParams;
};

extern const std::vector<command_t> commandList;
