# yymicros

An implementation of the barebone on RISC-V architecture microcontroller.

It was first started at the beginning of 2024 and had once stopped.

But now, it reborns again and continues to live.

It'll be a fun project to work on and a composition for the oscomp2025.

## Features

[ 2024.11.17 ] Modified Slab Global Alllocator Added, at `crates/lab_allocator` and `crates/new_slab_allocator` directory.

## VSCode Configuration

```json
{
    "rust-analyzer.cargo.target": "riscv64gc-unknown-none-elf",
    "rust-analyzer.check.allTargets": false
}
```

## Running Steps

You should install a rustup and riscv64-unknown-elf-gcc toolchain, ensuring they are in your PATH.

```bash
rustup default nightly
rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils
rustup component add llvm-tools-preview
wget https://github.com/rustsbi/rustsbi-qemu/releases/download/v0.1.1/rustsbi-qemu-release.zip
unzip rustsbi-qemu-release.zip
# Bootstrap the OS on QEMU
make run
# Clean up
make clean
# Count out the lines of source
make total_lines
```

## Contents

rCore experiments.

|Name|Note|
|:-:|:-:|
|os1|hello-world to no-std|
