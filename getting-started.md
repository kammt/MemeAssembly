## Installation
Quickly navigate to the installation instructions for [Windows](getting-started?id=windows), [MacOS](getting-started?id=macos) and [Linux](getting-started?id=linux)

### Windows 10
Fasten your seatbelts, because this is gonna be cursed. Running MemeAssembly under Windows requires tools like gcc to be accessible. To do this, we will first need to install mingw-w64 via MySys2.

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
See [Manual Installation (Linux)](getting-started?id=manual-installation-linux).

### Linux
#### Debian-based distributions (Ubuntu, Linux Mint etc.)
Download the latest installer file (*.deb) [here](https://github.com/kammt/MemeAssembly/releases/latest)

#### RPM-based distributions (Fedora, OpenSUSE, Red Hat etc.)
Download the latest installer file (*.rpm) [here](https://github.com/kammt/MemeAssembly/releases/latest) or install it from COPR: (`sudo dnf copr enable kammt/memeasm && sudo dnf install memeasm`).

#### Arch-based Distributions (Arch Linux, Manjaro etc.)
1. If not already done, install a package manager that is able to access the AUR. [Yay](https://github.com/Jguer/yay) is recommended.
2. Open a terminal and type `yay -S memeassembly`

#### Manual Installation (Linux)
1. Make sure that [GCC](https://gcc.gnu.org/) is installed.
2. Download MemeAssembly. This can be done either by 
    - [downloading the latest release](https://github.com/kammt/MemeAssembly/releases/latest) 
    - or by cloning the GitHub repository using \
```git clone https://github.com/kammt/MemeAssembly.git```
3. Open a terminal and set the current directory so that you are in the root directory of the project
4. 
    - Install the compiler in `/usr/local/bin` by running ```sudo make install```. Root privileges are required for this.
    - Install to a custom location by modifying the DESTDIR variable. For example: `sudo make DESTDIR="~/Downloads" install`
    - Simply compile the project by runnning ```make```. The executable is now in the current working directory.

## Compiler usage
- ```memeasm [options] -S -o destinationFile.S sourceFile``` - compiles the specified file into x86_64 Assembly
- ```memeasm [options] -O -o destinationFile.o sourceFile``` - compiles the specified file and saves it as an object file
- ```memeasm [options] -o destinationFile sourceFile``` - compiles the specified file and saves it as an executable 
- ```memeasm --help``` - opens a help page with this information
<br><br>

Let's take a look at some compiler features:

### Reverse optimisation options
The `p` in MemeAssembly stands for performance, so MemeAssembly naturally includes some optimisation flags for you. Well, at least not the ones you'd expect. Here's an overview:
- `-O-1` - reverse optimisation stage 1: A nop is inserted after every command
- `-O-2` - reverse optimisation stage 2: A register is moved to and from the stack after every command
- `-O-3` - reverse optimisation stage 3: A xmm-register is moved to and from the stack using *movups* after every command
- `-O-s` - reverse storage optimisation: Intentionally increases the file size by aligning the end of the compiled Assembly-code to 536870912B, creating a 1.6GB binary
- `-O69420` - maximum optimisation. Reduces the execution to close to 0s by optimising out your entire code

If multiple optimisation options are specified, only the last option is taken into account. This means that you *cannot* combine e.g. `-O-3` and `-O-s` to create a fat and slow binary.

### Martyrdom
An alternative title for this section would be "Why does my shell close when I Ctrl+C a MemeASM program?".
Well, this is because of [Martyrdom](https://knowyourmeme.com/memes/martyrdom-drop-a-live-grenade-when-killed). By default, MemeASM programs drop a live grenade when killed. This means that before executing your own code, a signal handler is created for SIGINT, in which SIGKILL is sent to the parent process. If you don't want this to happen, remember to compile your program with `-fno-martyrdom`.

?> Protip: It'd be beneficial for your system if `init` is not the parent process of a MemeAssembly program. Just saying.

### Compilation modes
MemeAssembly has three compilation modes, which you can set with the option `-fcompile-mode`:
1. **Noob mode** (`-fcompile-mode=noob`): This is the default
2. **Obfuscated Mode** (`-fcompile-mode=obfuscated`): Every error message is replaced with something else. It might be an actual error the compiler would print, it might be buzzword bingo, it might even be a `sudo` error message. What will you get?
3.  **Bully mode** (`-fcompile-mode=bully`): This is where it gets fun. In bully mode, all compiler errors are silenced. Instead, the program is modified by the compiler to fix the error. This could include replacing parameters, removing code, adding new code or just replacing the broken command with a randomly generated one. Isn't this beautiful? \
Now by random, I don't mean truly random. Every change done in bully mode by the compiler is either pre-defined or dependent on the input file (e.g. file name, the ASCII codes of a faulty line of code). This means that broken code in bully mode will _always_ compile to the same "fixed" code. This creates a sort of "meta programming language". You can now create bullshit code which normally does not compile, but has a defined behavior<a href="#/memeasm-standard">*</a> in bully mode.

### Debugging a MemeASM program
`-g` exists, but is only supported on Linux-systems. And even there, it is kind of broken. If you get it to work, good for you.
The recommended debugger is `gdb`.

## MemeAssembly Basics
### Which IDE should I use?
The recommended IDE is VSCode. Why? Because it has an awesome MemeAssembly Extension, which enables suggestions and code highlighting. MemeAssembly does not implement lsp yet, but this is a planned feature with no ETA.

### Supported parameters 
Before we start looking at MemeAssemnly commands, let's take a look at possible parameters first:
- Numbers \
Immediate numbers are supported. However, keep in mind that only the `long long` type (i.e. signed 64 Bit numbers) is supported. Also, the number must be provided in the decimal notation. Binary, hex, octal etc. are unsupported.
- Registers \
All General purpose registers can be provided
- Pointers \
Whenever a register variable is supported, one can also make it a Memory access by appending `do you know de wey` to the register name. \
Example: `rax is brilliant, but I like rdi do you know de wey` loads the 8B-value from the address that is in `rdi`, and saves it in rax. This would be equivalent to `mov rax, [rdi]` in x86-64 (with Intel syntax).
- Characters: \
ASCII characters can also be provided. Contrary to other programming languages, no quotation marks are required for this. Escape sequences are also supported, here is an overview: 
    Escape Sequence  | Usage
    ------------- | -------------
    ```\n``` | New line
    ```\s``` / ```space``` | Space
    ```\b``` | Backspace
    ```\t``` | Horizontal Tab
    ```\v``` | Vertical Tab
    ```\f``` | Formfeed Page Break
    ```\"``` | Double quotation mark
    ```\?``` | Question mark
    ```\\``` | Backslash

    Keep in mind that the numbers 0-9 are interpreted as characters, and NOT their ASCII value.
    Printing Unicode-Characters directly is not possible. However, using [this program](https://play.golang.org/p/TojzlTMIcJe), you can get the sequence of commands needed to print your character.
- Monke label name \
See the description of the Monke-command for more information
- Function name \
Well, yeah.

### Hello World
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
