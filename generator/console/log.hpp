// log.hpp
#ifndef DEBUG_LOG_HPP
#define DEBUG_LOG_HPP

#include <emscripten.h>
#include <cstring>
#include <string>

extern "C" {
    // Declare the JavaScript function that will be imported into C++
    extern void log_string(int offset, int length);
}

// Helper function to log a C-style string
inline void log(const char* str) {
    log_string((int)str, std::strlen(str));
}

// Overload for std::string if needed
inline void log(const std::string& str) {
    log_string((int)str.c_str(), str.size());
}

#endif // DEBUG_LOG_HPP
