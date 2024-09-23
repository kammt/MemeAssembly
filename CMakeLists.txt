cmake_minimum_required(VERSION 3.20)
project(memeassembly LANGUAGES C)

set(FILES src/memeasm.c src/compiler.c src/logger/log.c src/parser/parser.c src/parser/fileParser.c src/parser/functionParser.c src/analyser/analysisHelper.c src/analyser/parameters.c src/analyser/functions.c src/analyser/jumpMarkers.c src/analyser/comparisons.c src/analyser/randomCommands.c src/analyser/analyser.c src/translator/translator.c)

if(APPLE)
    set(PLATFORM_MACRO "MACOS")
elseif(UNIX)
    set(PLATFORM_MACRO "Linux")
else()
    MESSAGE(FATAL_ERROR "Unsupported Platform: MemeAssembly is only supported on Linux/MacOS.")
endif()
add_definitions(-${PLATFORM_MACRO})

# Add other build commands
execute_process(COMMAND pwd OUTPUT_VARIABLE PWD OUTPUT_STRIP_TRAILING_WHITESPACE)
message(${PWD})
execute_process(COMMAND id -u OUTPUT_VARIABLE UID OUTPUT_STRIP_TRAILING_WHITESPACE)
execute_process(COMMAND id -g OUTPUT_VARIABLE GID OUTPUT_STRIP_TRAILING_WHITESPACE)
# build memeasm for WASM
add_custom_target(wasm
    docker run --rm -v ${CMAKE_SOURCE_DIR}:/src -w /src -u ${UID}:${GID} emscripten/emsdk emcc -o memeasm.js -s WASM=1 -s INVOKE_RUN=0 -s EXPORT_ES6=1 -s MODULARIZE=1 -s EXPORTED_RUNTIME_METHODS="['FS', 'callMain', 'cwrap']" -s 'EXPORT_NAME="runMemeAssemblyCompiler"' ${FILES} -std=gnu17 -D ${PLATFORM_MACRO} -O2
)
# formats the project
add_custom_target(format
    rm -frfr ${PROJECT_SOURCE_DIR}          
)

# Add Fadec
include(ExternalProject)
set(FADEC_RES ${CMAKE_BINARY_DIR}/fadec-prefix/src/fadec/build/libfadec.a)
ExternalProject_Add(fadec
    GIT_REPOSITORY    https://github.com/aengelke/fadec.git
    GIT_TAG           ee111375d99d98538fff448cc81c355911562ef2
    CONFIGURE_COMMAND ""
    BUILD_COMMAND     ""
    INSTALL_COMMAND   ""
    BUILD_BYPRODUCTS  ${FADEC_RES}

    COMMAND ${CMAKE_COMMAND} -E echo "Configuring fadec using Meson"
    COMMAND meson setup ${CMAKE_BINARY_DIR}/fadec-prefix/src/fadec/build ${CMAKE_BINARY_DIR}/fadec-prefix/src/fadec --buildtype=release

    COMMAND ${CMAKE_COMMAND} -E echo "Building fadec"
    COMMAND meson compile -C ${CMAKE_BINARY_DIR}/fadec-prefix/src/fadec/build
)

set(CMAKE_C_STANDARD 17)
set(CMAKE_C_STANDARD_REQUIRED TRUE)

add_executable(memeasm ${FILES})
target_link_libraries(memeasm ${FADEC_RES})

install(TARGETS memeasm DESTINATION /usr/bin)