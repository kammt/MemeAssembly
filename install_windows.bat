@echo off
ECHO Checking for correct installation of mingw64

WHERE gcc
IF %ERRORLEVEL% == 0 goto :checkMake
ECHO Error! Could not find gcc in your PATH. Check https://kammt.github.io/MemeAssembly/#/getting-started for more information
ECHO Installation failed
PAUSE
EXIT 1

:checkMake
WHERE mingw32-make
IF %ERRORLEVEL% == 0 goto :downloadMemeasm
ECHO Error! Could not find mingw32-make in your PATH. Check https://kammt.github.io/MemeAssembly/#/getting-started for more information
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

:runMakefile
set /p installDir="Enter the desired installation directory. Make sure that you have the required priviledges (Default: C:\Program Files\MemeAssembly): "
IF NOT DEFINED installDir SET "installDir=C:\Program Files\MemeAssembly"
ECHO Compiling...
mingw32-make
ECHO Installing...
mkdir "%installDir%"
move "./memeasm.exe" "%installDir%\memeasm.exe"
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
