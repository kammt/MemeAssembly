#pragma once
#include <filesystem>

namespace parser {
    struct parsedCommand_t {
        std::string_view filename;
        size_t line;
        unsigned opcode;
        std::string_view param1;
    };

    /**
     *Receives a file, and parses it line by line.
     *Nothing is returned, but the command IR is added to
     *the respective vector
     **/
    void parseFile(std::ifstream& input);
}
