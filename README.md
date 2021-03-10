# MemeAssembly
Or MemeASM for short, is a highly advanced x86-Assembly based programming language using only memes as commands.

## Commands overview
- stonks [value] - pushes a value to the stack
- not stonks [destination] - pops a value from the stack
- upgrade ... fuck go back - defines a marker and jumps back to that marker at a later point
- [comand] or draw 25 - the compiler will completely disregard the command before and instead add 25 to the register *eax*
- guess I'll die - crashes the program with a segmentation fault
- bitconeeeeeeect [destination] [register/value] - performs a logical and
- sneak 100 [register] - sets a register to zero
- upvote [register] - adds 1 to the specified register
- downvote [register] - subtracts 1 from the specified register
- [destination] is great, but I want [register/value] - move a value into a register
- perfectly balanced as all things should be - restore balance to your code by randomly deleting half the commands at compile-time

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
3. 

## Installation instructions
To correctly use MemeAssembly, you will further need **gcc** and **nasm**. After those are installed, clone this repository, open it in a terminal and type:
```
make
sudo make install
```
The compiler can now be called by using the *memeasm* command

## Compiling and running your code
First, do what a developer does best: code! Keep in mind that it is recommended to save your code in a .memeasm file. 
Once that's done, open a terminal and use one of the following commands:
- ```memeasm (-c | --compile) sourceFile destinationFile``` - only compiles the specified file
- ```memeasm sourceFile``` - compiles and runs the specified file

Furthermore, you can use *memeasm (-h | --help)* to open a help page and *memeasm (-v | --version)* to print some version information
    
## Why code in MemeAssembly, anyway?
![Developers GIF](https://media.tenor.com/images/02fea7f02da34ea33c58a38d57fd5c5f/tenor.gif)\
Put simply, MemeAssembly brings back the good 'old days of coding. Can you even remember all the commands that your 'shiny new' programming language has to offer? Are you overwhelmed and confused by all the ways a modern programming language can be used for? Well look no further than MemeAssembly!
MemeAssembly stands out from the crowd, since it:
- only supports a tiny fraction of instructions
- only supports registers and immediates
- completely refuses to provide storage functionality
- provides a simple command to make your entire program crash
- But most important of all: **Memes**

## Badge of honor
Whoever dares to write their code in MemeASM shall be rewarded. Not only with a joyful coding experience, but also by being able to place this badge of honor in their GitHub ReadMe:\
![Made with MemeAssembly Badge](https://img.shields.io/badge/made%20with-MemeAssembly-blue)\
To do so, insert this code block into your ReadMe file: \
```![Made with MemeAssembly Badge](https://img.shields.io/badge/made%20with-MemeAssembly-blue)```
