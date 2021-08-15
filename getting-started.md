## Installation
1. Make sure that [GCC](https://gcc.gnu.org/) is installed on your computer.
2. Download MemeAssembly. This can be done either by 
    - [downloading the latest release](https://github.com/kammt/MemeAssembly/releases/latest) 
    - or by cloning the GitHub repository using \
```git clone https://github.com/kammt/MemeAssembly.git```
3. Open a terminal and set the current directory so that you are in the root directory of the project
4. 
    - Install the compiler in */usr/bin* by typing ```sudo make install```. Root priveleges are required for this.
    - Simply compile the project by typing ```make all```. The executable is now in the current working directory.

## Compiler usage
- ```memeasm [options] (-c | --compile) -o destinationFile.S sourceFile``` - compiles the specified file into x86_64 Assembly
- ```memeasm [options] -o destinationFile sourceFile``` - compiles the specified file and saves it as an executable 
- ```memeasm --help``` - opens a help page with this information
<br><br>

Compiler options:
- `-O-1` - reverse optimisation stage 1: A nop is inserted after every command.
- `-O-2` - reverse optimisation stage 2: A register is moved to and from the stack
- `-O-3` - reverse optimisation stage 3: A xmm-register is moved to and from the stack using *movups*
- `-O69420` - maximum optimisation. Reduces the execution to close to 0s by optimising out your entire code
- `-i` - enables information logs
- `-d` - enables debug logs

## Hello World
So let's create our first program! 
1. Create a new file and name it "HelloWorld.memeasm"
2. Now open it using a text editor of your choice and paste the following code into it:
    ```
    I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun main
        what can I say except H
        what can I say except e
        what can I say except l
        what can I say except l
        what can I say except o
        what can I say except \s
        what can I say except W
        what can I say except o
        what can I say except r
        what can I say except l
        what can I say except d
        what can I say except !
        what can I say except \n
        
        I see this as an absolute win
    ```
3. Open a terminal in the directory of your file and type ```memeasm -o helloWorld HelloWorld.memeasm```
4. Run the program using ```./helloWorld```
5. Voilà! The output of your program is "Hello World!"