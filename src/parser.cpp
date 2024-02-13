#include "parser.h"
#include <string>
#include <string_view>
#include <fstream>
#include <iostream>

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

        std::string_view next() {
            size_t wordEndIndex {0};
            for(char c : str) {
                if(c == ' ' && wordEndIndex != 0) {
                    break;
                }
                wordEndIndex++;
            }

            std::string_view result = str.substr(0, wordEndIndex);
            //If we're not at the end of the string, skip the space after our word
            str = str.substr((str.size() > wordEndIndex) ? wordEndIndex + 1 : wordEndIndex);
            return result;
        }
    private:
        std::string_view str;
    };

    void parseFile(std::ifstream &input) {
        std::string line;
        while(std::getline(input, line)) {
            std::string_view trimmed {trim(line)};
            std::cout << trimmed << std::endl;
            for(auto it = WordIterator(trimmed); it.hasNext();) {
                std::cout << "Word: \"" << it.next() << "\"\n";
            }
        }
    }
}
