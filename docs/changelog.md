## v1.0
This major release contains the following changes:
- Added function support. It is now possible to not create an executable, but rather create a function in MemeAssembly that can then be used by other programming languages like C.
- Since MemeAssembly is up to date on the latest technologies, it now contains a new feature before only found in Big Data and Machine Learning technologies: Pointers! Pointers.. point to stuff (duh). You can make the compiler interpret a register as a pointer by using the "do you know de wey prefix"
- Implemented some shiny new commands, which include but are not limited to:
    - ```I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun [function name]``` \
        Defines a new function. If an executable is to be created, a main-function must exist
    - ```right back at ya buckaroo``` \
         Return from a function
    - ```[register] units are ready with [value/register] more well on the way``` \
         Adds a value to the specified register
    - ```monke [monke jump label]``` \
         Defines a new jump label. Monke jump labels must only contain the letters 'a' and 'u', of which both must appear at least once
    - ```return to monke [monke jump label]``` \
         Jump to a monke jump label
    - ```who would win? [value/register] or [value/register]``` \
        Compares the two parameters provided and jumps to a label defined by using ```[value/register] wins```
- MemeAssembly code is now translated into x86_64 Assembly
- The copiler has been rewritten. While it is most likely not faster (admittedly, it is already faster than light) it brings better compatibility with new commands in the future and has improved error handling (Rest assured: It still does not print compiler warnings). Also, it no longer depends on NASM!

Things that probably no one cares about - Minor fixes:
- Getopt is now used instead of manual string parsing. Yes, I used strcmp and yes, I'm sorry.
- Input and output files are now checked, meaning you can't just provide a directory as an argument and expect the compiler to work. (At least I hope)

[View on GitHub](https://github.com/kammt/MemeAssembly/tree/v1.0)

## v0.1
Initial Release