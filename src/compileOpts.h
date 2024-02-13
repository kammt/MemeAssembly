#pragma once
#include <string_view>

enum class compileMode {
    noob, obfuscated, bully
};

enum class outputMode {
    executable, objFile, asmFile
};

enum class optLevel {
    none, o_1, o_2, o_3, o_69420, o_s
};

struct compileOpts {
    std::string_view outputFile {};
    compileMode compileMd = compileMode::noob;
    optLevel optLvl = optLevel::none;
    outputMode outputMd = outputMode::executable;
    bool martyrdom = true;
    bool debuginfo = false;
};

extern struct compileOpts compileOpts;
