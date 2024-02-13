#pragma once
#include <functional>
#include <memory>
#include <string_view>

//TODO
class IRGroup;
class Analyser;

enum class commandType {
    funcDef,    //This is a function definition
    ret,        //This returns from a function
    labelDef,   //This defines a label (NOT a function)
    labelUse,   //This uses a defined label (NOT a function)
    normal      //Nothing special about this command
};

struct command_t {
    //The pattern used while parsing. This is just the command syntax, with {} denoting a parameter
    //Example: "{} is brilliant, but I like {}"
    std::string_view pattern;
    //A pointer to the analyser object used for analysis
    std::weak_ptr<Analyser> analyser;
    //A function that creates the IR, based on the parameters provided to the command
    std::function<IRGroup(std::string_view, std::string_view)> generateIR;
    //If this command has a special meaning during analysis, then save that here
    commandType cmdType = commandType::normal;
    //Which parameters are allowed
    std::array<uint8_t, 2> allowedParams;
};
