# yymicros

An implementation of the barebone on RISC-V architecture microcontroller.

It was first started at the beginning of 2024 and had once stopped.

But now, it reborns again and continues to live.

It'll be a fun project to work on and a composition for the oscomp2025.

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
make run
```

## Contents

rCore experiments.

|Name|Note|
|:-:|:-:|
|os1|hello-world to no-std|
