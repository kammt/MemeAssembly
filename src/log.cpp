#include "log.h"
#include "compileOpts.h"
#include <iostream>
#include <vector>

//ANSI escape sequences for some fancy terminal colors
#define RED   "\x1B[31m"
#define GRN   "\x1B[32m"
#define YEL   "\x1B[33m"
#define BLU   "\x1B[34m"
#define MAG   "\x1B[35m"
#define CYN   "\x1B[36m"
#define WHT   "\x1B[37m"
#define RESET "\x1B[0m"

const std::vector<std::string_view> randomErrorMessages = {
        //sudo
        "you are not in the sudoers file. This incident will be reported.",
        "you are not allowed to run sudo. This incident will be reported.",
        //erno - cause why not?
        "ENOSPC: No space left on device",
        "ENOENT: No such file or directory",
        //glibc
        "double free or corruption (top)",
        "corrupted top size",
        "double free or corruption (!prev)",
        "free: invalid pointer",
        //gcc
        "implicit declaration of function `gets'",
        "invalid type argument of `unary *'",
        "passing arg 2 of `strcpy' makes pointer from integer without a cast",
        "syntax error before '}' token",
        "ld returned exit 1 status",
        "no match for ‘operator==’ in ‘__first.__gnu_cxx::__normal_iterator::operator* [with _Iterator = std::vector*, _Container = std::vector >, __gnu_cxx::__normal_iterator::reference = std::vector&]() == __val’",
        "error: cannot bind rvalue ‘(short unsigned int)((const char*)\"\")’ to ‘short unsigned int&’",
        "no match for 'operator<<' (operand types are 'std::ostream' {aka 'std::basic_ostream<char>'} and 'std::array<int, 1>')",
        "invalid conversion from `int' to `std::_Rb_tree_node<std::pair<const int, double> >*'",
        //ld
        "relocation truncated to fit: R_X86_64_PC32 against symbol `main'",
        "(.ARM.exidx.text._ZNSt8_Rb_treeIiSt4pairIKiSt10shared_ptrIN4SWGL7ContextEEESt10_Select1stIS6_ESt4lessIiESaIS6_EE13_Rb_tree_implISA_Lb1EED2Ev[_ZNSt8_Rb_treeIiSt4pairIKiSt10shared_ptrIN4SWGL7ContextEEESt10_Select1stIS6_ESt4lessIiESaIS6_EE13_Rb_tree_implISA_Lb1EED5Ev]+0x0): relocation truncated to fit: R_ARM_PREL31 against `.text._ZNSt8_Rb_treeIiSt4pairIKiSt10shared_ptrIN4SWGL7ContextEEESt10_Select1stIS6_ESt4lessIiESaIS6_EE13_Rb_tree_implISA_Lb1EED2Ev'",
        "error adding symbols: DSO missing from command line",
        "undefined reference to `main'",
        //Cobol
        "programmer is impolite",
        "programmer is excessively polite",
        //actual MemeASM errors
        "a decimal number cannot be a pointer",
        "a function name cannot be a pointer",
        "invalid parameter combination: 64 Bit arithmetic operation commands require the decimal number to be sign-extendable from 32 Bits",
        "function does not return",
        "an executable cannot be created if no main function exists"
};

namespace logger {
    size_t compileErrors {0};

    void printError(std::string_view filename, size_t line, std::string_view msg) {
        std::cerr << filename << ":" << line << " " RED "error: " RESET;
        if(compileOpts.compileMd != compileMode::obfuscated) {
            std::cerr << msg;
        } else {
            std::cerr << randomErrorMessages.at(0); //TODO actual randomness
        }
        std::cerr << std::endl;
        compileErrors++;
    }
}
