// Define system call numbers for RISC-V
#define SYSCALL_READ 63
#define SYSCALL_WRITE 64
#define SYSCALL_EXIT 93

// Inline assembly function for `read` syscall
static inline long syscall_read(int fd, char *buf, long count) {
    long ret;
    asm volatile (
        "mv a7, %[syscall_num]\n"   // Move syscall number to a7
        "mv a0, %[fd]\n"            // Move file descriptor to a0 (stdin = 0)
        "mv a1, %[buf]\n"           // Move buffer address to a1
        "mv a2, %[count]\n"         // Move number of bytes to read to a2
        "ecall\n"                   // Make the syscall
        "mv %[ret], a0\n"           // Store return value (number of bytes read) in ret
        : [ret] "=r" (ret)
        : [syscall_num] "r" (SYSCALL_READ), [fd] "r" (fd), [buf] "r" (buf), [count] "r" (count)
        : "a0", "a1", "a2", "a7"
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

int _start() {
    const char hello[] = "Hello World in ELF!\n";

    // Write the input back to stdout (fd = 1)
    syscall_write(1, hello, sizeof(hello));

    // Exit the program with code 0
    syscall_exit(0);

    return 0;  // This return won't be reached
}
