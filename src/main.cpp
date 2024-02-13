#include <cstring>
#include <fstream>
#include <getopt.h>
#include <iostream>

#include "command.h"
#include "version.h"
#include "compileOpts.h"
#include "parser.h"

//Compiler options stored as a global variable
struct compileOpts compileOpts;

int main(int argc, char** argv) {
    int martyrdom = true;
    int fortify = false;
    const struct option long_options[] = {
            {"output", required_argument, nullptr, 'o'},
            {"help", no_argument, nullptr, 'h'},
            {"fno-martyrdom", no_argument, &martyrdom, false}, //TODO actually use this
            {"D_FORTIFY_SOURCE", optional_argument, &fortify, true}, //TODO actually use this
            {"fcompile-mode", required_argument, nullptr, 'c'},
            { nullptr, 0, nullptr, 0 }
    };

    int opt;
    int option_index = 0;

    while ((opt = getopt_long_only(argc, argv, "o:hO::Sv", long_options, &option_index)) != -1) {
        switch (opt) {
            case 'h':
                return 0;
            case 'v':
                std::cout << MEMEASM_VERSION << std::endl;
                return 0;
            case 'S':
                compileOpts.outputMd = outputMode::asmFile;
                break;
            case 'O':
                if(!optarg) {
                    compileOpts.outputMd = outputMode::objFile;
                } else {
                    std::string_view opt {optarg};
                    if(opt == "-s") {
                        compileOpts.optLvl = optLevel::o_s;
                    } else if(opt == "-1") {
                        compileOpts.optLvl = optLevel::o_1;
                    } else if(opt == "-2") {
                        compileOpts.optLvl = optLevel::o_2;
                    } else if(opt == "-3") {
                        compileOpts.optLvl = optLevel::o_3;
                    } else if(opt == "69420") {
                        compileOpts.optLvl = optLevel::o_69420;
                    } else {
                        std::cerr << "Error: Unknown optimisation level provided (must be one of \"-1\", \"-2\", \"-3\", \"69420\", \"-s\")" << std::endl;
                        return 1;
                    }
                }
                break;
            case 'o':
                compileOpts.outputFile = {optarg};
                break;
            case 'c': { //-fcompile-mode
                std::string_view mode {optarg};
                if(mode == "bully") {
                    compileOpts.compileMd = compileMode::bully;
                } else if(mode == "obfuscated") {
                    compileOpts.compileMd = compileMode::obfuscated;
                } else if(mode == "noob") {
                    compileOpts.compileMd = compileMode::noob;
                } else {
                    std::cerr << "Error: invalid compile mode (must be one of \"noob\", \"bully\", \"obfuscated\")" << std::endl;
                    return 1;
                }
                break;
            }
            case '?':
                std::cerr << "Error: Unknown option provided" << std::endl;
                return 1;
        }
    }
    compileOpts.martyrdom = martyrdom;
    compileOpts.fortify = fortify;

    if(compileOpts.outputFile.empty()) {
        std::cerr << "Error: no output file specified" << std::endl;
        return 1;
    } else if(argc < optind + 1) {
        std::cerr << "Error: no input file(s) specified" << std::endl;
        return 1;
    } else {
        //We have one or more input files, check how many there are
        //The first is at optind, the last at argc-1
        uint32_t fileCount = argc - optind;

        for(unsigned i = optind; i < argc; i++) {
            std::string_view filename {argv[i]};

            std::ifstream file {argv[i]};
            if(!file.is_open()) {
                std::cerr << "Error: unable to open " << filename << ": " << strerror(errno) << std::endl;
                return 1;
            }

            if(!std::filesystem::is_regular_file(filename)) {
                std::cerr << "Error: provided input file " << filename << " is not a regular file" << std::endl;
            }

            //Start with file parsing
            parser::parseFile(file, filename);
        }

        for(command_t command : commandList) {
            if(command.analyser) {
                command.analyser->analysisEnd();
            }
        }
    }
}
