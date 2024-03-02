#include "parameters.h"
#include "../command.h"
#include "../log.h"
#include <exception>
#include <stdexcept>
#include <format>

namespace analyser {
  //Maps all escape sequences to their char-value
  const std::unordered_map<std::string, char> escapeSeqMap = {
    {"\\n", '\n'}, {"\\s", ' '}, {"space", ' '}, {"\\t", '\t'}, {"\\f", '\f'}, {"\\b", '\b'}, {"\\v", '\v'}, {"\\\\", '\\'}, {"\\?", '\?'}
  };



  std::optional<paramArray_t> checkParameters(parser::parsedCommand_t &cmd) {
    auto forbidPointer = [&cmd](unsigned i) {
      if(cmd.isPointer == i + 1) {
        logger::printError(cmd.filename, cmd.line, std::format("only registers can be pointers (found: \"{}\")", cmd.params[i]));
      }
    };

    auto allowedParams = commandList[cmd.opcode].allowedParams;
    std::array<parameter_t, 2> result {};
    bool failed = false;

    for (int i = 0; i < 2; i++) {
        if (allowedParams[i] == NO_PARAM) {
          break;
        } else {
          //Check if it is a register
          auto regLookup = regMap.find(cmd.params[i]);
          if(regLookup != regMap.end()) {
            reg reg = regLookup->second;
            unsigned regSize = getRegSize(reg);
            if(((allowedParams[i] & PARAM_REG8) != 0 && regSize == 8) || ((allowedParams[i] & PARAM_REG16) != 0 && regSize == 16) || ((allowedParams[i] & PARAM_REG32) != 0 && regSize == 32) || ((allowedParams[i] & PARAM_REG64) != 0 && regSize == 64)) {
              //Set param type to register
              result[i] = std::make_pair(reg, (cmd.isPointer == i + 1));
              continue;
            }
          }

          //Not a register. A decimal number?
          if(allowedParams[i] & PARAM_DECIMAL) {
            int64_t number;
            bool success {false};
            try {
              number = std::stoll(cmd.params[i], nullptr, 10);
              success = true;
            } catch(std::out_of_range) {
              logger::printError(cmd.filename, cmd.line, std::format("parameter {} ({}) is not a valid number: does not fit into 64 bits", i, cmd.params[i]));
              continue;
            } catch(std::invalid_argument) {
              //It's probably just not a number - continue
            }

            if(success) {
              forbidPointer(i);
              result[i] = number;
              continue;
            }
          }

          //Characters (including escape sequences) / ASCII-code
          if(allowedParams[i] & PARAM_CHAR) {
            //An escape sequence?
            auto escapeSeq = escapeSeqMap.find(cmd.params[i]);
            if(escapeSeq != escapeSeqMap.end()) {
              result[i] = escapeSeq->second;
              forbidPointer(i);
              continue;
            }

            //A single character?
            if(cmd.params[i].size() == 1) {
              result[i] = cmd.params[i][0];
              forbidPointer(i);
              continue;
            }

            //An ASCII-code?
            int64_t number {-1};
            try {
              number = std::stoll(cmd.params[i], nullptr, 10);
            } catch(std::exception) {}

            //We allow values greater than 128 so that it is possible to print unicode in multiple steps. See e.g. https://play.golang.org/p/TojzlTMIcJe
            if(number >= 0 && number <= 255) {
              result[i] = static_cast<char>(number);
              forbidPointer(i);
              continue;
            }
          }

          //A monke Label
          if(allowedParams[i] & PARAM_MONKE_LABEL) {
            bool a_used = false;
            bool u_used = false;
            bool invalid = false;

            for(char c : cmd.params[i]) {
              if(c == 'a') {
                a_used = true;
              } else if(c == 'u') {
                u_used = true;
              } else {
                //Invalid character
                invalid = true;
                break;
              }
            }

            if(a_used && u_used && !invalid) {
              result[i] = std::move(cmd.params[i]);
              forbidPointer(i);
              continue;
            }
          }

          //A function name
          if(allowedParams[i] & PARAM_FUNC_NAME) {
            bool first = true;
            bool invalid = false;
            for(char c : cmd.params[i]) {
              if(first) {
                first = false;
                //May not start with a number
                if(c >= '0' && c <= '9') {
                  invalid = true;
                }
              }
              if(!(c == '_' || c == '$' || c == '.' || (c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z'))) {
                invalid = true;
              }
            }

            if(!invalid) {
              result[i] = std::move(cmd.params[i]);
              forbidPointer(i);
              continue;
            }
          }

          //We got here, meaning that the parameter is invalid
          logger::printError(cmd.filename, cmd.line, std::format("provided parameter \"{}\" is invalid", cmd.params[i]));
          //Don't return directly to ensure that *all* errors are printed
          failed = true;
        }
      }

      if(failed) {
        return {};
      } else {
        return {result};
      }
   }
}
