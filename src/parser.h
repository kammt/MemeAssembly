#pragma once
#include <filesystem>

namespace parser {
    /**
     *Receives a file, and parses it line by line.
     *Nothing is returned, but the command IR is added to
     *the respective vector
     **/
    void parseFile(std::ifstream& input);
}
