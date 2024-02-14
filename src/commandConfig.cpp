#include <memory>
#include <vector>
#include "analyser/analyser.h"
#include "analyser/parameters.h"
#include "command.h"
#include "ir/irCommand.h"

std::shared_ptr<analyser::AnalyserBase> functionAnalyser = std::make_shared<analyser::DefinitionAnalyser>("function");
std::shared_ptr<analyser::AnalyserBase> monkeAnalyser = std::make_shared<analyser::DefinitionAnalyser>("monke-label");
std::shared_ptr<analyser::AnalyserBase> upgradeAnalyser = std::make_shared<analyser::OneLabelJumpAnalyser>("upgrade-marker");
std::shared_ptr<analyser::AnalyserBase> bananaAnalyser = std::make_shared<analyser::OneLabelJumpAnalyser>("banana-marker");

std::vector<std::unique_ptr<IRCommand>> irCommands {};
auto insertStartCommand = []() { irCommands.emplace_back(std::make_unique<StartCommand>()); };

/*
 * This is where all MemeASM commands are documented.
 * See command.h for an explanation of the struct fields
 */
const std::vector<command_t> commandList {{
    {
        .pattern {"I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun {}"},
        .analyser {functionAnalyser},
        .generateIR = [](analyser::paramArray_t params) {
            analyser::assertParamType<analyser::label_t>(params[0]);
            insertStartCommand();
            irCommands.emplace_back(std::make_unique<LabelDefCommand>(std::get<analyser::label_t>(params[0])));
        },
        .cmdType = commandType::funcDef,
        .allowedParams {PARAM_FUNC_NAME, 0}
    },
    //TODO all other commands

    //The command here is special. It MUST be at the end of this array.
    {
        .pattern {"or draw 25"},
    }
}};
