#pragma once
#include <array>
#include <string>
#include <unordered_map>

enum class reg {
    rax=0, rbx, rcx, rdx, rdi, rsi, rsp, rbp, r8, r9, r10, r11, r12, r13, r14, r15,
    eax, ebx, ecx, edx, edi, esi, esp, ebp, r8d, r9d, r10d, r11d, r12d, r13d, r14d, r15d,
    ax, bx, cx, dx, di, si, sp, bp, r8w, r9w, r10w, r11w, r12w, r13w, r14w, r15w,
    al, ah, bl, bh, cl, ch, dl, dh, dil, sil, spl, bpl, r8b, r9b, r10b, r11b, r12b, r13b, r14b, r15b,
    num_regs // Not used as a register, is used to get the total number of registers easily
};

unsigned getRegSize(reg reg);

extern const std::unordered_map<std::string, reg> regMap;
extern const std::array<std::string, static_cast<unsigned>(reg::num_regs)> regStrTable;
