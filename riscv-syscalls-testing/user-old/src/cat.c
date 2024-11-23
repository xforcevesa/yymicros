#include "syscall_test.h"

void _start() {
    const char name[] = "/yes/no3";
    
    syscall_write(1, "Open file: ", 12);
    syscall_write(1, name, sizeof(name));
    syscall_write(1, "\n", 1);
    
    int fd = syscall_open(name, 1 << 9);

    if (fd < 0) {
        syscall_write(1, "Failed to open file\n", 20);
        syscall_exit(1);
    }

    const char msg[] = "Hello world in FAT32!\n";

    const int msg_len = sizeof(msg);

    syscall_write(1, "Write file: ", 12);
    syscall_write(1, name, sizeof(name));
    syscall_write(1, "\n", 1);
    
    syscall_write(fd, msg, msg_len);

    syscall_close(fd);

    fd = syscall_open(name, 0);

    char msg_buff[msg_len + 2];

    syscall_read(fd, msg, msg_len);

    syscall_close(fd);

    msg_buff[msg_len] = '\0';

    syscall_write(1, "Read file: ", 11);
    syscall_write(1, name, sizeof(name));
    syscall_write(1, "\n", 1);
    syscall_write(1, "Content: ", 9);

    syscall_write(1, msg, msg_len);
    
    syscall_exit(0);
}
