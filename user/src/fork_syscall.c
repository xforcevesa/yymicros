// Define syscall numbers for RISC-V
#define SYS_fork    220
#define SYS_waitpid 260
#define SYS_getpid  172
#define SYS_write   64
#define SYS_exit    93

// Syscall wrapper function
static inline long syscall(long syscall_num, long arg1, long arg2, long arg3) {
    long ret;
    asm volatile (
        "mv a7, %1\n"   // Move syscall number to a7
        "mv a0, %2\n"   // Move arg1 to a0
        "mv a1, %3\n"   // Move arg2 to a1
        "mv a2, %4\n"   // Move arg3 to a2
        "ecall\n"       // Trigger syscall with ecall
        "mv %0, a0\n"   // Move the return value from a0 to ret
        : "=r" (ret)
        : "r" (syscall_num), "r" (arg1), "r" (arg2), "r" (arg3)
        : "a0", "a1", "a2", "a7"
    );
    return ret;
}

// Write system call wrapper
static inline long write(int fd, const char *buf, int count) {
    return syscall(SYS_write, fd, (long)buf, count);
}

// Fork system call wrapper
static inline long fork() {
    return syscall(SYS_fork, 0, 0, 0);
}

// Waitpid system call wrapper
static inline long waitpid(int pid, int *wstatus, int options) {
    return syscall(SYS_waitpid, pid, (long)wstatus, options);
}

// Getpid system call wrapper
static inline long getpid() {
    return syscall(SYS_getpid, 0, 0, 0);
}

// Exit system call wrapper
static inline void exit(int status) {
    syscall(SYS_exit, status, 0, 0);
}

int main() {
    const char *msg_parent = "Hello from parent process!\n";
    const char *msg_child = "Hello from child process!\n";
    long pid = fork();  // Call fork

    if (pid == 0) {
        // Child process
        long child_pid = getpid();  // Get child PID
        write(1, msg_child, 26);    // Write to stdout (fd 1)
        exit(0);                    // Exit child process
    } else if (pid > 0) {
        // Parent process
        int wstatus;
        waitpid(pid, &wstatus, 0);  // Wait for child to exit
        long parent_pid = getpid(); // Get parent PID
        write(1, msg_parent, 27);   // Write to stdout (fd 1)
    } else {
        // Fork failed
        exit(1);  // Exit with error
    }
    return 0;
}
