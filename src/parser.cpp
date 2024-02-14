#include "parser.h"
#include <string>
#include <string_view>
#include <fstream>
#include <iostream>
#include <optional>
#include <format>
#include "parameters.h"
#include "command.h"
#include "log.h"

namespace parser {
    /* Receives a string, and returns a string view where the leading and trailing whitespace is gone */
    std::string_view trim(std::string& str) {
        size_t start {0};
        size_t end {str.size()};

        for(char c : str) {
            if(c != ' ' && c != '\t') {
                break;
            }
            start++;
        }

        for(auto it = str.crbegin(); it != str.crend(); ++it) {
            char c = *it;
            if(c != ' ' && c != '\t') {
                break;
            }
            end--;
        }

        return {str.begin() + start, str.begin() + end};
    }

    //A basic class for iterating over a string word by word (Yes, a Java-style iterator)
    class WordIterator {
    public:
        WordIterator() = delete;
        WordIterator(std::string_view strv) : str{strv} {};

        bool hasNext() {
            return str.size() > 0;
        }

        bool restEquals(std::string_view reference) {
            return str == reference;
        }

        std::string_view next() {
            size_t wordEndIndex {0};
            for(char c : str) {
                if(c == ' ' && wordEndIndex != 0) {
                    break;
                }
                wordEndIndex++;
            }

            std::string_view result = str.substr(0, wordEndIndex);
            //Update our reference string_view, so that the returned word is removed
            //If we're not at the end of the string, skip the space after our word as well
            str = str.substr((str.size() > wordEndIndex) ? wordEndIndex + 1 : wordEndIndex);
            return result;
        }

        std::string_view str;
    };

    void parseFile(std::ifstream &input, std::string_view filename) {
        std::string line;
        for(size_t lineNum = 1; std::getline(input, line); lineNum++) {
            std::string_view trimmedLine {trim(line)};
            //Empty line or comment
            if(trimmedLine.size() == 0 || trimmedLine.starts_with("What the hell happened here?")) {
                continue;
            }

            //Now, iterate over all possible commands and check if they match
            std::optional<parsedCommand_t> result {};
            for(unsigned i = 0; i < commandList.size(); i++) {
                command_t command = commandList[i];

                WordIterator lineIt(trimmedLine);
                WordIterator patternIt(command.pattern);

                std::array<std::string_view, 2> params;
                unsigned isPointer = 0;
                bool error {false};

                while(lineIt.hasNext() && patternIt.hasNext()) {
                    std::string_view referenceToken = patternIt.next();
                    std::string_view lineToken = lineIt.next();

                    if(referenceToken == "{}") {
                        //A parameter
                        unsigned paramNum = (params[0].empty()) ? 0 : 1;
                        params[paramNum] = lineToken;
                        if(lineIt.str.starts_with("do you know de wey")) {
                            //This is a pointer!
                            if(isPointer != 0) {
                                logger::printError(filename, lineNum, "only one parameter is allowed to be a pointer");
                            } else {
                                isPointer = paramNum + 1;
                                //Move our line iterator forward to skip "do you know de wey"
                                lineIt.next(); lineIt.next(); lineIt.next(); lineIt.next(); lineIt.next();
                            }
                        }
                    } else if(referenceToken != lineToken) {
                        error = true;
                        break;
                    }
                }

                if(error) {
                    continue;
                } else if(!lineIt.hasNext() && !patternIt.hasNext()) {
                    //Success!
                    result = {{filename, lineNum, i, std::string {params[0]}}};
                    if(command.analyser) {
                        command.analyser->commandEncountered(result.value());
                    }
                } else if(!patternIt.hasNext() && lineIt.hasNext()) {
                    if(lineIt.restEquals("or draw 25")) {
                        result = {{filename, lineNum, static_cast<unsigned>(commandList.size() - 1)}};
                    }
                }

            }

            if(!result.has_value()) {
                logger::printError(filename, lineNum, std::format("undefined command: \"{}\"", trimmedLine));
            } else {
                std::cout << "Success!\n";
                //TODO call generateIR() on command_t, and use isPointer in some way
            }
        }
        for(command_t command : commandList) {
            if(command.analyser) {
                command.analyser->endOfFile();
            }
        }
    }
}
