#pragma once
#include "../parser/parser.h"
#include "../log.h"
#include <cstdint>
#include <functional>
#include <unordered_map>
#include <string>
#include <variant>
#include <optional>
#include <format>
#include "../register.h"

namespace analyser {
    using register_t = std::pair<reg, bool>; //Register, isPointer
    using immediate_t = int64_t;
    using character_t = unsigned char;
    using label_t = std::string;
    using parameter_t = std::variant<register_t, immediate_t, character_t, label_t>;

    using paramArray_t = std::array<parameter_t, 2>;
    std::optional<paramArray_t> checkParameters(parser::parsedCommand_t& cmd);


    template<typename T>
    std::function<void(parameter_t)> assertParamType = [](parameter_t param) {
        if(!std::holds_alternative<T>(param)) {
            logger::printInternalError(std::format("Mismatching type in IR generation"));
            exit(1);
        }
    };
}


