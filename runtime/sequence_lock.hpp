#pragma once
// Preserve the caller's interrupt state, including nested calls and IRQ context.
#ifndef EPOK_SEQUENCE_HOST_TEST
#include "psyqo/kernel.hh"
#endif
namespace epok {
#ifdef EPOK_SEQUENCE_HOST_TEST
inline unsigned sequence_lock_depth=0;
#endif
struct SequenceLock {
#ifndef EPOK_SEQUENCE_HOST_TEST
    uint32_t previous;
    SequenceLock() : previous(psyqo::Kernel::Internal::getCop0Status()) {
        asm volatile("" ::: "memory");
        psyqo::Kernel::Internal::setCop0Status(previous & ~1u);
        asm volatile("" ::: "memory");
    }
    ~SequenceLock() {
        asm volatile("" ::: "memory");
        psyqo::Kernel::Internal::setCop0Status(previous);
        asm volatile("" ::: "memory");
    }
#else
    SequenceLock(){++sequence_lock_depth;}
    ~SequenceLock(){--sequence_lock_depth;}
#endif
};
}
