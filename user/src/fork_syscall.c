// Define system call numbers for RISC-V
#define SYSCALL_FORK 220
#define SYSCALL_WRITE 64
#define SYSCALL_EXIT 93

// Inline assembly function for `fork` syscall
static inline long syscall_fork() {
    long ret;
    asm volatile (
        "mv a7, %[syscall_num]\n"   // Move syscall number to a7
        "ecall\n"                   // Make the syscall
        "mv %[ret], a0\n"           // Store return value (PID or 0 for child) in ret
        : [ret] "=r" (ret)
        : [syscall_num] "r" (SYSCALL_FORK)
        : "a0", "a7"
    );
    return ret;
}

// Inline assembly function for `write` syscall
static inline long syscall_write(int fd, const char *buf, long count) {
    long ret;
    asm volatile (
        "mv a7, %[syscall_num]\n"   // Move syscall number to a7
        "mv a0, %[fd]\n"            // Move file descriptor to a0 (stdout = 1)
        "mv a1, %[buf]\n"           // Move buffer address to a1
        "mv a2, %[count]\n"         // Move number of bytes to write to a2
        "ecall\n"                   // Make the syscall
        "mv %[ret], a0\n"           // Store return value (number of bytes written) in ret
        : [ret] "=r" (ret)
        : [syscall_num] "r" (SYSCALL_WRITE), [fd] "r" (fd), [buf] "r" (buf), [count] "r" (count)
        : "a0", "a1", "a2", "a7"
    );
    return ret;
}

// Inline assembly function for `exit` syscall
static inline void syscall_exit(int exit_code) {
    asm volatile (
        "mv a7, %[syscall_num]\n"   // Move syscall number to a7
        "mv a0, %[exit_code]\n"     // Move exit code to a0
        "ecall\n"                   // Make the syscall
        :
        : [syscall_num] "r" (SYSCALL_EXIT), [exit_code] "r" (exit_code)
        : "a0", "a7"
    );
}

int main() {
    long pid = syscall_fork();

    // Buffer for messages
    const char parent_msg[] = "This is the parent process\n";
    const char child_msg[] = "This is the child process\n";

    if (pid > 0) {
        // Parent process (pid > 0)
        syscall_write(1, parent_msg, sizeof(parent_msg));  // Write parent message to stdout
    } else if (pid == 0) {
        // Child process (pid == 0)
        syscall_write(1, child_msg, sizeof(child_msg));   // Write child message to stdout
    } else {
        // Fork failed (pid < 0), exit with error code
        syscall_exit(1);
    }

    syscall_exit(0);  // Exit the process
    return 0;         // This will never be reached
}
