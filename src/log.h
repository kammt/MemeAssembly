#pragma once
#include <string_view>

namespace logger {
    extern size_t compileErrors;
    void printError(std::string_view filename, size_t line, std::string_view msg);
    void printErrorASCII();
    void printHelpPage();
}
