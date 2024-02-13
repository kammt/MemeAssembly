#pragma once
#include <string_view>

namespace logger {
    void printError(std::string_view filename, size_t line, std::string_view msg);
}
