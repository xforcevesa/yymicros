#!/bin/bash

# 统计 .rs 和 .c 文件的总行数，排除./rustsbi-qemu-build和./target。
total_lines=$(
    find . -path ./rustsbi-qemu-build -prune -o -path ./target \
        -prune -o -type f \
        \( -name "*.rs" -o -name "*.c" -o -name "*.S" -o -name "*.asm" \) \
        -exec wc -l {} + | tail -1 | awk '{total += $1} END {print total}'
)

echo "Total lines in .rs and .c files: $total_lines"
