#include "syscall_test.h"

void _start() {
    const char hello[] = "Hello World in ELF!\n";

    // Write the input back to stdout (fd = 1)
    syscall_write(1, hello, sizeof(hello));

    // Exit the program with code 0
    syscall_exit(0);
}
