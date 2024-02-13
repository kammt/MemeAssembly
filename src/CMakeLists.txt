add_executable(memeasm main.cpp parser.cpp)
target_include_directories(memeasm PUBLIC ${PROJECT_BINARY_DIR}/src)
target_compile_features(memeasm PUBLIC cxx_std_20)
