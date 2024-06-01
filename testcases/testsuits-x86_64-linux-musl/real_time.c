#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>

int main() {
    time_t current_time;
    struct tm *time_info;
    char time_string[40];
    char *time_zone;

    // 获取当前时间
    time(&current_time);
    time_info = localtime(&current_time);

    // 将时间转换为本地时间字符串
    strftime(time_string, sizeof(time_string), "%Y-%m-%d %H:%M:%S %Z", time_info);

    printf("Current local time: %s\n", time_string);

    return 0;
}
