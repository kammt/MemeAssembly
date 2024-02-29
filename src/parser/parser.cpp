#include "parser.h"
#include <iterator>
#include <string>
#include <string_view>
#include <fstream>
#include <iostream>
#include <optional>
#include <format>
#include "../analyser/parameters.h"
#include "../command.h"
#include "../log.h"

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

    //A basic class for iterating over a string word by word - an input iterator
    class WordIterator {
    public:
        using iterator_category = std::input_iterator_tag;

        WordIterator() = delete;
        explicit WordIterator(std::string_view strv) : str{strv} {
            if(strv != "") {
                next();
            }
        };

        static WordIterator end() {
            return WordIterator("");
        }

        WordIterator begin() {
            return *this;
        }

        WordIterator& operator++() {
            next();
            return *this;
        }

        std::string_view operator*() {
            return currentWord;
        }

        bool operator==(const WordIterator& other) const {
            return currentWord == other.currentWord;
        }

        bool operator!=(const WordIterator& other) const {
            return currentWord != other.currentWord;
        }

        bool restEquals(std::string_view reference) const {
            return str == reference;
        }

        bool restStartsWith(std::string_view reference) const {
            return str.starts_with(reference);
        }

    private:
        void next() {
            size_t wordEndIndex {0};
            for(char c : str) {
                if(c == ' ' && wordEndIndex != 0) {
                    break;
                }
                wordEndIndex++;
            }

           currentWord = str.substr(0, wordEndIndex);
            //Update our reference string_view, so that the returned word is removed
            //If we're not at the end of the string, skip the space after our word as well
            str = str.substr((str.size() > wordEndIndex) ? wordEndIndex + 1 : wordEndIndex);
        }

        std::string_view currentWord;
        std::string_view str;
    };

    /**
     * Receives a line, and returns the command type of the resulting command that was parsed
     * This is used for function parsing in the parent function
     **/
    commandType parseCommand(std::string_view& trimmedLine, std::string_view& filename, size_t lineNum) {
        //Now, iterate over all possible commands and check if they match
        std::optional<parsedCommand_t> result {};
        for(unsigned i = 0; i < commandList.size(); i++) {
            command_t command = commandList[i];

            WordIterator lineIt(trimmedLine);
            WordIterator patternIt(command.pattern);

            std::array<std::string, 2> params;
            char isPointer = 0;
            bool error {false};


            for(std::string_view lineToken : lineIt) {
                if(patternIt == patternIt.end()) {
                    if(lineIt.restEquals("or draw 25")) {
                        result = {{filename, lineNum, static_cast<unsigned>(commandList.size() - 1)}};
                    } else {
                        error = true;
                    }
                    break;
                }
                std::string_view referenceToken = *patternIt;

                size_t paramStartIndex = referenceToken.find("{}");
                if(paramStartIndex != std::string_view::npos) {
                    //Verify that all chars up until that token match
                    //The first condition is to allow for short-circuiting
                    if(paramStartIndex != 0 && referenceToken.substr(0, paramStartIndex) != lineToken.substr(0, paramStartIndex)) {
                        error = true;
                        break;
                    }

                    //Does the stuff after the parameter match?
                    std::string_view stuffAfterParam = referenceToken.substr(paramStartIndex + 2);
                    if(stuffAfterParam.size() > 0 && stuffAfterParam != lineToken.substr(lineToken.size() - stuffAfterParam.size())) {
                        error = true;
                        break;
                    }

                    //Now parse the parameter
                    unsigned paramNum = (params[0].empty()) ? 0 : 1;
                    params[paramNum] = lineToken.substr(paramStartIndex, lineToken.size() - stuffAfterParam.size());
                    if(lineIt.restStartsWith("do you know de wey")) {
                        //This is a pointer!
                        if(isPointer != 0) {
                            logger::printError(filename, lineNum, "only one parameter is allowed to be a pointer");
                        } else {
                            isPointer = paramNum + 1;
                            //Move our line iterator forward to skip "do you know de wey"
                            for(int i = 0; i < 5; i++) { ++lineIt; }
                        }
                    }
                } else if(referenceToken != lineToken) {
                    error = true;
                    break;
                }
                ++patternIt;
            }

            if(error) {
                continue;
            } else if(patternIt == WordIterator::end()) {
                //Success!
                result = {{filename, lineNum, i, params, isPointer}};
                if(command.analyser) {
                    command.analyser->commandEncountered(result.value());
                }
                break;
            }
            break;
        }

        if(!result.has_value()) {
            logger::printError(filename, lineNum, std::format("undefined command: \"{}\"", trimmedLine));
            return commandType::normal;
        } else {
          std::optional<analyser::paramArray_t> params = analyser::checkParameters(result.value());
          //Generate IR - iff the command makes sense of course
          if(params.has_value()) {
              commandList[result.value().opcode].generateIR(params.value());
          }
          //Return the command type for function parsing
          return commandList[result.value().opcode].cmdType;
        }
    }


    void parseFile(std::ifstream &input, std::string_view& filename) {
        std::string line;

        bool prevCommandWasReturn {true};
        size_t lastLineNum;
        for(size_t lineNum = 1; std::getline(input, line); lineNum++) {
            std::string_view trimmedLine {trim(line)};
            //Empty line or comment
            if(trimmedLine.size() == 0 || trimmedLine.starts_with("What the hell happened here?")) {
                continue;
            }

            commandType cmdType = parseCommand(trimmedLine, filename, lineNum);

            /*
            ** Regarding function parsing:
            ** We cannot catch every error, as the control flow is quite intransparent in an assembly-like language
            ** What we can check for however is
            **  - that the last command of a file is a return
            **  - that when we encounter a function definition, the previous command was a return
             */
            if(cmdType == commandType::funcDef && !prevCommandWasReturn) {
                logger::printError(filename, lineNum, "illegal function definition: previous command wasn't a return statement");
            }

            prevCommandWasReturn = (cmdType == commandType::ret);
            lastLineNum = lineNum;
        }

        if(!prevCommandWasReturn) {
           logger::printError(filename, lastLineNum, "file does not end with a return command");
        }

        for(command_t command : commandList) {
            if(command.analyser) {
                command.analyser->endOfFile();
            }
        }
    }
}
