#include <memory>
#include <vector>
#include "analyser.h"
#include "command.h"

std::shared_ptr<Analyser> functionAnalyser = std::make_shared<DefinitionAnalyser>("function");

/*
 * This is where all MemeASM commands are documented.
 * See command.h for an explanation of the struct fields
 */
const std::vector<command_t> commandList {{
    {
        .pattern {"I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun {}"},
        .analyser {functionAnalyser},
        .generateIR {},
        .cmdType = commandType::funcDef,
        .allowedParams {0, 0}
    },
    //TODO all other commands

    //The command here is special. It MUST be at the end of this array.
    {
        .pattern {"or draw 25"},
    }
}};
