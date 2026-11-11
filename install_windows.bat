@echo off
ECHO Checking for correct installation of Rust and mingw64

WHERE cargo
IF %ERRORLEVEL% == 0 goto :checkRustup
ECHO Error! Could not find cargo in your PATH. Install Rust using rustup (https://rustup.rs). Check https://kammt.github.io/MemeAssembly/#/getting-started for more information
ECHO Installation failed
PAUSE
EXIT 1

:checkRustup
WHERE rustup
IF %ERRORLEVEL% == 0 goto :checkGcc
ECHO Error! Could not find rustup in your PATH. Install Rust using rustup (https://rustup.rs). Check https://kammt.github.io/MemeAssembly/#/getting-started for more information
ECHO Installation failed
PAUSE
EXIT 1

:checkGcc
REM gcc (mingw64) compiles the C code that MemeAssembly includes and links the compiler
WHERE gcc
IF %ERRORLEVEL% == 0 goto :downloadMemeasm
ECHO Error! Could not find gcc in your PATH. Check https://kammt.github.io/MemeAssembly/#/getting-started for more information
ECHO Installation failed
PAUSE
EXIT 1

:downloadMemeasm
ECHO All dependencies satisfied
ECHO Downloading latest release of MemeAssembly...
powershell -Command "(New-Object Net.WebClient).DownloadFile('https://github.com/kammt/MemeAssembly/zipball/master', '%temp%\MemeAssembly.zip')
ECHO Extracting...
powershell Expand-Archive %temp%\MemeAssembly.zip -DestinationPath %temp%\MemeAssembly-latest
cd %temp%\MemeAssembly-latest\kammt-*

:runCargo
set /p installDir="Enter the desired installation directory. Make sure that you have the required priviledges (Default: C:\Program Files\MemeAssembly): "
IF NOT DEFINED installDir SET "installDir=C:\Program Files\MemeAssembly"
ECHO Installing the Rust toolchain for mingw64 (if necessary)...
REM MemeAssembly cannot be built with the MSVC toolchain, so the GNU (mingw64) toolchain is used
rustup toolchain install stable-x86_64-pc-windows-gnu --profile minimal
IF %ERRORLEVEL% NEQ 0 goto :buildFailed
ECHO Compiling...
cargo +stable-x86_64-pc-windows-gnu build --release
IF %ERRORLEVEL% NEQ 0 goto :buildFailed
ECHO Installing...
mkdir "%installDir%"
copy /Y "target\release\memeasm.exe" "%installDir%\memeasm.exe"
icacls  "%installDir%\memeasm.exe"  /t  /grant  everyone:R

:cleanup
ECHO Cleaning up...
cd ../..
DEL /S /Q MemeAssembly-latest
DEL /F MemeAssembly.zip

:endofinstall
ECHO Installation done! To use MemeAssembly from the command line, add the directory %installDir% to your system-wide PATH-Variable
PAUSE
EXIT 0

:buildFailed
ECHO Error! Compiling MemeAssembly failed
ECHO Installation failed
PAUSE
EXIT 1
