#define SYSCALL_READ    63
#define SYSCALL_WRITE   64
#define SYSCALL_EXIT    93
#define SYSCALL_FORK    220
#define SYSCALL_EXECVE  221
#define SYSCALL_WAITPID 260
#define SYSCALL_YIELD   124
#define SYSCALL_OPEN    56
#define SYSCALL_CLOSE   57

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

// Inline assembly for `write` syscall
static inline int syscall_open(const char *path, int flags) {
    long ret;
    asm volatile (
        "mv a7, %[syscall_num]\n"
        "mv a1, %[path]\n"
        "mv a2, %[flags]\n"
        "ecall\n"
        "mv %[ret], a0\n"
        : [ret] "=r" (ret)
        : [syscall_num] "r" (SYSCALL_OPEN), [path] "r" (path), [flags] "r" (flags)
        : "a0", "a1", "a2", "a7"
    );
    return ret;
}

static inline int syscall_close(int fd) {
    long ret;
    asm volatile (
        "mv a7, %[syscall_num]\n"
        "mv a0, %[fd]\n"
        "ecall\n"
        "mv %[ret], a0\n"
        : [ret] "=r" (ret)
        : [syscall_num] "r" (SYSCALL_CLOSE), [fd] "r" (fd)
        : "a0", "a7"
    );
    return ret;
}


static inline void write_num(int fd, int num) {
    char buf[10];
    int i = 0;
    do {
        buf[i++] = '0' + num % 10;
        num /= 10;
    } while (num > 0);
    while (i > 0) {
        syscall_write(fd, &buf[--i], 1);
    }
}