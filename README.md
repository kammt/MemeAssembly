<p align="center">
 <h1 align="center">MemeAssembly</h1>
 <p align="center">or MemeASM for short, is a highly advanced x86-Assembly based programming language using only memes as commands.</p>
</p>
  <p align="center">
    <a href="https://github.com/kammt/MemeAssembly/actions/workflows/build_test.yml">
        <img src="https://github.com/kammt/MemeAssembly/actions/workflows/build_test.yml/badge.svg" alt="Build Test">
    </a>
    <a href="https://github.com/kammt/MemeAssembly/actions/workflows/compilation_test.yml">
      <img src="https://github.com/kammt/MemeAssembly/actions/workflows/compilation_test.yml/badge.svg" alt="Compilation Test" />
   </a>
  </p>

## Commands overview
Syntax  | Usage
------------- | -------------
```stonks [value]``` | Pushes a value to the stack
``` not stonks [destination]``` | Pops a value from the stack
```confused stonks``` | Jumps to a random line of code decided at compilation time
```upgrade [other commands] fuck go back``` | Defines a marker and jumps back to that marker at a later point
```[command] or draw 25``` | The compiler will completely disregard the command before and instead add 25 to the register eax
```guess I'll die``` | Crashes the program with a segmentation fault
```bitconeeeeeeect [destination] [register/value]``` | Performs a logical and
```sneak 100 [register]``` | Sets a register to zero
```upvote [register]``` | Adds 1 to the specified register
```downvote [register]``` | Subtracts 1 from the specified register
```parry [value/register] you filthy casual [destination]``` | Subtracts the specified value from a register
```[destination] is great, but I want [register/value]``` | Move a value into a register
```ah shit, here we go again``` | Jumps back to the beginning of your code
```they're the same picture``` | A jump marker for the command below
```corporate needs you to find the difference between [register/value] and [register/value]``` | Compares both arguments and jumps to "they're the same picture" if both values are equal
```no, I don't think I will``` | Exits the program
```perfectly balanced as all things should be``` | Restore balance to your code by randomly deleting half the commands at compile-time
```what can I say except [character]``` | Prints the given character to stdout

## Code examples
Before continuing, please keep in mind that these examples only represent a tiny fraction of what can be done with MemeAssembly. Are you ready to experience the infinite possibilities of MemeAssembly? Here we go:
1. Move the number 12 to the stack, pop that number back to ebx, perform a logical and on eax and ebx and then crash with a segmentation fault:
```
stonks 12
not stonks ebx
bitconeeeeeeect eax ebx
guess I'll die
```
2. Repeatedly set the register eax to zero until the program is killed by the user:
```
upgrade
sneak 100 eax
fuck go back
```
3. Add 25 to 64:
```
ebx is great, but I want 64
sneak 100 eax

upgrade
upvote ebx
upvote eax
corporate needs you to find the difference between eax and 25
fuck go back

they're the same picture
```
4. Express your feelings during a math exam:
```
upgrade
what can I say except A
fuck go back
```

## Installation instructions
To correctly use MemeAssembly, you will further need **gcc** and **nasm**. After those are installed, clone this repository, open it in a terminal and type:
```
sudo make install
```
This will install the binary file under /usr/bin/memeasm. The compiler can now be called by using the *memeasm* command

## Compiling and running your code
First, do what a developer does best: code! Keep in mind that it is recommended to save your code in a .memeasm file. 
Once that's done, open a terminal and use one of the following commands:
- ```memeasm (-c | --compile) [-v | -vv] sourceFile destinationFile``` - only compiles the specified file
- ```memeasm [-v | -vv] sourceFile``` - compiles and runs the specified file

-v enables information logs and -vv debug logs. Furthermore, you can use *memeasm (-h | --help)* to open a help page.
    
## Why code in MemeAssembly anyway?
![Developers GIF](https://media.tenor.com/images/02fea7f02da34ea33c58a38d57fd5c5f/tenor.gif)\
Put simply, MemeAssembly brings back the good 'old days of coding. Can you even remember all the commands that your 'shiny new' programming language has to offer? Are you overwhelmed and confused by all the ways a modern programming language can be used for? Well look no further than MemeAssembly!
MemeAssembly stands out from the crowd, since it:
- only supports a tiny fraction of instructions
- only supports registers and immediates
- completely refuses to provide storage functionality
- provides a simple command to make your entire program crash
- knows that you ignore compiler warnings and hence doesn't output any
- makes your program unpredictable with random jumps and line removals. Makes debugging more fun, right?
- doesn't force you to make 'understandable' code by commenting it, since comments aren't supported anyway and would cause the compiler to crash
- removes the pain of working with negative numbers by outright not supporting them

But most important of all: **Memes**

## Badge of honor
Whoever dares to write their code in MemeASM shall be rewarded. Not only with a joyful coding experience, but also by being able to place this badge of honor in their GitHub ReadMe:\
![Made with MemeAssembly Badge](https://img.shields.io/badge/made%20with-MemeAssembly-blue)\
To do so, insert this code block into your ReadMe file: \
```![Made with MemeAssembly Badge](https://img.shields.io/badge/made%20with-MemeAssembly-blue)```

## License and Contributing
But let's get serious for a minute: \
This project is licensed under the GPLv3 license. For more information, check this repo's <a href="LICENSE">LICENSE file</a>. This obviously excludes the "Developers"-GIF in this ReadMe file. \
Contributions to this repository are welcome! Especially ideas for new (and hopefully idiotic) commands. To add a new command, either create an issue with the tag "new-command" or add it yourself and create a pull-request. Check the beginning of <a href="compiler/translator.c">translator.c</a> for a quick tutorial on how to do that.

