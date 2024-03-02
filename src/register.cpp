#include "register.h"

const std::unordered_map<std::string, reg> regMap = {
    //Before you ask: A script generated that, I didn't type that by hand
    {"rax", reg::rax}, {"rbx", reg::rbx}, {"rcx", reg::rcx}, {"rdx", reg::rdx}, {"rdi", reg::rdi}, {"rsi", reg::rsi}, {"rsp", reg::rsp}, {"rbp", reg::rbp}, {"r8", reg::r8}, {"r9", reg::r9}, {"r10", reg::r10}, {"r11", reg::r11}, {"r12", reg::r12}, {"r13", reg::r13}, {"r14", reg::r14}, {"r15", reg::r15}, {"eax", reg::eax}, {"ebx", reg::ebx}, {"ecx", reg::ecx}, {"edx", reg::edx}, {"edi", reg::edi}, {"esi", reg::esi}, {"esp", reg::esp}, {"ebp", reg::ebp}, {"r8d", reg::r8d}, {"r9d", reg::r9d}, {"r10d", reg::r10d}, {"r11d", reg::r11d}, {"r12d", reg::r12d}, {"r13d", reg::r13d}, {"r14d", reg::r14d}, {"r15d", reg::r15d}, {"ax", reg::ax}, {"bx", reg::bx}, {"cx", reg::cx}, {"dx", reg::dx}, {"di", reg::di}, {"si", reg::si}, {"sp", reg::sp}, {"bp", reg::bp}, {"r8w", reg::r8w}, {"r9w", reg::r9w}, {"r10w", reg::r10w}, {"r11w", reg::r11w}, {"r12w", reg::r12w}, {"r13w", reg::r13w}, {"r14w", reg::r14w}, {"r15w", reg::r15w}, {"al", reg::al}, {"ah", reg::ah}, {"bl", reg::bl}, {"bh", reg::bh}, {"cl", reg::cl}, {"ch", reg::ch}, {"dl", reg::dl}, {"dh", reg::dh}, {"dil", reg::dil}, {"sil", reg::sil}, {"spl", reg::spl}, {"bpl", reg::bpl}, {"r8b", reg::r8b}, {"r9b", reg::r9b}, {"r10b", reg::r10b}, {"r11b", reg::r11b}, {"r12b", reg::r12b}, {"r13b", reg::r13b}, {"r14b", reg::r14b}, {"r15b", reg::r15b},
};

const std::array<std::string, static_cast<unsigned>(reg::num_regs)> regStrTable {{
    "rax", "rbx", "rcx", "rdx", "rdi", "rsi", "rsp", "rbp", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
    "eax", "ebx", "ecx", "edx", "edi", "esi", "esp", "ebp", "r8d", "r9d", "r10d", "r11d", "r12d", "r13d", "r14d", "r15d",
    "ax", "bx", "cx", "dx", "di", "si", "sp", "bp", "r8w", "r9w", "r10w", "r11w", "r12w", "r13w", "r14w", "r15w",
    "al", "ah", "bl", "bh", "cl", "ch", "dl", "dh", "dil", "sil", "spl", "bpl", "r8b", "r9b", "r10b", "r11b", "r12b", "r13b", "r14b", "r15b"
}};

unsigned getRegSize(reg reg) {
    if(reg <= reg::r15) {
        return 64;
    } else if(reg <= reg::r15d) {
        return 32;
    } else if(reg <= reg::r15w) {
        return 16;
    } else {
        return 8;
    }
}
