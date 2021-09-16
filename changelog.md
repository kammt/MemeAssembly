## v1.2
New features:
- New command: `let me in. LET ME IIIIIIIIN [8-bit register]`. Reads a single character from stdin into the specified register (#45, Thanks @xarantolus!)
- New command: `[register] is getting out of hand, now there are [value/register] of them`. Performs a signed multiplication
- New command: `look at what [value/register] needs to mimic a fraction of [register]`. Performs a signed division without changing the state of any registers except for the destination register

Improvements:
- MemeAssembly now has its own logo (#42)
- Compilation now does not require temporary files, but rather directly pipes the generated code to gcc (#43, Thanks @aengelke!)
- Fixed some buffer overruns during compilation (#43, Thanks @aengelke!)
- Added a new code example: toUpper (#44, Thanks @aengelke!)
- Added a new code example: is_nice

Bug fixes:
- Function analysis did not support branching functions, i.e. with multiple return statements (#41)
- Fixed crash when providing an empty file as input (#40)

[View on GitHub](https://github.com/kammt/MemeAssembly/tree/v1.2)

## v1.1.1
Bug Fixes:
- A line only containing spaces was wrongly treated as a command and translated, leading to error messages by the Assembler (#38, thanks @xarantolus!)

Other news:
- @xarantolus created an awesome VSCode extension for MemeAssembly, which can be found at https://github.com/xarantolus/MemeAssembly-vscode!
- MemeAssembly is now available to download from the AUR! It is available at https://aur.archlinux.org/packages/memeassembly/

For all other new features, check the changelog of v1.1.0

[View on GitHub](https://github.com/kammt/MemeAssembly/tree/v1.1.1)

## v1.1
New Features:
- Debugging support. When using the new `-g`-Flag, the MemeASM-compiler inserts Debug information in the stabs-Format to make it possible to debug your MemeASM code with e.g. gdb
- New command: `it's a trap` - raises SIGTRAP. If a debugger is attached, this breaks into the debugger. If no debugger is attached, the process is terminated and a core dump is generated (Thanks @RobinMarchart!)
- Reverse Storage Optimisation - When using `-O-s`, your code is "optimised" to use as much storage space as possible by aligning the end of your code to 536870912B 

Improvements:
- The `-c`-Flag no longer exists. It has been replaced by `-S` to create an Assembly-file and `-O` to create an object-file
- The StalinSort example has been updated to support Debugging

Bug fixes:
- Fixed a bug where it was not possible to use some long options such as -O-1

[View on GitHub](https://github.com/kammt/MemeAssembly/tree/v1.1)

## v1.0.1
Bug fixes:
- Fixed allocating less memory than needed for escape character translation
- Fixed allocating less memory than needed for Assembly translation, resulting in a crash

For all other new features, check the changelog of v1.0.0

[View on GitHub](https://github.com/kammt/MemeAssembly/tree/v1.0.1)

## v1.0
This major release contains the following changes:
- Added function support. It is now possible to not create an executable, but rather create a function in MemeAssembly that can then be used by other programming languages like C. To see an example of this, check out the StalinSort example.
- Since MemeAssembly is up to date on the latest technologies, it now contains a new feature before only found in Big Data and Machine Learning technologies: Pointers! Pointers revolutionise MemeASM programming by making it possible to access memory. You can make the compiler interpret a register as a pointer by using the `do you know de wey` prefix
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
    - ```wait, that's illegal``` \
        Intentionally violate the System V ABI by modifying callee-saved registers    
- Added reverse optimisation flags. These flags make your code intentionally slower and are as follows
     -  `-O-1` Inserts a nop after every command      
     -  `-O-2` Pushes and pops a register after every command
     -  `-O-3` Saves and restores an xmm register onto the Stack using *movups* after every command
- Added the optimisation flag `-O69420` which makes your code faster than lightning! \
     When using this flag, the MemeAssembly compiler reduces your run time to 0s by optimising out your entire code.
- MemeAssembly code is now translated into x86_64 Assembly
- The copiler has been rewritten. While it is most likely not faster (admittedly, it is already faster than light) it brings better compatibility with new commands in the future and has improved error handling (Rest assured: It still does not print compiler warnings). Also, it no longer depends on NASM and uses the GNU Assembler!

Things that probably no one cares about - Minor fixes:
- Getopt is now used instead of manual string parsing. Yes, I used strcmp and yes, I'm sorry.
- Input and output files are now checked, meaning you can't just provide a directory as an argument and expect the compiler to work. (At least I hope)

[View on GitHub](https://github.com/kammt/MemeAssembly/tree/v1.0)

## v0.1
Initial Release

[View on GitHub](https://github.com/kammt/MemeAssembly/tree/v0.1)