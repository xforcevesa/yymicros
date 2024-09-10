riscv64-unknown-elf-gcc -o elf/hello_syscall.elf src/hello_syscall.c -nostdlib -static
riscv64-unknown-elf-gcc -o elf/fork_syscall.elf src/fork_syscall.c -nostdlib -static
