#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int main() {
    char buf[256];
    ssize_t len;

    // 读取 /etc/localtime 符号链接的目标路径
    len = readlink("/etc/localtime", buf, sizeof(buf) - 1);

    if (len != -1) {
        buf[len] = '\0';  // 确保字符串结尾
        printf("Time zone information file: %s\n", buf);
    } else {
        perror("Error reading /etc/localtime");
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}