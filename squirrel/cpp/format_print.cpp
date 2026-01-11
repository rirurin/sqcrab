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
    int length = vsprintf_s(nullptr, 0, fmt, args) + 1;
    char* buf = (char*)malloc(length);
    va_start(args, fmt);
    vsprintf_s(buf, length, fmt, args);
    va_end(args);
    sq_print_callback_rust(vm, buf);
    free(buf);
}

extern "C" void sq_error_callback_cpp(uintptr_t vm, const char* fmt, ...) {
    va_list args;
    va_start(args, fmt);
    int length = vsprintf_s(nullptr, 0, fmt, args) + 1;
    char* buf = (char*)malloc(length);
    va_start(args, fmt);
    vsprintf_s(buf, length, fmt, args);
    va_end(args);
    sq_print_callback_rust(vm, buf);
    free(buf);
}