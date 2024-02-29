#include "irCommand.h"
#include <fstream>
#include <iostream>

void LabelDefCommand::transpile(std::ofstream& out) {
    out << labelName << ":" << std::endl;
}

void CallCommand::transpile(std::ofstream& out) {
   out << "\tcall " << labelName << std::endl;
}

void RetCommand::transpile(std::ofstream& out) {
    if(val.has_value()) {
        out << "\tmov rax, " << val.value() << std::endl;
    }
    out << "\tret" << std::endl;
}
