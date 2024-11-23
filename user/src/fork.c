#include "syscall_test.h"

void _start() {
    const char msg_parent[] = "Hello from parent process!\n";
    const char msg_child[] = "Hello from child process!\n";
    long pid = syscall_fork();  // Call fork

    if (pid == 0) {
        // Child process
        // long child_pid = getpid();  // Get child PID
        syscall_write(1, msg_child, sizeof(msg_child));    // Write to stdout (fd 1)
    } else if (pid != -1) {
        // Parent process
        // long parent_pid = getpid(); // Get parent PID
        syscall_write(1, msg_parent, sizeof(msg_parent));   // Write to stdout (fd 1)
    } else {
        // Fork failed
        syscall_exit(1);  // Exit with error
    }
    syscall_exit(0);  // Exit with success
}
