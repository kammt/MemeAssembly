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


namespace analyser {
    enum class reg {
        rax, rbx, rcx, rdx, rdi, rsi, rsp, rbp, r8, r9, r10, r11, r12, r13, r14, r15,
        eax, ebx, ecx, edx, edi, esi, esp, ebp, r8d, r9d, r10d, r11d, r12d, r13d, r14d, r15d,
        ax, bx, cx, dx, di, si, sp, bp, r8w, r9w, r10w, r11w, r12w, r13w, r14w, r15w,
        al, ah, bl, bh, cl, ch, dl, dh, dil, sil, spl, bpl, r8b, r9b, r10b, r11b, r12b, r13b, r14b, r15b
    };
    unsigned getRegSize(reg reg);

    using register_t = std::pair<reg, bool>; //Register, isPointer
    using immediate_t = int64_t;
    using character_t = unsigned char;
    using label_t = std::string;
    using parameter_t = std::variant<register_t, immediate_t, character_t, label_t>;

    using paramArray_t = std::array<parameter_t, 2>;
    std::optional<paramArray_t> checkParameters(parser::parsedCommand_t& cmd);

    using regMap_t = std::unordered_map<std::string, analyser::reg>;
    extern regMap_t regMap;

    template<typename T>
    std::function<void(parameter_t)> assertParamType = [](parameter_t param) {
        if(!std::holds_alternative<T>(param)) {
            logger::printInternalError(std::format("Mismatching type in IR generation"));
            exit(1);
        }
    };
}


