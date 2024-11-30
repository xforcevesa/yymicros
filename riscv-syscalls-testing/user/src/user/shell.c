#include "stdio.h"
#include "unistd.h"
#include "string.h"

// Parse command line arguments from stdin input and execute the command
int main(int argc, char *argv[]) {
    const int MAX_CMD_LEN = 4096;
    const int MAX_ARGS = 100;
    char cmd[MAX_CMD_LEN];
    int i;

    // Read command line from stdin
    while (1) {
        printf("shell> ");
        gets(cmd, 100);
        cmd[strcspn(cmd, "\n")] = 0;

        // Split command line into arguments
        char *args[MAX_ARGS];
        int arg_count = 0;
        char *token = strtok(cmd, " ");
        while (token!= NULL) {
            args[arg_count] = token;
            arg_count++;
            token = strtok(NULL, " ");
        }

        // Execute command
        if (strcmp(args[0], "exit") == 0) {
            return 0;
        } else if (arg_count > 0) {
            // use execve to execute the command
            execve(args[0], args, NULL);
        } else {
            printf("Command not found: %s\n", args[0]);
        }
    }

    return 0;
}
