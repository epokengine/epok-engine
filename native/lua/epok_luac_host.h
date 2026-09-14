#pragma once

/*
 * psxlua intentionally spells a few operations with GCC/Clang builtin names
 * for the freestanding PlayStation target. The editor's bytecode cooker builds
 * the same sources as ordinary host C. MSVC accepts those names as undeclared
 * external functions, which only fails later at link time, so map them to the
 * equivalent CRT operations for the native host build.
 */
#if defined(_MSC_VER) && !defined(__clang__)
#include <stdlib.h>
#include <string.h>

#define __builtin_memcpy memcpy
#define __builtin_unreachable abort
#endif
