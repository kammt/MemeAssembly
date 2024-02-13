cmake_minimum_required(VERSION 3.27)

project(memeasm 
	VERSION 2.0
	LANGUAGES CXX)

add_subdirectory(src)
configure_file (
    "${PROJECT_SOURCE_DIR}/src/version.h.in"
    "${PROJECT_BINARY_DIR}/src/version.h"
)

if(CMAKE_PROJECT_NAME STREQUAL PROJECT_NAME AND BUILD_TESTING)
    add_subdirectory(tests)
endif()
