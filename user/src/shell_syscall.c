#include "syscall_test.h"

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
