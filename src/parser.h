#pragma once
#include <filesystem>
#include <array>

namespace parser {
    struct parsedCommand_t {
        std::string_view filename;
        size_t line;
        unsigned opcode;
        std::array<std::string, 2> params;
        char isPointer;
    };

    /**
     *Receives a file, and parses it line by line.
     *Nothing is returned, but the command IR is added to
     *the respective vector
     **/
    void parseFile(std::ifstream& input, std::string_view filename);
}
