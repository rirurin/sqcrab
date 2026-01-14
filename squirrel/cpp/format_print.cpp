#include <cstdint>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

extern "C" {
    void sq_print_callback_rust(uintptr_t vm, const char* str);
    void sq_error_callback_rust(uintptr_t vm, const char* str);
}

extern "C" void sq_print_callback_cpp(uintptr_t vm, const char* fmt, ...) {
    va_list args;
    va_start(args, fmt);
    #if defined( _MSC_VER )
    int length = vsprintf_s(nullptr, 0, fmt, args) + 1;
    #else
    int length = vsprintf(nullptr, fmt, args) + 1;
    #endif
    char* buf = (char*)malloc(length);
    va_start(args, fmt);
    #if defined( _MSC_VER )
    vsprintf_s(buf, length, fmt, args);
    #else
    vsprintf(buf, fmt, args);
    #endif
    va_end(args);
    sq_print_callback_rust(vm, buf);
    free(buf);
}

extern "C" void sq_error_callback_cpp(uintptr_t vm, const char* fmt, ...) {
    va_list args;
    va_start(args, fmt);
    #if defined( _MSC_VER )
    int length = vsprintf_s(nullptr, 0, fmt, args) + 1;
    #else
    int length = vsprintf(nullptr, fmt, args) + 1;
    #endif
    char* buf = (char*)malloc(length);
    va_start(args, fmt);
    #if defined( _MSC_VER )
    vsprintf_s(buf, length, fmt, args);
    #else
    vsprintf(buf, fmt, args);
    #endif
    va_end(args);
    sq_print_callback_rust(vm, buf);
    free(buf);
}