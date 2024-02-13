#include <vector>
#include "command.h"

/*
 * This is where all MemeASM commands are documented.
 * See command.h for an explanation of the struct fields
 */
const std::vector<command_t> commandList {{
    {
        .pattern {"I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun {}"},
        .analyser {},
        .generateIR {},
        .allowedParams {0, 0}
    },
    //TODO all other commands

    //The command here is special. It MUST be at the end of this array.
    {
        .pattern {"or draw 25"},
    }
}};
