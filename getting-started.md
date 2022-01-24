## Installation
Quickly navigate to the installation instructions for [Windows](getting-started?id=windows), [MacOS](getting-started?id=macos) and [Linux](getting-started?id=linux)

### Windows 10
Running MemeAssembly under Windows requires tools like gcc to be accessible. To do this, we will first need to install mingw-w64 via MySys2.

#### Installing mingw-w64
1. Download the latest installer for MySys2 from https://www.mysys2.org
2. Use the installation instructions on the website to finish the installation. Make sure to not leave out any steps, especially not step 7
3. To be able to use mingw-w64 from the command line, we need to add it to our PATH-variable
   1. In the Windows search bar, type "Path"
   2. Select **Edit the system environment variables**
   3. Select **Environment variables**
   4. Choose the `Path` variable in your **System variables** and select **Edit**
   5. Select New and add the Mingw-w64 destination folder. If you chose the default install options, the correct directory should be `C:\msys64\mingw64\bin`
   6. Click **OK** to save

#### Installing MemeAssembly for Windows
1. Download the latest *.bat installer [here](https://github.com/kammt/MemeAssembly/releases/download/v1.3/install_windows.bat)
2. Depending on where you want to save it, consider running it with Admin priviledges 
3. The batch script will check if `gcc` and `make` are installed and downloads the latest release for you
4. To use MemeAssembly from the command line, we need to add it to our PATH-variable
   1. In the Windows search bar, type "Path"
   2. Select **Edit the system environment variables**
   3. Select **Environment variables**
   4. Choose the `Path` variable in your **System variables** and select **Edit**
   5. Select **New** and add the MemeAssembly destination folder. If you chose the default install options, the correct directory should be `C:\Program Files\MemeAssembly`
   6. Click **OK** to save

### MacOS
See [Manual Installation (Linux)](getting-started?id=manual-installation-linux)

### Linux
#### Debian-based distributions (Ubuntu, Linux Mint etc.)
Download the latest installer file (*.deb) [here](https://github.com/kammt/MemeAssembly/releases/download/v1.3/memeassembly_1.3.0_amd64.deb)

#### RPM-based distributions (Fedora, OpenSUSE, Red Hat etc.)
Download the latest installer file (*.rpm) [here](https://github.com/kammt/MemeAssembly/releases/download/v1.3/memeassembly-1.3.0.x86_64.rpm)

#### Arch-based Distributions (Arch Linux, Manjaro etc.)
1. If not already done, install a package manager that is able to access the AUR. [Yay](https://github.com/Jguer/yay) is recommended.
2. Open a terminal and type `yay -S memeassembly`

#### Manual Installation (Linux)
1. Make sure that [GCC](https://gcc.gnu.org/) is installed on your computer.
2. Download MemeAssembly. This can be done either by 
    - [downloading the latest release](https://github.com/kammt/MemeAssembly/releases/latest) 
    - or by cloning the GitHub repository using \
```git clone https://github.com/kammt/MemeAssembly.git```
3. Open a terminal and set the current directory so that you are in the root directory of the project
4. 
    - Install the compiler in `/usr/local/bin` by typing ```sudo make install```. Root priveleges are required for this.
    - Install to a custom location by modifying the DESTDIR variable. For example `sudo make DESTDIR="~/Downloads" install`
    - Simply compile the project by typing ```make all```. The executable is now in the current working directory.

## Compiler usage
- ```memeasm [options] -S -o destinationFile.S sourceFile``` - compiles the specified file into x86_64 Assembly
- ```memeasm [options] -O -o destinationFile.o sourceFile``` - compiles the specified file and saves it as an object file
- ```memeasm [options] -o destinationFile sourceFile``` - compiles the specified file and saves it as an executable 
- ```memeasm --help``` - opens a help page with this information
<br><br>

Compiler options:
- `g` - write debug info into the compiled file. Currently, only the STABS format is supported.
- `-O-1` - reverse optimisation stage 1: A nop is inserted after every command.
- `-O-2` - reverse optimisation stage 2: A register is moved to and from the stack
- `-O-3` - reverse optimisation stage 3: A xmm-register is moved to and from the stack using *movups*
- `-O-s` - reverse storage optimisation: Intentionally increases the file size by aligning end of the compiled Assembly-code to 536870912B
- `-O69420` - maximum optimisation. Reduces the execution to close to 0s by optimising out your entire code
- `-i` - enables information logs
- `-d` - enables debug logs

?> `-g` is currently only supported on Linux-systems, see [this issue](https://github.com/kammt/MemeAssembly/issues/59)

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