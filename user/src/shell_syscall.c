#define SYSCALL_READ    63
#define SYSCALL_WRITE   64
#define SYSCALL_EXIT    93
#define SYSCALL_FORK    220
#define SYSCALL_EXECVE  221
#define SYSCALL_WAITPID 260
#define SYSCALL_YIELD   124

#define BUF_SIZE 128

#define NULL ((void*)0)

// Inline assembly for `read` syscall
static inline long syscall_read(int fd, char *buf, long count) {
    long ret;
    asm volatile (
        "mv a7, %[syscall_num]\n"
        "mv a0, %[fd]\n"
        "mv a1, %[buf]\n"
        "mv a2, %[count]\n"
        "ecall\n"
        "mv %[ret], a0\n"
        : [ret] "=r" (ret)
        : [syscall_num] "r" (SYSCALL_READ), [fd] "r" (fd), [buf] "r" (buf), [count] "r" (count)
        : "a0", "a1", "a2", "a7"
    );
    return ret;
}

// Inline assembly for `write` syscall
static inline long syscall_write(int fd, const char *buf, long count) {
    long ret;
    asm volatile (
        "mv a7, %[syscall_num]\n"
        "mv a0, %[fd]\n"
        "mv a1, %[buf]\n"
        "mv a2, %[count]\n"
        "ecall\n"
        "mv %[ret], a0\n"
        : [ret] "=r" (ret)
        : [syscall_num] "r" (SYSCALL_WRITE), [fd] "r" (fd), [buf] "r" (buf), [count] "r" (count)
        : "a0", "a1", "a2", "a7"
    );
    return ret;
}

// Inline assembly for `fork` syscall
static inline long syscall_fork() {
    long ret;
    asm volatile (
        "mv a7, %[syscall_num]\n"
        "ecall\n"
        "mv %[ret], a0\n"
        : [ret] "=r" (ret)
        : [syscall_num] "r" (SYSCALL_FORK)
        : "a0", "a7"
    );
    return ret;
}

// Inline assembly for `execve` syscall
static inline long syscall_execve(const char *path, char *const argv[], char *const envp[]) {
    long ret;
    asm volatile (
        "mv a7, %[syscall_num]\n"
        "mv a0, %[path]\n"
        "mv a1, %[argv]\n"
        "mv a2, %[envp]\n"
        "ecall\n"
        "mv %[ret], a0\n"
        : [ret] "=r" (ret)
        : [syscall_num] "r" (SYSCALL_EXECVE), [path] "r" (path), [argv] "r" (argv), [envp] "r" (envp)
        : "a0", "a1", "a2", "a7"
    );
    return ret;
}

// Inline assembly for `waitpid` syscall
static inline long syscall_waitpid(long pid, int *wstatus, int options) {
    long ret;
    asm volatile (
        "mv a7, %[syscall_num]\n"
        "mv a0, %[pid]\n"
        "mv a1, %[wstatus]\n"
        "mv a2, %[options]\n"
        "ecall\n"
        "mv %[ret], a0\n"
        : [ret] "=r" (ret)
        : [syscall_num] "r" (SYSCALL_WAITPID), [pid] "r" (pid), [wstatus] "r" (wstatus), [options] "r" (options)
        : "a0", "a1", "a2", "a7"
    );
    return ret;
}

// Inline assembly for `exit` syscall
static inline void syscall_exit(int exit_code) {
    asm volatile (
        "mv a7, %[syscall_num]\n"
        "mv a0, %[exit_code]\n"
        "ecall\n"
        :
        : [syscall_num] "r" (SYSCALL_EXIT), [exit_code] "r" (exit_code)
        : "a0", "a7"
    );
}

// Inline assembly for `yield` syscall
static inline void syscall_yield() {
    asm volatile (
        "mv a7, %[syscall_num]\n"
        "ecall\n"
        :
        : [syscall_num] "r" (SYSCALL_YIELD)
        : "a7"
    );
}

// Function to print the prompt and read user input
void read_command(char *buffer, long buf_size) {
    syscall_write(1, "\r\n$ ", 4);  // Display prompt
    syscall_read(0, buffer, buf_size);  // Read input from stdin
    syscall_yield();  // Yield the CPU to allow other processes to run
}

// Function to strip newline character from the command
void strip_newline(char *buffer) {
    while (*buffer) {
        if (*buffer == '\n' || *buffer == '\r') {
            *buffer = '\0';
            break;
        }
        buffer++;
    }
}

int _start() {
    char buffer[BUF_SIZE];
    char *argv[] = {buffer, NULL};  // argv for execve
    char *envp[] = {NULL};  // Empty environment
    
    while (1) {
        // Read the command from the user
        read_command(buffer, BUF_SIZE);
        strip_newline(buffer);  // Remove the newline from the command

        // Check for exit command
        if (buffer[0] == '\0') {
            continue;  // If no command is given, go back to the prompt
        } else if (buffer[0] == 'e' && buffer[1] == 'x' && buffer[2] == 'i' && buffer[3] == 't') {
            syscall_exit(0);  // Exit the shell
        }

        // Fork the process
        long pid = syscall_fork();
        if (pid == 0) {
            char* p;
            for (p = buffer; *p && (*p == ' ' || *p == '\t'); p++);
            int empty = (*p == '\0');  // Check if the command is empty
            // In child process, execute the command
            if (!empty && syscall_execve(buffer, argv, envp) < 0) {
                // If execve fails, print an error message and exit
                syscall_write(1, "Command not found\r\n", 18);
                syscall_exit(1);
            } else {
                syscall_write(1, "\r\n", 2);  // Print a newline after the command is executed
                *buffer = '\0';  // Reset the buffer for the next command
            }
            
        } else {
            // In parent process, wait for the child to complete
            syscall_waitpid(pid, NULL, 0);
        }
    }

    return 0;
}
