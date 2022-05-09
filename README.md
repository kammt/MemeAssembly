This is a special version of the MemeASM compiler to be used in the lab course [GRA](https://www.in.tum.de/caps/lehre/ss22/praktika/gra/) - it allows the compiler to create executables that can be run within [raspsim](https://github.com/aengelke/raspsim). This includes
- throwing compiler errors when I/O-commands are used
- allowing for a special (sandboxed) gcc to be used
- passing compiler options to gcc to create an executable to be run in a freestanding environment
